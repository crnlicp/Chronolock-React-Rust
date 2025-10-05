// src/backend/chronolock/src/lib.rs

use base64::{engine::general_purpose, Engine as _};
use candid::{CandidType, Principal};
use ic_cdk::api::call::call_with_payment;
use ic_cdk::api::time;
use ic_cdk::caller;
use ic_cdk_macros::{init, query, update};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl, StableBTreeMap, StableCell, Storable,
};
use serde::Deserialize;
use serde::Serialize;
use std::borrow::Cow;
use std::cell::RefCell;

type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(CandidType, Deserialize)]
enum ChronoError {
    Unauthorized,
    TokenNotFound,
    MetadataTooLarge,
    TimeLocked,
    InvalidInput(String),
    InternalError(String),
    // Authentication-related errors
    NotAuthenticated,
    InvalidPrincipal,
    UnauthorizedCaller,
    AdminRequired,
}

#[derive(CandidType, Deserialize)]
struct HttpRequest {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

#[derive(CandidType)]
struct HttpResponse {
    status_code: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

#[derive(CandidType, Deserialize)]
struct VetKDPublicKeyArgs {
    canister_id: Option<Principal>,
    context: Vec<u8>,
    key_id: VetKDKeyId,
}

#[derive(CandidType, Deserialize)]
struct VetKDKeyId {
    curve: VetKDCurve,
    name: String,
}

#[derive(CandidType, Deserialize)]
enum VetKDCurve {
    #[serde(rename = "bls12_381_g2")]
    #[allow(non_camel_case_types)]
    Bls12_381_G2,
}

#[derive(CandidType, Deserialize)]
pub struct VetKDPublicKeyReply {
    pub public_key: Vec<u8>,
}

#[derive(CandidType, Deserialize)]
struct VetKDDeriveKeyArgs {
    input: Vec<u8>,
    context: Vec<u8>,
    transport_public_key: Vec<u8>,
    key_id: VetKDKeyId,
}

#[derive(CandidType, Deserialize)]
pub struct VetKDDeriveKeyReply {
    pub encrypted_key: Vec<u8>,
}

#[derive(CandidType, Deserialize, Clone)]
struct Chronolock {
    id: String,
    owner: Principal,
    metadata: String, // hex encoded metadata as MetaData
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MetaData {
    pub title: Option<String>,                // Optional title for the NFT
    pub owner: String,                        // Principal ID of the owner
    pub unlock_time: u64,                     // Unix timestamp in seconds
    pub created_at: Option<u64>, // Unix timestamp in seconds when chronolock was created
    pub user_keys: Option<serde_json::Value>, // Map of user principal to their encrypted keys
    pub encrypted_metadata: String, // hex encoded encrypted metadata as EncryptedMetadataPayload
}

// Example for the decrypted encrypted_metadata payload to be used in Frontend:
#[derive(Serialize, Deserialize, Clone)]
pub struct EncryptedMetadataPayload {
    pub name: Option<String>,                  // Optional name for the NFT
    pub description: Option<String>,           // Optional description
    pub file_type: Option<String>,             // MIME type, optional
    pub media_id: Option<String>,              // ID of the media file, if any
    pub media_size: Option<u64>,               // Size of the media file
    pub attributes: Option<serde_json::Value>, // Arbitrary user key-values
}

impl Storable for Chronolock {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(candid::encode_one(self).expect("Failed to encode Chronolock"))
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).expect("Failed to decode Chronolock")
    }
    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Clone)]
struct LogEntry {
    id: String,
    timestamp: u64,
    activity: String,
}

impl Storable for LogEntry {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(candid::encode_one(self).expect("Failed to encode LogEntry"))
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).expect("Failed to decode LogEntry")
    }
    const BOUND: Bound = Bound::Bounded {
        max_size: 1024,
        is_fixed_size: false,
    };
}

#[derive(CandidType, Deserialize, Clone)]
struct TokenList {
    tokens: Vec<String>,
}

impl Storable for TokenList {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(candid::encode_one(&self.tokens).expect("Failed to encode TokenList"))
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Self {
            tokens: candid::decode_one(&bytes).expect("Failed to decode TokenList"),
        }
    }
    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Clone)]
struct MediaUploadState {
    total_chunks: u32,
    received_chunks: u32,
    chunks: Vec<Option<Vec<u8>>>,
}

