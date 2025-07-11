// src/backend/chronolock/tests/canister_tests.rs

use candid::{decode_one, encode_args, CandidType, Principal};
use ic_vetkd_utils::TransportSecretKey;
use pocket_ic::PocketIc;
use serde::Deserialize;
use serde_json;
use std::fs;
use std::time::UNIX_EPOCH;

// Path to compiled WASM file (adjust as needed)
const BACKEND_WASM: &str =
    "../../../target/wasm32-unknown-unknown/release/chronolock_canister.wasm";
const VETKD_WASM: &str =
    "../../../target/wasm32-unknown-unknown/release/chainkey_testing_canister.wasm";

// Structures required for testing (must match canister definitions)
#[derive(CandidType, Deserialize, Clone, Debug)]
struct Chronolock {
    id: String,
    owner: Principal,
    metadata: String,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct LogEntry {
    id: String,
    timestamp: u64,
    activity: String,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct TokenList {
    tokens: Vec<String>,
}

#[derive(CandidType, Deserialize, Debug, PartialEq)]
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

#[derive(CandidType, Deserialize)]
struct HttpResponse {
    status_code: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

#[derive(CandidType, Deserialize)]
struct VetKDPublicKeyReply {
    public_key: Vec<u8>,
}

#[derive(CandidType, Deserialize, Debug, PartialEq)]
pub struct VetKDDeriveKeyReply {
    pub encrypted_key: Vec<u8>,
}

// Setup function
fn setup() -> (PocketIc, Principal, Principal, Principal) {
    std::env::set_var("POCKET_IC_BIN", "/usr/local/bin/pocket-ic");
    let pic = PocketIc::new();

    // Deploy VETKD mock canister
    let vetkd_canister = pic.create_canister();
    pic.add_cycles(vetkd_canister, 2_000_000_000_000);
    let vetkd_wasm = fs::read(VETKD_WASM).expect("VETKD WASM file not found");
    pic.install_canister(vetkd_canister, vetkd_wasm, vec![], None);

    // Deploy Chronolock canister
    let backend_canister = pic.create_canister();
    pic.add_cycles(backend_canister, 2_000_000_000_000);
    let wasm = fs::read(BACKEND_WASM).expect("Wasm file not found, run 'cargo build'.");
    let admin = Principal::from_text("aaaaa-aa").unwrap();
    let init_args =
        encode_args((admin, Some(vetkd_canister))).expect("Failed to encode init arguments");
    pic.install_canister(backend_canister, wasm, init_args, None);

    (pic, backend_canister, vetkd_canister, admin)
}

fn decode_with_fallback<T: CandidType + for<'de> Deserialize<'de>>(
    bytes: &[u8],
    method: &str,
) -> Result<T, String> {
    decode_one(bytes).map_err(|e| {
        let raw_hex = bytes
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        format!(
            "Decoding failed for {}: {:?}. Raw response: {}",
            method, e, raw_hex
        )
    })
}

// Initialization Test
#[test]
fn test_initialization() {
    let (pic, backend_canister, vetkd_canister, admin) = setup();

    let response = pic
        .query_call(
            backend_canister,
            admin,
            "get_logs_paginated",
            encode_args((0u64, 10u64)).unwrap(),
        )
        .expect("Failed to query get_logs_paginated");
    let logs_result: Result<Vec<LogEntry>, ChronoError> = decode_one(&response).unwrap();
    let logs = logs_result.expect("Failed to get logs");
    assert_eq!(logs.len(), 1);
    let expected_log = format!("Canister initialized with {} and {}", admin, vetkd_canister);
    assert_eq!(logs[0].activity, expected_log);
}

// Admin Function Tests
#[test]
fn test_set_max_metadata_size() {
    let (pic, backend_canister, _, admin) = setup();

    let response = pic
        .update_call(
            backend_canister,
            admin,
            "set_max_metadata_size",
            encode_args((2048u64,)).unwrap(),
        )
        .expect("Failed to call set_max_metadata_size");

    let result: Result<(), ChronoError> = decode_with_fallback(&response, "set_max_metadata_size")
        .unwrap_or_else(|e| {
            println!("Decoding error: {}", e);
            Err(ChronoError::InternalError(e))
        });
    assert!(result.is_ok(), "Expected Ok(()), got {:?}", result);
}

// Log Management Test
#[test]
fn test_get_logs_paginated() {
    let (pic, backend_canister, _, admin) = setup();

    let response = pic
        .query_call(
            backend_canister,
            admin,
            "get_logs_paginated",
            encode_args((0u64, 10u64)).unwrap(),
        )
        .expect("Failed to query get_logs_paginated");

    let logs_result: Result<Vec<LogEntry>, ChronoError> =
        decode_with_fallback(&response, "get_logs_paginated").unwrap_or_else(|e| {
            println!("Decoding error: {}", e);
            Err(ChronoError::InternalError(e))
        });
    let logs = logs_result.expect("Failed to get logs");
    assert_eq!(logs.len(), 1, "Expected 1 log entry, got {}", logs.len());
}

// ICRC-7 Query Tests
#[test]
fn test_icrc7_symbol() {
    let (pic, backend_canister, _, _) = setup();

    let response = pic
        .query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc7_symbol",
            encode_args(()).unwrap(),
        )
        .expect("Failed to query icrc7_symbol");
    let result: String = decode_one(&response).unwrap();
    assert_eq!(result, "CHRONOLOCK");
}

#[test]
fn test_icrc7_name() {
    let (pic, backend_canister, _, _) = setup();

    let response = pic
        .query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc7_name",
            encode_args(()).unwrap(),
        )
        .expect("Failed to query icrc7_name");
    let result: String = decode_one(&response).unwrap();
    assert_eq!(result, "Chronolock Collection");
}

#[test]
fn test_icrc7_description() {
    let (pic, backend_canister, _, _) = setup();

    let response = pic
        .query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc7_description",
            encode_args(()).unwrap(),
        )
        .expect("Failed to query icrc7_description");
    let result: String = decode_one(&response).unwrap();
    assert_eq!(result, "A collection of time-locked NFTs");
}

