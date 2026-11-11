//! Public Candid types and their stable-memory encodings.

use crate::payments::{Collected, PaymentError, PaymentMethod, PaymentReceipt, Prices};
use candid::{CandidType, Decode, Encode, Principal};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::Storable;
use serde::Deserialize;
use std::borrow::Cow;

/// Implements `Storable` for a Candid type (unbounded size).
macro_rules! candid_storable {
    ($t:ty) => {
        impl Storable for $t {
            fn to_bytes(&self) -> Cow<'_, [u8]> {
                Cow::Owned(Encode!(self).expect("candid encode"))
            }
            fn into_bytes(self) -> Vec<u8> {
                Encode!(&self).expect("candid encode")
            }
            fn from_bytes(bytes: Cow<[u8]>) -> Self {
                Decode!(bytes.as_ref(), Self).expect("candid decode")
            }
            const BOUND: Bound = Bound::Unbounded;
        }
    };
}

#[derive(CandidType, Deserialize, Debug, Clone, PartialEq)]
pub enum ChronoError {
    /// The anonymous principal cannot call this method. Sign in first.
    AnonymousCaller,
    Unauthorized,
    NotFound,
    TimeLocked,
    InvalidInput(String),
    LimitExceeded(String),
    /// Another request from the same caller is still in progress.
    AlreadyProcessing,
    Payment(PaymentError),
    Internal(String),
}

pub type ChronoResult<T> = Result<T, ChronoError>;

/// Per-recipient wrapped key. `user` is a principal text, or `"public"` for
/// locks anyone can open once the unlock time has passed.
#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub struct UserKey {
    pub user: String,
    pub key: String,
}

pub const PUBLIC_RECIPIENT: &str = "public";

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Chronolock {
    pub id: u64,
    pub owner: Principal,
    pub creator: Principal,
    /// Encryption scheme version (see vetkd.rs). Titles live inside the
    /// encrypted metadata, so nothing about the content is public.
    pub scheme: u8,
    /// Unix time in seconds.
    pub unlock_time: u64,
    /// Unix time in milliseconds.
    pub created_at: u64,
    pub user_keys: Vec<UserKey>,
    /// Base64 of the AES-GCM encrypted metadata payload (includes the title).
    pub encrypted_metadata: String,
    pub media: Option<MediaRef>,
    pub payment: Option<PaymentReceipt>,
}
candid_storable!(Chronolock);

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub struct MediaRef {
    pub media_id: u64,
    pub size: u64,
    pub chunk_count: u32,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CreateChronolockArgs {
    /// Must equal `vetkd::SCHEME_VERSION`.
    pub scheme: u8,
    pub unlock_time: u64,
    pub user_keys: Vec<UserKey>,
    pub encrypted_metadata: String,
    /// A finished upload session owned by the caller.
    pub media_id: Option<u64>,
    /// Ledger to pay with; approve price + fee for this canister first.
    pub payment: PaymentMethod,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Page {
    pub offset: u64,
    pub limit: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct LockPage {
    pub items: Vec<Chronolock>,
    pub total: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Stats {
    pub total_locks: u64,
    pub unique_creators: u64,
    pub media_locks: u64,
    pub public_locks: u64,
}

// ---------------------------------------------------------------------------
// Media upload sessions
// ---------------------------------------------------------------------------

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct MediaSession {
    pub owner: Principal,
    pub size: u64,
    pub chunk_count: u32,
    pub received: Vec<bool>,
    pub created_at_ns: u64,
    pub finished: bool,
    /// Set once a chronolock references this media; committed media never expires.
    pub lock_id: Option<u64>,
}
candid_storable!(MediaSession);

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct StartMediaUploadArgs {
    pub size: u64,
    pub chunk_count: u32,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct MediaInfo {
    pub media_id: u64,
    pub owner: Principal,
    pub size: u64,
    pub chunk_count: u32,
    pub finished: bool,
    pub lock_id: Option<u64>,
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct InitArgs {
    /// vetKD master key: `test_key_1` for local/testing, `key_1` for production.
    /// Fixed at first install; ignored on upgrade.
    pub vetkd_key_name: Option<String>,
    /// Extra admins besides the canister controllers.
    pub admins: Option<Vec<Principal>>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Limits {
    pub max_metadata_bytes: u64,
    pub max_media_bytes: u64,
    pub max_chunk_bytes: u64,
    pub max_open_uploads_per_user: u32,
    pub upload_ttl_seconds: u64,
    pub max_recipients: u32,
    /// vetKD derivations allowed per caller per hour (protects cycles).
    pub key_derivations_per_hour: u32,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_metadata_bytes: 64 * 1024,
            max_media_bytes: 10 * 1024 * 1024,
            max_chunk_bytes: 1_900_000,
            max_open_uploads_per_user: 2,
            upload_ttl_seconds: 60 * 60,
            max_recipients: 50,
            key_derivations_per_hour: 60,
        }
    }
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Config {
    pub vetkd_key_name: String,
    pub limits: Limits,
    pub next_lock_id: u64,
    pub next_media_id: u64,
    pub next_tx_id: u64,
    /// $CHRONO token launched on Soon. `None` until an admin sets it.
    pub token: Option<TokenConfig>,
    pub prices: Prices,
    pub collected: Collected,
}
candid_storable!(Config);

/// An ICRC-1/ICRC-2 token launched on Soon (https://soonlaunchpad.fun).
#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub struct TokenConfig {
    pub ledger: Principal,
    /// Soon market canister, used for the token page links (`/t/<market>`).
    pub market: Option<Principal>,
    pub symbol: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            vetkd_key_name: "test_key_1".to_string(),
            limits: Limits::default(),
            next_lock_id: 1,
            next_media_id: 1,
            next_tx_id: 0,
            token: None,
            prices: Prices::default(),
            collected: Collected::default(),
        }
    }
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct PublicConfig {
    pub vetkd_key_name: String,
    pub scheme: u8,
    pub limits: Limits,
    pub token: Option<TokenConfig>,
}