impl Storable for MediaUploadState {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(candid::encode_one(self).expect("Failed to encode MediaUploadState"))
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).expect("Failed to decode MediaUploadState")
    }
    const BOUND: Bound = Bound::Unbounded;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
    static LOGS: RefCell<StableBTreeMap<String, LogEntry, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))))
    );
    static ADMINS: RefCell<StableBTreeMap<u8, Principal, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))))
    );
    static MAX_METADATA_SIZE: RefCell<StableCell<u64, Memory>> = RefCell::new(
        StableCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))), 51200)
        .unwrap_or_else(|e| panic!("Failed to initialize MAX_METADATA_SIZE: {:?}", e))
    );
    static LAST_TIMESTAMP: RefCell<StableCell<u64, Memory>> = RefCell::new(
        StableCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3))), 0)
            .unwrap_or_else(|e| panic!("Failed to initialize LAST_TIMESTAMP: {:?}", e))
    );
    static COUNTER: RefCell<StableCell<u64, Memory>> = RefCell::new(
        StableCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4))), 0)
            .unwrap_or_else(|e| panic!("Failed to initialize COUNTER: {:?}", e))
    );
    static CHRONOLOCKS: RefCell<StableBTreeMap<String, Chronolock, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(5))))
    );
    static OWNER_TO_TOKENS: RefCell<StableBTreeMap<Principal, TokenList, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(6))))
    );
    static MEDIA_FILES: RefCell<StableBTreeMap<String, Vec<u8>, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(7))))
    );
    static NETWORK: RefCell<StableCell<Option<String>, Memory>> = RefCell::new(
        StableCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(8))), None)
            .unwrap_or_else(|e| panic!("Failed to initialize NETWORK: {:?}", e))
    );
    static MEDIA_UPLOADS: RefCell<StableBTreeMap<String, MediaUploadState, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(9))))
    );
    static SYMBOL: RefCell<String> = RefCell::new("CHRONOLOCK".to_string());
    static NAME: RefCell<String> = RefCell::new("Chronolock Collection".to_string());
    static DESCRIPTION: RefCell<String> = RefCell::new("A collection of time-locked NFTs".to_string());
    // Whitelist for trusted principals (e.g., Internet Identity principals)
    static TRUSTED_PRINCIPALS: RefCell<StableBTreeMap<Principal, bool, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(10))))
    );
    // Authentication bypass for admin operations
    static ADMIN_BYPASS_ENABLED: RefCell<StableCell<bool, Memory>> = RefCell::new(
        StableCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(11))), false)
            .unwrap_or_else(|e| panic!("Failed to initialize ADMIN_BYPASS_ENABLED: {:?}", e))
    );
}

fn generate_unique_id() -> String {
    let current_time = time();
    let mut counter = 0;
    LAST_TIMESTAMP.with(|last| {
        COUNTER.with(|cnt| {
            let mut last_time = last.borrow_mut();
            let mut count = cnt.borrow_mut();
            if *last_time.get() != current_time {
                last_time
                    .set(current_time)
                    .expect("Failed to set LAST_TIMESTAMP");
                count.set(0).expect("Failed to set COUNTER");
            } else {
                let new_count = *count.get() + 1;
                count.set(new_count).expect("Failed to increment COUNTER");
            }
            counter = *count.get();
        })
    });
    format!("{}-{}", current_time, counter)
}

const MAX_LOGS: u64 = 10_000;

fn log_activity(activity: String) {
    let sanitized_activity = if activity.len() > 100 {
        format!("{}...", &activity[..97])
    } else {
        activity
    };
    LOGS.with(|logs| {
        let mut logs = logs.borrow_mut();
        if logs.len() >= MAX_LOGS {
            if let Some(oldest_key) = logs.keys().next().map(|k| k.clone()) {
                logs.remove(&oldest_key);
            }
        }
        let id = generate_unique_id();
        let entry = LogEntry {
            id: id.clone(),
            timestamp: time(),
            activity: sanitized_activity,
        };
        logs.insert(id, entry);
    });
}

fn get_vetkd_key_name() -> String {
    match get_network().as_deref() {
        Some("local") => "dfx_test_key".to_string(),
        _ => "test_key_1".to_string(), // Use test_key_1 for IC mainnet
    }
}

async fn call_vetkd_derive_key(
    input: Vec<u8>,
    context: Vec<u8>,
    transport_public_key: Vec<u8>,
) -> Result<VetKDDeriveKeyReply, ChronoError> {
    let args = VetKDDeriveKeyArgs {
        key_id: VetKDKeyId {
            name: get_vetkd_key_name(),
            curve: VetKDCurve::Bls12_381_G2,
        },
        input,
        context,
        transport_public_key,
    };

    let management_canister = Principal::management_canister();

    // VetKD operations require cycles to be sent with the call
    // Based on the error message, approximately 26 billion cycles are needed
    let cycles = 30_000_000_000u64; // 30 billion cycles for safety margin

    let (result,): (VetKDDeriveKeyReply,) =
        call_with_payment(management_canister, "vetkd_derive_key", (args,), cycles)
            .await
            .map_err(|e| ChronoError::InternalError(format!("Call failed: {:?}", e)))?;

    Ok(result)
}

#[init]
fn init(admin: Principal, network: Option<String>) {
    ADMINS.with(|admins| {
        admins.borrow_mut().insert(0, admin);
    });

    if let Some(net) = network {
        NETWORK.with(|n| {
            n.borrow_mut()
                .set(Some(net))
                .expect("Failed to set NETWORK")
        });
    }

    log_activity(format!("Canister initialized with admin: {}", admin));
}

fn is_admin(caller: Principal) -> bool {
    ADMINS.with(|admins| admins.borrow().get(&0) == Some(caller))
}

fn get_network() -> Option<String> {
    NETWORK.with(|n| n.borrow().get().clone())
}