#[test]
fn test_icrc7_total_supply() {
    let (pic, backend_canister, _, _) = setup();

    let response = pic
        .query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc7_total_supply",
            encode_args(()).unwrap(),
        )
        .expect("Failed to query icrc7_total_supply");
    let result: u64 = decode_one(&response).unwrap();
    assert_eq!(result, 0);
}

#[test]
fn test_icrc7_balance_of() {
    let (pic, backend_canister, _, admin) = setup();

    let response = pic
        .query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc7_balance_of",
            encode_args((admin,)).unwrap(),
        )
        .expect("Failed to query icrc7_balance_of");
    let result: u64 = decode_one(&response).unwrap();
    assert_eq!(result, 0);
}

#[test]
fn test_icrc7_owner_of_and_metadata() {
    let (pic, backend_canister, _, admin) = setup();

    let unlock_time = pic.get_time().duration_since(UNIX_EPOCH).unwrap().as_secs() + 3600;
    let metadata = serde_json::json!({
        "unlock_time": unlock_time,
        "title": "Test NFT",
        "encrypted_metadata": "hex_or_base64_string"
    })
    .to_string();
    let create_response = pic
        .update_call(
            backend_canister,
            admin,
            "create_chronolock",
            encode_args((metadata.clone(),)).unwrap(),
        )
        .expect("Failed to call create_chronolock");
    let token_id_result: Result<String, ChronoError> = decode_one(&create_response).unwrap();
    let token_id = token_id_result.expect("Failed to create chronolock");

    let owner_response = pic
        .query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc7_owner_of",
            encode_args((token_id.clone(),)).unwrap(),
        )
        .expect("Failed to query icrc7_owner_of");
    let owner: Option<Principal> = decode_one(&owner_response).unwrap();
    assert_eq!(owner, Some(admin));

    let metadata_response = pic
        .query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc7_token_metadata",
            encode_args((token_id.clone(),)).unwrap(),
        )
        .expect("Failed to query icrc7_token_metadata");
    let received_metadata: Option<String> = decode_one(&metadata_response).unwrap();
    assert_eq!(received_metadata, Some(metadata));
}

