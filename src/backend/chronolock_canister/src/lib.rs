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
    pub unlock_time: u64,                     // Unix timestamp in seconds
    pub title: Option<String>,                // Optional title for the NFT
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
    pub media_url: Option<String>,             // URL to encrypted media
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

#[update]
fn set_max_metadata_size(new_size: u64) -> Result<(), ChronoError> {
    let caller = caller();
    if !is_admin(caller) {
        return Err(ChronoError::Unauthorized);
    }
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
    let caller = caller();
    if !is_admin(caller) {
        return Err(ChronoError::Unauthorized);
    }
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
    let caller = caller();
    CHRONOLOCKS.with(|locks| {
        OWNER_TO_TOKENS.with(|owner_to_tokens| {
            let mut locks = locks.borrow_mut();
            let mut owner_to_tokens = owner_to_tokens.borrow_mut();
            let lock = locks
                .get(&token_id)
                .ok_or(ChronoError::TokenNotFound)?
                .clone();
            if lock.owner != caller {
                return Err(ChronoError::Unauthorized);
            }
            let mut caller_tokens = owner_to_tokens
                .get(&caller)
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
            owner_to_tokens.insert(caller, caller_tokens);
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
    let caller = caller();
    let authorized_principal = Principal::from_text(&user_id)
        .map_err(|e| ChronoError::InvalidInput(format!("Invalid user id: {}", e)))?;

    if caller != authorized_principal {
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
    let caller = caller();
    let metadata_size = metadata.len() as u64;
    let max_size = MAX_METADATA_SIZE.with(|size| *size.borrow().get());
    if metadata_size > max_size {
        return Err(ChronoError::MetadataTooLarge);
    }
    let id = generate_unique_id();
    let chronolock = Chronolock {
        id: id.clone(),
        owner: caller,
        metadata,
    };
    CHRONOLOCKS.with(|locks| {
        locks.borrow_mut().insert(id.clone(), chronolock);
    });
    OWNER_TO_TOKENS.with(|owner_to_tokens| {
        let mut owner_to_tokens = owner_to_tokens.borrow_mut();
        let mut tokens = owner_to_tokens
            .get(&caller)
            .map(|t| t.clone())
            .unwrap_or(TokenList { tokens: vec![] });
        tokens.tokens.push(id.clone());
        owner_to_tokens.insert(caller, tokens);
    });
    log_activity(format!("Chronolock created with ID: {}", id));
    Ok(id)
}

#[update]
fn update_chronolock(token_id: String, metadata: String) -> Result<(), ChronoError> {
    let caller = caller();
    CHRONOLOCKS.with(|locks| {
        let mut locks = locks.borrow_mut();
        let mut lock = locks
            .get(&token_id)
            .ok_or(ChronoError::TokenNotFound)?
            .clone();
        if lock.owner != caller {
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
    let caller = caller();
    CHRONOLOCKS.with(|locks| {
        OWNER_TO_TOKENS.with(|owner_to_tokens| {
            let mut locks = locks.borrow_mut();
            let mut owner_to_tokens = owner_to_tokens.borrow_mut();
            let lock = locks.get(&token_id).ok_or(ChronoError::TokenNotFound)?;
            if lock.owner != caller {
                return Err(ChronoError::Unauthorized);
            }
            locks.remove(&token_id);
            if let Some(caller_tokens) = owner_to_tokens.get(&caller).map(|t| t.clone()) {
                let mut caller_tokens = caller_tokens;
                caller_tokens.tokens.retain(|id| id != &token_id);
                owner_to_tokens.insert(caller, caller_tokens);
            }
            log_activity(format!("Burned chronolock {}", token_id));
            Ok(())
        })
    })
}

#[update]
fn start_media_upload(total_chunks: u32) -> String {
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
    media_id
}

#[update]
fn upload_media_chunk(
    media_id: String,
    chunk_index: u32,
    chunk: Vec<u8>,
) -> Result<u32, ChronoError> {
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
    OWNER_TO_TOKENS.with(|owner_to_tokens| {
        owner_to_tokens.borrow().len() as u64
    })
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
                // Parse the base64-encoded metadata
                if let Ok(metadata_bytes) = general_purpose::STANDARD.decode(&chronolock.metadata) {
                    if let Ok(metadata_str) = String::from_utf8(metadata_bytes) {
                        if let Ok(metadata_value) =
                            serde_json::from_str::<serde_json::Value>(&metadata_str)
                        {
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
                                            let user_time_key =
                                                format!("{}:{}", user_text, lock_time);
                                            if user_key_user == user_time_key {
                                                return true;
                                            }
                                        }
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
                // Parse the base64-encoded metadata
                if let Ok(metadata_bytes) = general_purpose::STANDARD.decode(&chronolock.metadata) {
                    if let Ok(metadata_str) = String::from_utf8(metadata_bytes) {
                        if let Ok(metadata_value) =
                            serde_json::from_str::<serde_json::Value>(&metadata_str)
                        {
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
                                            let user_time_key =
                                                format!("{}:{}", user_text, lock_time);
                                            if user_key_user == user_time_key {
                                                return Some(chronolock.clone());
                                            }
                                        }
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

ic_cdk::export_candid!();