// Decode metadata which may be hex-encoded or base64-encoded into JSON Value
fn decode_metadata_value(metadata_encoded: &str) -> Option<serde_json::Value> {
    // Try hex decode first
    if let Ok(bytes) = hex::decode(metadata_encoded) {
        if let Ok(s) = String::from_utf8(bytes) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
                return Some(v);
            }
        }
    }

    // Fallback to base64
    if let Ok(bytes) = general_purpose::STANDARD.decode(metadata_encoded) {
        if let Ok(s) = String::from_utf8(bytes) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
                return Some(v);
            }
        }
    }

    None
}

// -------------------------
// Authentication Helper Functions
// -------------------------

// Check if a principal is a valid Internet Identity principal
// Internet Identity principals have specific characteristics:
// - They are not anonymous
// - They follow a specific textual format with hyphens
// - Real II principals like: 4s3y7-25yvt-jbdte-vpvcq-n4ghs-j5jo6-beihs-om2zi-oqzu6-krbhf-gqe
//   are 29-byte self-authenticating principals (ending with 0x02) with proper random-looking segments
// - Test/mock II principals may be shorter (10 bytes, ending with 0x01)
// - We reject simple/mock self-auth principals with many repeated segments (like "aaaaa")
fn is_valid_internet_identity_principal(principal: Principal) -> bool {
    // Reject anonymous outright
    if principal == Principal::anonymous() {
        return false;
    }

    // If principal is in the trusted whitelist, accept immediately
    let trusted = TRUSTED_PRINCIPALS.with(|tp| tp.borrow().get(&principal).unwrap_or(false));
    if trusted {
        return true;
    }

    let principal_bytes = principal.as_slice();

    // Reject management canister principal explicitly
    if principal.to_text() == "aaaaa-aa" {
        return false;
    }

    // Enforce textual structure to match proper II principals
    let text = principal.to_text();

    // Principal text must have proper length and contain hyphens
    if text.len() < 10 || text.len() > 63 || !text.contains('-') {
        return false;
    }

    // Validate the format: segments separated by hyphens
    // Each segment must be non-empty, max 5 characters, lowercase alphanumeric
    if !text.split('-').all(|segment| {
        !segment.is_empty()
            && segment.len() <= 5
            && segment.chars().all(|c| matches!(c, 'a'..='z' | '0'..='9'))
    }) {
        return false;
    }

    // For self-authenticating principals (29 bytes ending with 0x02):
    // Reject if they look like simple test/mock principals
    if principal_bytes.len() == 29 && principal_bytes.last() == Some(&0x02) {
        let segments: Vec<&str> = text.split('-').collect();

        // Count segments that are just "aaaaa" or similar repeated patterns
        let repeated_count = segments
            .iter()
            .filter(|s| {
                if s.len() < 3 {
                    return false;
                }
                let chars: Vec<char> = s.chars().collect();
                chars
                    .windows(3)
                    .all(|w| w[0] == 'a' && w[1] == 'a' && w[2] == 'a')
            })
            .count();

        // Reject if more than half the segments look like test data
        if repeated_count > segments.len() / 2 {
            return false;
        }
    }

    // Accept shorter principals (e.g., 10 bytes ending with 0x01 for test/mock II principals)
    // and proper self-auth principals that pass validation
    principal_bytes.len() >= 10
}

// Validate that the caller is properly authenticated
fn validate_caller_authentication() -> Result<Principal, ChronoError> {
    let caller = caller();

    // Allow admin to bypass authentication if enabled (only if the caller is admin)
    // NOTE: previous logic allowed ANY caller to be treated as authenticated when
    // ADMIN_BYPASS_ENABLED was true. That effectively disabled authentication for
    // all callers and is a critical security bug. Require the caller to be an admin
    // when bypass is enabled, matching the intended behavior used in the ledger
    // canister.
    if ADMIN_BYPASS_ENABLED.with(|ab| ab.borrow().get().clone()) && is_admin(caller) {
        return Ok(caller);
    }

    // Check if the caller is a trusted principal
    if TRUSTED_PRINCIPALS.with(|tp| tp.borrow().get(&caller).unwrap_or(false)) {
        return Ok(caller);
    }

    // Check if the caller is a valid Internet Identity principal
    if !is_valid_internet_identity_principal(caller) {
        log_activity(format!(
            "Authentication failure: Invalid principal attempted access: {}",
            caller
        ));
        return Err(ChronoError::NotAuthenticated);
    }

    Ok(caller)
}

// Validate admin authentication (stricter requirements)
fn validate_admin_authentication() -> Result<Principal, ChronoError> {
    let caller = validate_caller_authentication()?;

    if !is_admin(caller) {
        log_activity(format!("Unauthorized admin access attempt: {}", caller));
        return Err(ChronoError::AdminRequired);
    }

    Ok(caller)
}

#[update]
fn set_max_metadata_size(new_size: u64) -> Result<(), ChronoError> {
    // Validate admin authentication
    let _authenticated_admin = validate_admin_authentication()?;
    MAX_METADATA_SIZE.with(|size| {
        size.borrow_mut()
            .set(new_size)
            .expect("Failed to set MAX_METADATA_SIZE");
        log_activity(format!("MAX_METADATA_SIZE updated to {}", new_size));
        Ok(())
    })
}