#[test]
fn test_icrc7_transfer() {
    let (pic, backend_canister, _, admin) = setup();

    // Step 1: Create a chronolock token
    let unlock_time = pic.get_time().duration_since(UNIX_EPOCH).unwrap().as_secs() + 3600;
    let metadata = serde_json::json!({
        "unlock_time": unlock_time,
        "title": "Test NFT",
        "encrypted_metadata": "hex_or_base64_string"
    })
    .to_string();
    let create_response = pic
        .update_call(
            backend_canister,
            admin,
            "create_chronolock",
            encode_args((metadata.clone(),)).unwrap(),
        )
        .expect("Failed to call create_chronolock");
    let token_id_result: Result<String, ChronoError> = decode_one(&create_response).unwrap();
    let token_id = token_id_result.expect("Failed to create chronolock");

    // Step 2: Generate a valid recipient Principal
    let recipient = Principal::self_authenticating(&[1, 2, 3]); // Generates a valid test principal

    // Step 3: Transfer token from admin to recipient
    let transfer_response = pic
        .update_call(
            backend_canister,
            admin,
            "icrc7_transfer",
            encode_args((token_id.clone(), recipient)).unwrap(),
        )
        .expect("Failed to call icrc7_transfer");

    let transfer_result: Result<(), ChronoError> = decode_one(&transfer_response).unwrap();
    assert!(
        transfer_result.is_ok(),
        "Transfer failed: {:?}",
        transfer_result
    );

    // Step 4: Verify the new owner
    let owner_response = pic
        .query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc7_owner_of",
            encode_args((token_id.clone(),)).unwrap(),
        )
        .expect("Failed to query icrc7_owner_of");

    let owner: Option<Principal> = decode_one(&owner_response).unwrap();
    assert_eq!(owner, Some(recipient), "Ownership transfer failed");
}

#[test]
fn test_icrc7_transfer_no_op() {
    let (pic, backend_canister, _, admin) = setup();

    let unlock_time = pic.get_time().duration_since(UNIX_EPOCH).unwrap().as_secs() + 3600;
    let metadata = serde_json::json!({
        "unlock_time": unlock_time,
        "title": "Test NFT",
        "encrypted_metadata": "hex_or_base64_string"
    })
    .to_string();
    let create_response = pic
        .update_call(
            backend_canister,
            admin,
            "create_chronolock",
            encode_args((metadata.clone(),)).unwrap(),
        )
        .expect("Failed to call create_chronolock");
    let token_id_result: Result<String, ChronoError> =
        decode_one::<Result<String, ChronoError>>(&create_response).unwrap();
    let token_id = token_id_result.expect("Failed to create chronolock");

    let transfer_response = pic
        .update_call(
            backend_canister,
            admin,
            "icrc7_transfer",
            encode_args((token_id.clone(), admin)).unwrap(),
        )
        .expect("Failed to call icrc7_transfer");

    let transfer_result: Result<(), ChronoError> = decode_one(&transfer_response).unwrap();
    assert!(
        transfer_result.is_ok(),
        "Transfer to self should be allowed"
    );

    let owner_response = pic
        .query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc7_owner_of",
            encode_args((token_id.clone(),)).unwrap(),
        )
        .expect("Failed to query icrc7_owner_of");
    let owner: Option<Principal> = decode_one(&owner_response).unwrap();
    assert_eq!(owner, Some(admin), "Owner should remain the same");
}

