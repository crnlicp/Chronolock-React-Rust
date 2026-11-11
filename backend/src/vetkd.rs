//! Time-lock keys via vetKD identity-based encryption (scheme v2).
//!
//! Each lock's random AES-256-GCM key is IBE-encrypted in the browser to an
//! identity that binds the unlock time and, for private locks, the recipient:
//!
//! ```text
//! identity = "CL" || 0x02 || kind || unlock_time (u64, big endian) [|| len || principal]
//!   kind 0 = public (anyone may open after unlock_time)
//!   kind 1 = recipient (only that principal may open after unlock_time)
//! ```
//!
//! After `unlock_time` the canister derives the matching vetKey, encrypted to a
//! fresh transport key from the caller, and only for the caller's own identity.
//! Content is never decrypted on-chain.

use crate::auth::{authenticated_caller, check_rate_limit};
use crate::state::config;
use crate::types::{ChronoError, ChronoResult};
use candid::Principal;
use ic_cdk_management_canister::{
    vetkd_derive_key, vetkd_public_key, VetKDCurve, VetKDDeriveKeyArgs, VetKDKeyId,
    VetKDPublicKeyArgs,
};

pub const SCHEME_VERSION: u8 = 2;
/// Domain separator for all chronolock keys; changing it changes every key.
pub const CONTEXT: &[u8] = b"chronolock/v2";
const TRANSPORT_KEY_LEN: usize = 48;
const KIND_PUBLIC: u8 = 0;
const KIND_RECIPIENT: u8 = 1;

/// The IBE identity bytes. Must match `ibeIdentity` in frontend/src/lib/crypto.ts.
pub fn identity(unlock_time: u64, recipient: Option<Principal>) -> Vec<u8> {
    let mut out = Vec::with_capacity(48);
    out.extend_from_slice(b"CL");
    out.push(SCHEME_VERSION);
    match recipient {
        None => {
            out.push(KIND_PUBLIC);
            out.extend_from_slice(&unlock_time.to_be_bytes());
        }
        Some(p) => {
            out.push(KIND_RECIPIENT);
            out.extend_from_slice(&unlock_time.to_be_bytes());
            let bytes = p.as_slice();
            out.push(bytes.len() as u8);
            out.extend_from_slice(bytes);
        }
    }
    out
}

fn key_id() -> VetKDKeyId {
    VetKDKeyId {
        curve: VetKDCurve::Bls12_381_G2,
        name: config().vetkd_key_name,
    }
}

/// The canister's IBE public key for `CONTEXT`. On mainnet the frontend
/// derives the same key offline from the master key and does not trust this.
pub async fn public_key() -> ChronoResult<Vec<u8>> {
    let res = vetkd_public_key(&VetKDPublicKeyArgs {
        canister_id: None,
        context: CONTEXT.to_vec(),
        key_id: key_id(),
    })
    .await
    .map_err(|e| ChronoError::Internal(format!("vetkd_public_key failed: {e}")))?;
    Ok(res.public_key)
}

fn check_unlocked(unlock_time: u64) -> ChronoResult<()> {
    let now_s = ic_cdk::api::time() / 1_000_000_000;
    if now_s < unlock_time {
        return Err(ChronoError::TimeLocked);
    }
    Ok(())
}

fn check_transport_key(key: &[u8]) -> ChronoResult<()> {
    if key.len() != TRANSPORT_KEY_LEN {
        return Err(ChronoError::InvalidInput(
            "transport public key must be 48 bytes".to_string(),
        ));
    }
    Ok(())
}

async fn derive(
    unlock_time: u64,
    recipient: Option<Principal>,
    transport_public_key: Vec<u8>,
) -> ChronoResult<Vec<u8>> {
    let caller = authenticated_caller()?;
    check_transport_key(&transport_public_key)?;
    check_unlocked(unlock_time)?;
    check_rate_limit(caller, config().limits.key_derivations_per_hour)?;
    let res = vetkd_derive_key(&VetKDDeriveKeyArgs {
        input: identity(unlock_time, recipient),
        context: CONTEXT.to_vec(),
        transport_public_key,
        key_id: key_id(),
    })
    .await
    .map_err(|e| ChronoError::Internal(format!("vetkd_derive_key failed: {e}")))?;
    Ok(res.encrypted_key)
}

/// Key for public locks. Requires sign-in so key derivations (which cost
/// cycles) can be rate-limited per caller.
pub async fn public_time_key(
    unlock_time: u64,
    transport_public_key: Vec<u8>,
) -> ChronoResult<Vec<u8>> {
    derive(unlock_time, None, transport_public_key).await
}

/// Key for locks addressed to the caller.
pub async fn user_time_key(
    unlock_time: u64,
    transport_public_key: Vec<u8>,
) -> ChronoResult<Vec<u8>> {
    let caller = ic_cdk::api::msg_caller();
    derive(unlock_time, Some(caller), transport_public_key).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_layout() {
        assert_eq!(
            identity(1, None),
            vec![b'C', b'L', 2, 0, 0, 0, 0, 0, 0, 0, 0, 1]
        );
        let p = Principal::from_slice(&[7, 8, 9]);
        assert_eq!(
            identity(256, Some(p)),
            vec![b'C', b'L', 2, 1, 0, 0, 0, 0, 0, 0, 1, 0, 3, 7, 8, 9]
        );
    }
}