#[query]
fn get_logs_paginated(offset: u64, limit: u64) -> Result<Vec<LogEntry>, ChronoError> {
    // Validate admin authentication for log access
    let _authenticated_admin = validate_admin_authentication()?;
    Ok(LOGS.with(|logs| {
        logs.borrow()
            .iter()
            .skip(offset as usize)
            .take(limit as usize)
            .map(|(_, entry)| entry.clone())
            .collect()
    }))
}

#[query]
fn get_logs_by_range(start_time: u64, end_time: u64) -> Result<Vec<LogEntry>, ChronoError> {
    // Validate admin authentication for log access
    let _authenticated_admin = validate_admin_authentication()?;
    Ok(LOGS.with(|logs| {
        logs.borrow()
            .iter()
            .map(|(_, entry)| entry.clone())
            .filter(|e| e.timestamp >= start_time && e.timestamp <= end_time)
            .collect()
    }))
}

#[query]
fn icrc7_symbol() -> String {
    SYMBOL.with(|s| s.borrow().clone())
}

#[query]
fn icrc7_name() -> String {
    NAME.with(|n| n.borrow().clone())
}

#[query]
fn icrc7_description() -> String {
    DESCRIPTION.with(|d| d.borrow().clone())
}

#[query]
fn icrc7_total_supply() -> u64 {
    CHRONOLOCKS.with(|locks| locks.borrow().len() as u64)
}

#[query]
fn icrc7_balance_of(account: Principal) -> u64 {
    OWNER_TO_TOKENS.with(|owner_to_tokens| {
        owner_to_tokens
            .borrow()
            .get(&account)
            .map(|list| list.tokens.len() as u64)
            .unwrap_or(0)
    })
}

// Returns a list of token IDs owned by the specified account.
#[query]
fn icrc7_owner_of(token_id: String) -> Option<Principal> {
    CHRONOLOCKS.with(|locks| locks.borrow().get(&token_id).map(|lock| lock.owner))
}

#[query]
fn icrc7_token_metadata(token_id: String) -> Option<String> {
    CHRONOLOCKS.with(|locks| {
        locks
            .borrow()
            .get(&token_id)
            .map(|lock| lock.metadata.clone())
    })
}

#[update]
fn icrc7_transfer(token_id: String, to: Principal) -> Result<(), ChronoError> {
    // Validate caller authentication
    let authenticated_caller = validate_caller_authentication()?;
    CHRONOLOCKS.with(|locks| {
        OWNER_TO_TOKENS.with(|owner_to_tokens| {
            let mut locks = locks.borrow_mut();
            let mut owner_to_tokens = owner_to_tokens.borrow_mut();
            let lock = locks
                .get(&token_id)
                .ok_or(ChronoError::TokenNotFound)?
                .clone();
            if lock.owner != authenticated_caller {
                return Err(ChronoError::Unauthorized);
            }
            let mut caller_tokens = owner_to_tokens
                .get(&authenticated_caller)
                .map(|t| t.clone())
                .unwrap_or(TokenList { tokens: vec![] });
            if !caller_tokens.tokens.contains(&token_id) {
                return Err(ChronoError::TokenNotFound);
            }
            caller_tokens.tokens.retain(|id| id != &token_id);
            let mut to_tokens = owner_to_tokens
                .get(&to)
                .map(|t| t.clone())
                .unwrap_or(TokenList { tokens: vec![] });
            to_tokens.tokens.push(token_id.clone());
            let mut updated_lock = lock;
            updated_lock.owner = to;
            locks.insert(token_id.clone(), updated_lock);
            owner_to_tokens.insert(authenticated_caller, caller_tokens);
            owner_to_tokens.insert(to, to_tokens);
            log_activity(format!("Transferred token {} to {}", token_id, to));
            Ok(())
        })
    })
}

#[update]
async fn ibe_encryption_key() -> Result<VetKDPublicKeyReply, ChronoError> {
    let args = VetKDPublicKeyArgs {
        key_id: VetKDKeyId {
            name: get_vetkd_key_name(),
            curve: VetKDCurve::Bls12_381_G2,
        },
        context: b"chronolock-encryption".to_vec(),
        canister_id: None,
    };

    let management_canister = Principal::management_canister();

    let (result,): (VetKDPublicKeyReply,) =
        ic_cdk::call(management_canister, "vetkd_public_key", (args,))
            .await
            .map_err(|e| ChronoError::InternalError(format!("Call failed: {:?}", e)))?;

    Ok(result)
}