// Chronolock Management Tests
#[test]
fn test_create_update_burn_chronolock() {
    let (pic, backend_canister, _, admin) = setup();

    let unlock_time = pic.get_time().duration_since(UNIX_EPOCH).unwrap().as_secs() + 3600;
    let metadata = serde_json::json!({
        "unlock_time": unlock_time,
        "title": "Test NFT",
        "encrypted_metadata": "hex_or_base64_string"
    })
    .to_string();
    let create_response = pic
        .update_call(
            backend_canister,
            admin,
            "create_chronolock",
            encode_args((metadata.clone(),)).unwrap(),
        )
        .expect("Failed to call create_chronolock");
    let token_id_result: Result<String, ChronoError> = decode_one(&create_response).unwrap();
    let token_id = token_id_result.expect("Failed to create chronolock");

    let new_unlock_time = pic.get_time().duration_since(UNIX_EPOCH).unwrap().as_secs() + 7200;
    let new_metadata = serde_json::json!({
        "unlock_time": new_unlock_time,
        "title": "Test NFT",
        "encrypted_metadata": "hex_or_base64_string"
    })
    .to_string();
    let update_response = pic
        .update_call(
            backend_canister,
            admin,
            "update_chronolock",
            encode_args((token_id.clone(), new_metadata.clone())).unwrap(),
        )
        .expect("Failed to call update_chronolock");
    let update_result: Result<(), ChronoError> = decode_one(&update_response).unwrap();
    assert!(update_result.is_ok(), "Update failed: {:?}", update_result);

    let metadata_response = pic
        .query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc7_token_metadata",
            encode_args((token_id.clone(),)).unwrap(),
        )
        .expect("Failed to query icrc7_token_metadata");
    let received_metadata: Option<String> = decode_one(&metadata_response).unwrap();
    assert_eq!(received_metadata, Some(new_metadata));

    let burn_response = pic
        .update_call(
            backend_canister,
            admin,
            "burn_chronolock",
            encode_args((token_id.clone(),)).unwrap(),
        )
        .expect("Failed to call burn_chronolock");
    let burn_result: Result<(), ChronoError> = decode_one(&burn_response).unwrap();
    assert!(burn_result.is_ok(), "Burn failed: {:?}", burn_result);

    let owner_response = pic
        .query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc7_owner_of",
            encode_args((token_id.clone(),)).unwrap(),
        )
        .expect("Failed to query icrc7_owner_of");
    let owner: Option<Principal> = decode_one(&owner_response).unwrap();
    assert_eq!(owner, None);
}

#[test]
fn test_upload_and_get_media() {
    let (pic, backend_canister, _, admin) = setup();

    let file_data = vec![
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
    ];
    let total_chunks = 2;
    let chunk1 = file_data[..10].to_vec();
    let chunk2 = file_data[10..].to_vec();

    // Start upload
    let start_response = pic
        .update_call(
            backend_canister,
            admin,
            "start_media_upload",
            encode_args((total_chunks as u32,)).unwrap(),
        )
        .expect("Failed to call start_media_upload");
    let media_id: String = decode_one(&start_response).unwrap();

    // Upload chunks
    let _ = pic
        .update_call(
            backend_canister,
            admin,
            "upload_media_chunk",
            encode_args((media_id.clone(), 0u32, chunk1.clone())).unwrap(),
        )
        .expect("Failed to call upload_media_chunk 0");
    let _ = pic
        .update_call(
            backend_canister,
            admin,
            "upload_media_chunk",
            encode_args((media_id.clone(), 1u32, chunk2.clone())).unwrap(),
        )
        .expect("Failed to call upload_media_chunk 1");

    // Finish upload
    let finish_response = pic
        .update_call(
            backend_canister,
            admin,
            "finish_media_upload",
            encode_args((media_id.clone(),)).unwrap(),
        )
        .expect("Failed to call finish_media_upload");
    let media_url_result: Result<String, ChronoError> = decode_one(&finish_response).unwrap();
    let media_url = media_url_result.expect("Failed to upload media");
    let media_id_from_url = media_url.split('/').last().unwrap().to_string();

    let get_response = pic
        .query_call(
            backend_canister,
            Principal::anonymous(),
            "get_media",
            encode_args((media_id_from_url.clone(),)).unwrap(),
        )
        .expect("Failed to query get_media");
    let retrieved_data_result: Result<Vec<u8>, ChronoError> = decode_one(&get_response).unwrap();
    let retrieved_data = retrieved_data_result.expect("Failed to get media");
    assert_eq!(retrieved_data, file_data);

    let http_request = HttpRequest {
        method: "GET".to_string(),
        url: format!("/media/{}", media_id_from_url),
        headers: vec![],
        body: vec![],
    };
    let http_response = pic
        .query_call(
            backend_canister,
            Principal::anonymous(),
            "http_request",
            encode_args((http_request,)).unwrap(),
        )
        .expect("Failed to query http_request");
    let response: HttpResponse = decode_one(&http_response).unwrap();
    assert_eq!(response.status_code, 200);
    assert_eq!(response.body, file_data);
}

// VETKD Tests cases