#[update]
async fn get_time_decryption_key(
    unlock_time_hex: String,
    transport_public_key: Vec<u8>,
) -> Result<VetKDDeriveKeyReply, ChronoError> {
    if transport_public_key.is_empty() {
        return Err(ChronoError::InvalidInput(
            "Transport public key cannot be empty".to_string(),
        ));
    }
    if unlock_time_hex.len() != 16 || !unlock_time_hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ChronoError::InvalidInput(
            "unlock_time_hex must be a 16-character hexadecimal string".to_string(),
        ));
    }

    let unlock_time = u64::from_str_radix(&unlock_time_hex, 16)
        .map_err(|e| ChronoError::InvalidInput(format!("Invalid hex for unlock time: {}", e)))?;
    let unlock_time_ns = unlock_time * 1_000_000_000;

    let current_time_ns = time();
    if current_time_ns < unlock_time_ns {
        return Err(ChronoError::TimeLocked);
    }

    // Use IBE identity format for VetKD derivation to ensure compatibility
    // For public chronolocks, IBE identity is just the decimal time string
    let input = unlock_time.to_string().into_bytes();
    let context = b"chronolock-encryption".to_vec();

    call_vetkd_derive_key(input, context, transport_public_key).await
}

#[update]
async fn get_user_time_decryption_key(
    unlock_time_hex: String,
    user_id: String,
    transport_public_key: Vec<u8>,
) -> Result<VetKDDeriveKeyReply, ChronoError> {
    if transport_public_key.is_empty() {
        return Err(ChronoError::InvalidInput(
            "Transport public key cannot be empty".to_string(),
        ));
    }

    // Validate caller authentication
    let authenticated_caller = validate_caller_authentication()?;
    let authorized_principal = Principal::from_text(&user_id)
        .map_err(|e| ChronoError::InvalidInput(format!("Invalid user id: {}", e)))?;

    if authenticated_caller != authorized_principal {
        return Err(ChronoError::Unauthorized);
    }

    let unlock_time = u64::from_str_radix(&unlock_time_hex, 16)
        .map_err(|e| ChronoError::InvalidInput(format!("Invalid hex: {}", e)))?;
    let unlock_time_ns = unlock_time * 1_000_000_000; // Convert to nanoseconds
    let current_time_ns = time();
    if current_time_ns < unlock_time_ns {
        return Err(ChronoError::TimeLocked);
    }

    // Use IBE identity format for VetKD derivation to ensure compatibility
    // For private chronolocks, IBE identity is "user_id:decimal_time"
    let combined_id = format!("{}:{}", user_id, unlock_time);
    let input = combined_id.into_bytes();
    let context = b"chronolock-encryption".to_vec();

    call_vetkd_derive_key(input, context, transport_public_key).await
}

#[update]
fn create_chronolock(metadata: String) -> Result<String, ChronoError> {
    // Validate caller authentication
    let authenticated_caller = validate_caller_authentication()?;
    let metadata_size = metadata.len() as u64;
    let max_size = MAX_METADATA_SIZE.with(|size| *size.borrow().get());
    if metadata_size > max_size {
        return Err(ChronoError::MetadataTooLarge);
    }
    let id = generate_unique_id();
    let chronolock = Chronolock {
        id: id.clone(),
        owner: authenticated_caller,
        metadata,
    };
    CHRONOLOCKS.with(|locks| {
        locks.borrow_mut().insert(id.clone(), chronolock);
    });
    OWNER_TO_TOKENS.with(|owner_to_tokens| {
        let mut owner_to_tokens = owner_to_tokens.borrow_mut();
        let mut tokens = owner_to_tokens
            .get(&authenticated_caller)
            .map(|t| t.clone())
            .unwrap_or(TokenList { tokens: vec![] });
        tokens.tokens.push(id.clone());
        owner_to_tokens.insert(authenticated_caller, tokens);
    });
    log_activity(format!("Chronolock created with ID: {}", id));
    Ok(id)
}

#[update]
fn update_chronolock(token_id: String, metadata: String) -> Result<(), ChronoError> {
    // Validate caller authentication
    let authenticated_caller = validate_caller_authentication()?;
    CHRONOLOCKS.with(|locks| {
        let mut locks = locks.borrow_mut();
        let mut lock = locks
            .get(&token_id)
            .ok_or(ChronoError::TokenNotFound)?
            .clone();
        if lock.owner != authenticated_caller {
            return Err(ChronoError::Unauthorized);
        }
        if metadata.len() as u64 > MAX_METADATA_SIZE.with(|s| *s.borrow().get()) {
            return Err(ChronoError::MetadataTooLarge);
        }
        lock.metadata = metadata;
        locks.insert(token_id.clone(), lock);
        log_activity(format!("Updated chronolock {}", token_id));
        Ok(())
    })
}

#[update]
fn burn_chronolock(token_id: String) -> Result<(), ChronoError> {
    // Validate caller authentication
    let authenticated_caller = validate_caller_authentication()?;
    CHRONOLOCKS.with(|locks| {
        OWNER_TO_TOKENS.with(|owner_to_tokens| {
            let mut locks = locks.borrow_mut();
            let mut owner_to_tokens = owner_to_tokens.borrow_mut();
            let lock = locks.get(&token_id).ok_or(ChronoError::TokenNotFound)?;

            // Allow owner or admin to burn
            let is_owner = lock.owner == authenticated_caller;
            let caller_is_admin = is_admin(authenticated_caller);

            if !is_owner && !caller_is_admin {
                return Err(ChronoError::Unauthorized);
            }

            let owner = lock.owner;
            locks.remove(&token_id);

            // Remove from owner's token list
            if let Some(owner_tokens) = owner_to_tokens.get(&owner).map(|t| t.clone()) {
                let mut owner_tokens = owner_tokens;
                owner_tokens.tokens.retain(|id| id != &token_id);
                owner_to_tokens.insert(owner, owner_tokens);
            }
            log_activity(format!("Burned chronolock {}", token_id));
            Ok(())
        })
    })
}

#[update]
fn start_media_upload(total_chunks: u32) -> Result<String, ChronoError> {
    // Validate caller authentication
    let _authenticated_caller = validate_caller_authentication()?;

    let media_id = generate_unique_id();
    MEDIA_UPLOADS.with(|uploads| {
        uploads.borrow_mut().insert(
            media_id.clone(),
            MediaUploadState {
                total_chunks,
                received_chunks: 0,
                chunks: vec![None; total_chunks as usize],
            },
        );
    });
    log_activity(format!(
        "Started media upload: {} ({} chunks)",
        media_id, total_chunks
    ));
    Ok(media_id)
}

#[update]
fn upload_media_chunk(
    media_id: String,
    chunk_index: u32,
    chunk: Vec<u8>,
) -> Result<u32, ChronoError> {
    // Validate caller authentication
    let _authenticated_caller = validate_caller_authentication()?;

    const MAX_CHUNK_SIZE: usize = 2 * 1024 * 1024; // 2MB
    if chunk.len() > MAX_CHUNK_SIZE {
        return Err(ChronoError::InvalidInput(format!("Chunk size exceeds 2MB")));
    }
    let chunk_len = chunk.len() as u32;
    MEDIA_UPLOADS.with(|uploads| {
        let mut uploads = uploads.borrow_mut();
        let mut entry = uploads
            .get(&media_id)
            .ok_or_else(|| {
                ChronoError::InvalidInput("Invalid media_id for chunk upload".to_string())
            })?
            .clone();

        if chunk_index as usize >= entry.total_chunks as usize {
            return Err(ChronoError::InvalidInput("Invalid chunk index".to_string()));
        }
        if entry.chunks[chunk_index as usize].is_some() {
            return Err(ChronoError::InvalidInput(
                "Chunk already uploaded".to_string(),
            ));
        }
        entry.chunks[chunk_index as usize] = Some(chunk);
        entry.received_chunks += 1;

        uploads.insert(media_id, entry);
        Ok(chunk_len)
    })
}

#[update]
fn finish_media_upload(media_id: String) -> Result<String, ChronoError> {
    // Validate caller authentication
    let _authenticated_caller = validate_caller_authentication()?;

    const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB
    MEDIA_UPLOADS.with(|uploads| {
        let mut uploads = uploads.borrow_mut();
        let MediaUploadState {
            total_chunks,
            received_chunks,
            chunks,
            ..
        } = uploads
            .remove(&media_id)
            .ok_or_else(|| ChronoError::InvalidInput("Invalid media_id for finish".to_string()))?;
        if received_chunks != total_chunks {
            return Err(ChronoError::InvalidInput(format!(
                "Not all chunks uploaded: {}/{}",
                received_chunks, total_chunks
            )));
        }
        let mut file_data = Vec::new();
        for chunk in chunks.into_iter() {
            let chunk = chunk
                .ok_or_else(|| ChronoError::InvalidInput("Missing chunk in upload".to_string()))?;
            file_data.extend_from_slice(&chunk);
        }
        if file_data.len() > MAX_FILE_SIZE {
            return Err(ChronoError::InvalidInput(format!(
                "File size exceeds maximum of {} bytes",
                MAX_FILE_SIZE
            )));
        }
        MEDIA_FILES.with(|media| {
            media.borrow_mut().insert(media_id.clone(), file_data);
        });
        let canister_id = ic_cdk::id();
        let base_url = match get_network().as_deref() {
            Some("local") => format!("http://{}.localhost:4943", canister_id),
            _ => format!("https://{}.raw.ic0.app", canister_id),
        };
        let url = format!("{}/media/{}", base_url, media_id);
        log_activity(format!("Finished media upload: {}", media_id));
        Ok(url)
    })
}

#[query]
fn get_media_chunk(media_id: String, offset: u32, length: u32) -> Result<Vec<u8>, ChronoError> {
    MEDIA_FILES.with(|media| {
        if let Some(data) = media.borrow().get(&media_id) {
            let start = offset as usize;
            let end = std::cmp::min(start + length as usize, data.len());
            ic_cdk::println!(
                "Returning chunk: start={}, end={}, length={}",
                start,
                end,
                end - start
            );
            if start >= data.len() {
                ic_cdk::println!("Offset exceeds data length, returning empty chunk");
                return Ok(vec![]); // Return empty chunk if offset exceeds data length
            }
            Ok(data[start..end].to_vec()) // Return the requested slice
        } else {
            Err(ChronoError::TokenNotFound) // Return error if media_id is invalid
        }
    })
}

// Query to fetch a single chronolock by id
#[query]
fn get_chronolock(token_id: String) -> Result<Chronolock, ChronoError> {
    CHRONOLOCKS.with(|locks| {
        locks
            .borrow()
            .get(&token_id)
            .map(|c| c.clone())
            .ok_or(ChronoError::TokenNotFound)
    })
}