#[test]
fn test_ibe_encryption_key() {
    let (pic, backend_canister, _, admin) = setup();

    let response = pic
        .update_call(
            backend_canister,
            admin,
            "ibe_encryption_key",
            encode_args(()).unwrap(),
        )
        .expect("Failed to call ibe_encryption_key");
    let key_result: Result<VetKDPublicKeyReply, ChronoError> = decode_one(&response).unwrap();
    let key = key_result.expect("Failed to get encryption key");
    assert!(!key.public_key.is_empty(), "Expected non-empty public key");
    println!("VETKD Public Key: {:?}", key.public_key);
    println!("Key length: {:?}", key.public_key.len());
}

#[test]
fn test_get_time_decryption_key_time_lock() {
    let (pic, backend_canister, _, admin) = setup();

    let current_time = pic.get_time().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let unlock_time = current_time + 1000; // 1000 seconds from now
    let unlock_time_hex = format!("{:016x}", unlock_time);

    let tsk = TransportSecretKey::from_seed([42u8; 32].to_vec()).unwrap();
    let encryption_public_key = tsk.public_key();

    let encoded = encode_args((unlock_time_hex.clone(), encryption_public_key.clone())).unwrap();
    println!("Encoded args: {}", hex::encode(&encoded));

    // Before unlock time
    let response = pic
        .update_call(
            backend_canister,
            admin,
            "get_time_decryption_key",
            encode_args((unlock_time_hex.clone(), encryption_public_key.clone())).unwrap(),
        )
        .expect("Failed to call get_time_decryption_key");
    let result: Result<VetKDDeriveKeyReply, ChronoError> = decode_one(&response).unwrap();
    assert_eq!(
        result,
        Err(ChronoError::TimeLocked),
        "Expected TimeLocked error before unlock time"
    );

    // Advance time by 1001 seconds
    pic.advance_time(std::time::Duration::from_secs(1001));

    // After unlock time
    let response = pic
        .update_call(
            backend_canister,
            admin,
            "get_time_decryption_key",
            encode_args((unlock_time_hex.clone(), encryption_public_key.clone())).unwrap(),
        )
        .expect("Failed to call get_time_decryption_key");
    let result: Result<VetKDDeriveKeyReply, ChronoError> = decode_one(&response).unwrap();
    let key = result.expect("Failed to get decryption key");

    assert!(
        !key.encrypted_key.is_empty(),
        "Expected non-empty encrypted key"
    );
    assert_eq!(
        key.encrypted_key.len(),
        192,
        "Expected encrypted key to be 48 bytes"
    );
}

#[test]
fn test_get_user_time_decryption_key_auth_and_time() {
    let (pic, backend_canister, _, admin) = setup();

    let current_time = pic.get_time().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let unlock_time = current_time + 1000; // 1000 seconds from now
    let unlock_time_hex = format!("{:016x}", unlock_time);
    let user_id = admin.to_text();

    let tsk = TransportSecretKey::from_seed([42u8; 32].to_vec()).unwrap();
    let encryption_public_key = tsk.public_key();

    let encoded = encode_args((
        unlock_time_hex.clone(),
        user_id.clone(),
        encryption_public_key.clone(),
    ))
    .unwrap();
    println!("Encoded args: {}", hex::encode(&encoded));

    // Test unauthorized caller
    let unauthorized_caller = Principal::self_authenticating(&[1, 2, 3]);
    let response = pic
        .update_call(
            backend_canister,
            unauthorized_caller,
            "get_user_time_decryption_key",
            encode_args((
                unlock_time_hex.clone(),
                user_id.clone(),
                encryption_public_key.clone(),
            ))
            .unwrap(),
        )
        .expect("Failed to call get_user_time_decryption_key");
    let result: Result<VetKDDeriveKeyReply, ChronoError> = decode_one(&response).unwrap();
    assert_eq!(
        result,
        Err(ChronoError::Unauthorized),
        "Expected Unauthorized error for wrong caller"
    );

    // Test authorized caller before unlock time
    let response = pic
        .update_call(
            backend_canister,
            admin,
            "get_user_time_decryption_key",
            encode_args((
                unlock_time_hex.clone(),
                user_id.clone(),
                encryption_public_key.clone(),
            ))
            .unwrap(),
        )
        .expect("Failed to call get_user_time_decryption_key");
    let result: Result<VetKDDeriveKeyReply, ChronoError> = decode_one(&response).unwrap();
    assert_eq!(
        result,
        Err(ChronoError::TimeLocked),
        "Expected TimeLocked error before unlock time"
    );

    // Advance time by 1001 seconds
    pic.advance_time(std::time::Duration::from_secs(1001));

    // Test authorized caller after unlock time
    let response = pic
        .update_call(
            backend_canister,
            admin,
            "get_user_time_decryption_key",
            encode_args((
                unlock_time_hex.clone(),
                user_id.clone(),
                encryption_public_key.clone(),
            ))
            .unwrap(),
        )
        .expect("Failed to call get_user_time_decryption_key");
    let result: Result<VetKDDeriveKeyReply, ChronoError> = decode_one(&response).unwrap();
    let key = result.expect("Failed to get decryption key");

    assert!(
        !key.encrypted_key.is_empty(),
        "Expected non-empty encrypted key"
    );
    assert_eq!(
        key.encrypted_key.len(),
        192,
        "Expected encrypted key to be 48 bytes"
    );
}

#[test]
fn test_encryption_decryption_invalid_inputs() {
    let (pic, backend_canister, _, admin) = setup();

    // Invalid unlock_time_hex
    let invalid_unlock_time_hex = "invalid_hex".to_string();

    let tsk = TransportSecretKey::from_seed([42u8; 32].to_vec()).unwrap();
    let encryption_public_key = tsk.public_key();

    println!("Encoded args: {}", hex::encode(&encryption_public_key));
    let response = pic
        .update_call(
            backend_canister,
            admin,
            "get_time_decryption_key",
            encode_args((
                invalid_unlock_time_hex.clone(),
                encryption_public_key.clone(),
            ))
            .unwrap(),
        )
        .expect("Failed to call get_time_decryption_key");
    let result: Result<VetKDDeriveKeyReply, ChronoError> = decode_one(&response).unwrap();
    assert!(
        matches!(result, Err(ChronoError::InvalidInput(_))),
        "Expected InvalidInput for invalid unlock_time_hex"
    );

    let current_time = pic.get_time().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let unlock_time = current_time + 1000; // 1000 seconds from now
    let unlock_time_hex = format!("{:016x}", unlock_time);

    println!(
        "Encoded args: {},{}",
        hex::encode(&unlock_time_hex.clone()),
        hex::encode(encryption_public_key.clone())
    );

    // Before unlock time
    let response = pic
        .update_call(
            backend_canister,
            admin,
            "get_time_decryption_key",
            encode_args((unlock_time_hex.clone(), encryption_public_key.clone())).unwrap(),
        )
        .expect("Failed to call get_time_decryption_key");
    let result: Result<VetKDDeriveKeyReply, ChronoError> = decode_one(&response).unwrap();
    assert_eq!(
        result,
        Err(ChronoError::TimeLocked),
        "Expected TimeLocked error before unlock time"
    );

    // Empty encryption_public_key
    let unlock_time_hex = "00000000000003e8".to_string(); // 1000 in hex
    let empty_key: Vec<u8> = vec![];
    let response = pic
        .update_call(
            backend_canister,
            admin,
            "get_time_decryption_key",
            encode_args((unlock_time_hex.clone(), empty_key.clone())).unwrap(),
        )
        .expect("Failed to call get_time_decryption_key");
    let result: Result<VetKDDeriveKeyReply, ChronoError> = decode_one(&response).unwrap();
    assert_eq!(
        result,
        Err(ChronoError::InvalidInput(
            "Encryption public key cannot be empty".to_string()
        )),
        "Expected InvalidInput for empty encryption key"
    );

    // Invalid user_id
    let invalid_user_id = "not_a_principal".to_string();
    let response = pic
        .update_call(
            backend_canister,
            admin,
            "get_user_time_decryption_key",
            encode_args((
                unlock_time_hex.clone(),
                invalid_user_id,
                encryption_public_key.clone(),
            ))
            .unwrap(),
        )
        .expect("Failed to call get_user_time_decryption_key");
    let result: Result<VetKDDeriveKeyReply, ChronoError> = decode_one(&response).unwrap();
    assert!(
        matches!(result, Err(ChronoError::InvalidInput(_))),
        "Expected InvalidInput for invalid user_id"
    );
}