#[query]
fn http_request(request: HttpRequest) -> HttpResponse {
    if request.method != "GET" {
        return HttpResponse {
            status_code: 405,
            headers: vec![],
            body: b"Method not allowed".to_vec(),
        };
    }
    let path = request.url.split('?').next().unwrap_or("");
    if path.starts_with("/media/") {
        let media_id = path.strip_prefix("/media/").unwrap();
        MEDIA_FILES.with(|media| {
            if let Some(data) = media.borrow().get(&media_id.to_string()) {
                HttpResponse {
                    status_code: 200,
                    headers: vec![(
                        "Content-Type".to_string(),
                        "application/octet-stream".to_string(),
                    )],
                    body: data.clone(),
                }
            } else {
                HttpResponse {
                    status_code: 404,
                    headers: vec![],
                    body: b"Media not found".to_vec(),
                }
            }
        })
    } else {
        HttpResponse {
            status_code: 404,
            headers: vec![],
            body: b"Not found".to_vec(),
        }
    }
}

// Function to get total count of all chronolocks
#[query]
fn get_total_chronolocks_count() -> u64 {
    CHRONOLOCKS.with(|locks| locks.borrow().len() as u64)
}

// Function to get total count of unique creators
#[query]
fn get_unique_creators_count() -> u64 {
    OWNER_TO_TOKENS.with(|owner_to_tokens| owner_to_tokens.borrow().len() as u64)
}

// Function to get total count of owner's chronolocks
#[query]
fn get_owner_chronolocks_count(owner: Principal) -> u64 {
    OWNER_TO_TOKENS.with(|owner_to_tokens| {
        owner_to_tokens
            .borrow()
            .get(&owner)
            .map(|list| list.tokens.len() as u64)
            .unwrap_or(0)
    })
}