#[test]
fn test_chronolock_encryption_integration() {
    let (pic, backend_canister, _, admin) = setup();

    let current_time = pic.get_time().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let unlock_time = current_time + 1000; // 1000 seconds from now
    let unlock_time_hex = format!("{:016x}", unlock_time);

    let tsk = TransportSecretKey::from_seed([42u8; 32].to_vec()).unwrap();
    let encryption_public_key = tsk.public_key();

    let encoded = encode_args((unlock_time_hex.clone(), encryption_public_key.clone())).unwrap();
    println!("Encoded args: {}", hex::encode(&encoded));
    let metadata = serde_json::json!({
        "unlock_time": unlock_time,
        "title": "Test NFT",
        "encrypted_metadata": "hex_or_base64_string"
    })
    .to_string();
    // Create chronolock
    let create_response = pic
        .update_call(
            backend_canister,
            admin,
            "create_chronolock",
            encode_args((metadata.clone(),)).unwrap(),
        )
        .expect("Failed to call create_chronolock");
    let token_id_result: Result<String, ChronoError> = decode_one(&create_response).unwrap();
    token_id_result.expect("Failed to create chronolock");

    // Try to get decryption key before unlock time
    let response = pic
        .update_call(
            backend_canister,
            admin,
            "get_time_decryption_key",
            encode_args((unlock_time_hex.clone(), encryption_public_key.clone())).unwrap(),
        )
        .expect("Failed to call get_time_decryption_key");
    let result: Result<VetKDDeriveKeyReply, ChronoError> = decode_one(&response).unwrap();
    assert_eq!(
        result,
        Err(ChronoError::TimeLocked),
        "Expected TimeLocked before unlock time"
    );

    // Advance time by 1001 seconds to surpass unlock_time
    pic.advance_time(std::time::Duration::from_secs(1001));

    // Get decryption key after unlock time
    let response = pic
        .update_call(
            backend_canister,
            admin,
            "get_time_decryption_key",
            encode_args((unlock_time_hex.clone(), encryption_public_key.clone())).unwrap(),
        )
        .expect("Failed to call get_time_decryption_key");
    let result: Result<VetKDDeriveKeyReply, ChronoError> = decode_one(&response).unwrap();
    let key = result.expect("Failed to get decryption key");

    assert!(
        !key.encrypted_key.is_empty(),
        "Expected non-empty encrypted key"
    );
    assert_eq!(
        key.encrypted_key.len(),
        192,
        "Expected encrypted key to be 48 bytes"
    );
}

#[test]
fn test_multi_user_time_locked_decryption_keys() {
    let (pic, backend_canister, _, admin) = setup();

    // Setup unlock time and encryption public key
    let current_time = pic.get_time().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let unlock_time = current_time + 1000;
    let unlock_time_hex = format!("{:016x}", unlock_time);

    let tsk = TransportSecretKey::from_seed([42u8; 32].to_vec()).unwrap();
    let encryption_public_key = tsk.public_key();

    // Define multiple users
    let user1 = admin;
    let user2 = Principal::self_authenticating(&[4, 5, 6]);
    let user3 = Principal::self_authenticating(&[7, 8, 9]);
    let users = vec![user1, user2, user3];

    // Before unlock time, all users should get TimeLocked error
    for user in &users {
        let user_id = user.to_text();
        let response = pic
            .update_call(
                backend_canister,
                *user,
                "get_user_time_decryption_key",
                encode_args((
                    unlock_time_hex.clone(),
                    user_id.clone(),
                    encryption_public_key.clone(),
                ))
                .unwrap(),
            )
            .expect("Failed to call get_user_time_decryption_key");
        let result: Result<VetKDDeriveKeyReply, ChronoError> = decode_one(&response).unwrap();
        assert_eq!(
            result,
            Err(ChronoError::TimeLocked),
            "Expected TimeLocked error before unlock time for user {}",
            user_id
        );
    }

    // Advance time past unlock_time
    pic.advance_time(std::time::Duration::from_secs(1001));

    // After unlock time, each user should get their own decryption key
    for user in &users {
        let user_id = user.to_text();
        let response = pic
            .update_call(
                backend_canister,
                *user,
                "get_user_time_decryption_key",
                encode_args((
                    unlock_time_hex.clone(),
                    user_id.clone(),
                    encryption_public_key.clone(),
                ))
                .unwrap(),
            )
            .expect("Failed to call get_user_time_decryption_key");
        let result: Result<VetKDDeriveKeyReply, ChronoError> = decode_one(&response).unwrap();
        let key = result.expect("Failed to get decryption key");

        assert!(
            !key.encrypted_key.is_empty(),
            "Expected non-empty encrypted key"
        );
        assert_eq!(
            key.encrypted_key.len(),
            192,
            "Expected encrypted key to be 48 bytes"
        );
    }
}