// Function to get total count of user accessible chronolocks
#[query]
fn get_user_accessible_chronolocks_count(user: Principal) -> u64 {
    let current_time = time();
    let user_text = user.to_text();

    CHRONOLOCKS.with(|locks| {
        locks
            .borrow()
            .iter()
            .filter(|(_, chronolock)| {
                if let Some(metadata_value) = decode_metadata_value(&chronolock.metadata) {
                    // Extract lockTime and userKeys from the parsed JSON
                    if let (Some(lock_time), Some(user_keys_array)) = (
                        metadata_value.get("lockTime").and_then(|v| v.as_u64()),
                        metadata_value.get("userKeys").and_then(|v| v.as_array()),
                    ) {
                        let unlock_time_ns = lock_time * 1_000_000_000;

                        // Check if the unlock time has passed
                        if current_time >= unlock_time_ns {
                            // Check if user has access
                            for user_key_obj in user_keys_array {
                                if let Some(user_key_user) =
                                    user_key_obj.get("user").and_then(|v| v.as_str())
                                {
                                    // Check if it's a public chronolock
                                    if user_key_user == "public" {
                                        return true;
                                    }

                                    // Check if user matches directly
                                    if user_key_user == user_text {
                                        return true;
                                    }

                                    // Check for user:unlock_time format (which is used by the frontend)
                                    let user_time_key = format!("{}:{}", user_text, lock_time);
                                    if user_key_user == user_time_key {
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                }
                false
            })
            .count() as u64
    })
}

// Function to get all chronolocks paginated
#[query]
fn get_all_chronolocks_paginated(offset: u64, limit: u64) -> Result<Vec<Chronolock>, ChronoError> {
    let max_limit = 100; // Limit to prevent excessive data transfer
    let actual_limit = std::cmp::min(limit, max_limit);

    Ok(CHRONOLOCKS.with(|locks| {
        locks
            .borrow()
            .iter()
            .skip(offset as usize)
            .take(actual_limit as usize)
            .map(|(_, chronolock)| chronolock.clone())
            .collect()
    }))
}

// Function to get the owner's created chronolocks paginated
#[query]
fn get_owner_chronolocks_paginated(
    owner: Principal,
    offset: u64,
    limit: u64,
) -> Result<Vec<Chronolock>, ChronoError> {
    let max_limit = 100; // Limit to prevent excessive data transfer
    let actual_limit = std::cmp::min(limit, max_limit);

    OWNER_TO_TOKENS.with(|owner_to_tokens| {
        let token_list = owner_to_tokens
            .borrow()
            .get(&owner)
            .map(|list| list.clone())
            .unwrap_or(TokenList { tokens: vec![] });

        let chronolocks: Vec<Chronolock> = token_list
            .tokens
            .into_iter()
            .skip(offset as usize)
            .take(actual_limit as usize)
            .filter_map(|token_id| {
                CHRONOLOCKS.with(|locks| locks.borrow().get(&token_id).map(|lock| lock.clone()))
            })
            .collect();

        Ok(chronolocks)
    })
}

// Function to get chronolocks that can be opened and decrypted by a user
#[query]
fn get_user_accessible_chronolocks_paginated(
    user: Principal,
    offset: u64,
    limit: u64,
) -> Result<Vec<Chronolock>, ChronoError> {
    let max_limit = 100; // Limit to prevent excessive data transfer
    let actual_limit = std::cmp::min(limit, max_limit);
    let current_time = time();
    let user_text = user.to_text();
    let accessible_chronolocks: Vec<Chronolock> = CHRONOLOCKS.with(|locks| {
        locks
            .borrow()
            .iter()
            .filter_map(|(_, chronolock)| {
                if let Some(metadata_value) = decode_metadata_value(&chronolock.metadata) {
                    // Extract lockTime and userKeys from the parsed JSON
                    if let (Some(lock_time), Some(user_keys_array)) = (
                        metadata_value.get("lockTime").and_then(|v| v.as_u64()),
                        metadata_value.get("userKeys").and_then(|v| v.as_array()),
                    ) {
                        let unlock_time_ns = lock_time * 1_000_000_000;

                        // Check if the unlock time has passed
                        if current_time >= unlock_time_ns {
                            // Check if user has access
                            for user_key_obj in user_keys_array {
                                if let Some(user_key_user) =
                                    user_key_obj.get("user").and_then(|v| v.as_str())
                                {
                                    // Check if it's a public chronolock
                                    if user_key_user == "public" {
                                        return Some(chronolock.clone());
                                    }

                                    // Check if user matches directly
                                    if user_key_user == user_text {
                                        return Some(chronolock.clone());
                                    }

                                    // Check for user:unlock_time format (which is used by the frontend)
                                    let user_time_key = format!("{}:{}", user_text, lock_time);
                                    if user_key_user == user_time_key {
                                        return Some(chronolock.clone());
                                    }
                                }
                            }
                        }
                    }
                }
                None
            })
            .skip(offset as usize)
            .take(actual_limit as usize)
            .collect()
    });

    Ok(accessible_chronolocks)
}

// -------------------------
// Authentication Management Functions (Admin Only)
// -------------------------

#[update]
fn add_trusted_principal(principal: Principal) -> Result<(), ChronoError> {
    // Validate admin authentication
    let _authenticated_admin = validate_admin_authentication()?;

    TRUSTED_PRINCIPALS.with(|tp| {
        tp.borrow_mut().insert(principal, true);
    });

    log_activity(format!("Added trusted principal: {}", principal));
    Ok(())
}

#[update]
fn remove_trusted_principal(principal: Principal) -> Result<(), ChronoError> {
    // Validate admin authentication
    let _authenticated_admin = validate_admin_authentication()?;

    TRUSTED_PRINCIPALS.with(|tp| {
        tp.borrow_mut().remove(&principal);
    });

    log_activity(format!("Removed trusted principal: {}", principal));
    Ok(())
}

#[update]
fn set_admin_bypass(enabled: bool) -> Result<(), ChronoError> {
    // Validate admin authentication
    let _authenticated_admin = validate_admin_authentication()?;

    ADMIN_BYPASS_ENABLED.with(|ab| {
        ab.borrow_mut()
            .set(enabled)
            .expect("Failed to set admin bypass");
    });

    log_activity(format!("Admin bypass enabled: {}", enabled));
    Ok(())
}

// -------------------------
// Authentication Query Functions
// -------------------------

#[query]
fn is_caller_authenticated() -> bool {
    validate_caller_authentication().is_ok()
}

#[query]
fn is_principal_trusted(principal: Principal) -> bool {
    TRUSTED_PRINCIPALS.with(|tp| tp.borrow().get(&principal).unwrap_or(false))
}

#[query]
fn is_valid_ii_principal(principal: Principal) -> bool {
    is_valid_internet_identity_principal(principal)
}

#[query]
fn get_trusted_principals() -> Vec<Principal> {
    // Only admin can view the full list of trusted principals
    if validate_admin_authentication().is_err() {
        return vec![];
    }

    TRUSTED_PRINCIPALS.with(|tp| {
        tp.borrow()
            .iter()
            .filter(|(_, trusted)| *trusted)
            .map(|(principal, _)| principal.clone())
            .collect()
    })
}

#[query]
fn is_admin_bypass_enabled() -> bool {
    // Only admin can check this status
    if validate_admin_authentication().is_err() {
        return false;
    }

    ADMIN_BYPASS_ENABLED.with(|ab| ab.borrow().get().clone())
}

#[query]
fn get_caller_principal_info() -> (Principal, bool, bool) {
    let caller = caller();
    let is_authenticated = validate_caller_authentication().is_ok();
    let is_admin_user = is_admin(caller);
    (caller, is_authenticated, is_admin_user)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_internet_identity_principal() {
        let principal =
            Principal::from_text("dmp4o-pkoo3-lnzzj-cystz-2jlkk-v4zcv-yc5h4-iqoeg-v5arm-avsbm-bae")
                .expect("valid principal text");
        TRUSTED_PRINCIPALS.with(|tp| {
            tp.borrow_mut().remove(&principal);
        });
        assert!(is_valid_internet_identity_principal(principal));
    }

    #[test]
    fn rejects_anonymous_principal() {
        assert!(!is_valid_internet_identity_principal(Principal::anonymous()));
    }
}

ic_cdk::export_candid!();