#[test]
fn test_create_and_unlock_multi_user_chronolock() {
    let (pic, backend_canister, _, admin) = setup();

    // Setup unlock time and encryption public key
    let current_time = pic.get_time().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let unlock_time = current_time + 1000;
    let unlock_time_hex = format!("{:016x}", unlock_time);

    let tsk = TransportSecretKey::from_seed([42u8; 32].to_vec()).unwrap();
    let encryption_public_key = tsk.public_key();

    // Define multiple users
    let user1 = admin;
    let user2 = Principal::self_authenticating(&[10, 11, 12]);
    let user3 = Principal::self_authenticating(&[13, 14, 15]);
    let users = vec![user1, user2, user3];

    // Simulate encrypting a symmetric key for each user (in practice, this would be done off-chain)
    let mut encrypted_keys = vec![];
    for user in &users {
        let user_id = user.to_text();
        // In a real scenario, you would encrypt the symmetric key with the IBE key for (unlock_time_hex, user_id)
        // Here, we just store the tuple for test purposes
        encrypted_keys.push((user_id.clone(), format!("encrypted_key_for_{}", user_id)));
    }

    // Store the encrypted keys as JSON in metadata (simulate multi-user metadata)
    let metadata = serde_json::json!({
        "unlock_time": unlock_time,
        "title": "Multi-User Chronolock",
        "user_keys": encrypted_keys,
        "encrypted_metadata": "hex_or_base64_string"
    });

    // Create the chronolock
    let create_response = pic
        .update_call(
            backend_canister,
            admin,
            "create_chronolock",
            encode_args((metadata.to_string(),)).unwrap(),
        )
        .expect("Failed to call create_chronolock");
    let token_id_result: Result<String, ChronoError> = decode_one(&create_response).unwrap();
    let token_id = token_id_result.expect("Failed to create chronolock");

    // Before unlock time, all users should get TimeLocked error
    for user in &users {
        let user_id = user.to_text();
        let response = pic
            .update_call(
                backend_canister,
                *user,
                "get_user_time_decryption_key",
                encode_args((
                    unlock_time_hex.clone(),
                    user_id.clone(),
                    encryption_public_key.clone(),
                ))
                .unwrap(),
            )
            .expect("Failed to call get_user_time_decryption_key");
        let result: Result<VetKDDeriveKeyReply, ChronoError> = decode_one(&response).unwrap();
        assert_eq!(
            result,
            Err(ChronoError::TimeLocked),
            "Expected TimeLocked error before unlock time for user {}",
            user_id
        );
    }

    // Advance time past unlock_time
    pic.advance_time(std::time::Duration::from_secs(1001));

    // After unlock time, each user should get their own decryption key
    for user in &users {
        let user_id = user.to_text();
        let response = pic
            .update_call(
                backend_canister,
                *user,
                "get_user_time_decryption_key",
                encode_args((
                    unlock_time_hex.clone(),
                    user_id.clone(),
                    encryption_public_key.clone(),
                ))
                .unwrap(),
            )
            .expect("Failed to call get_user_time_decryption_key");
        let result: Result<VetKDDeriveKeyReply, ChronoError> = decode_one(&response).unwrap();
        let key = result.expect("Failed to get decryption key");

        assert!(
            !key.encrypted_key.is_empty(),
            "Expected non-empty encrypted key"
        );
        assert_eq!(
            key.encrypted_key.len(),
            192,
            "Expected encrypted key to be 48 bytes"
        );
    }

    // Optionally, check that the metadata contains all user keys
    let metadata_response = pic
        .query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc7_token_metadata",
            encode_args((token_id.clone(),)).unwrap(),
        )
        .expect("Failed to query icrc7_token_metadata");
    let returned_metadata: Option<String> = decode_one(&metadata_response).unwrap();
    assert_eq!(returned_metadata, Some(metadata.to_string()));
}
