// src/backend/chronolock/tests/canister_tests.rs

use base64::{engine::general_purpose, Engine as _};
use candid::{decode_one, encode_args, CandidType, Principal};
use pocket_ic::PocketIc;
use serde::Deserialize;
use serde_json;
use std::fs;

// Path to compiled WASM file (adjust as needed)
const BACKEND_WASM: &str =
    "../../../target/wasm32-unknown-unknown/release/chronolock_canister.wasm";

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

#[derive(CandidType, Deserialize, Debug, PartialEq)]
enum ChronoError {
    Unauthorized,
    TokenNotFound,
    MetadataTooLarge,
    TimeLocked,
    InvalidInput(String),
    InternalError(String),
    NotAuthenticated,
    AdminRequired,
    InvalidPrincipal,
    UnauthorizedCaller,
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

#[derive(CandidType, Deserialize, Debug, PartialEq)]
pub struct VetKDDeriveKeyReply {
    pub encrypted_key: Vec<u8>,
}

// Helper function to create an Internet Identity principal
fn create_ii_principal(seed: u8) -> Principal {
    // Create a 10-byte array ending with 0x01 (Internet Identity marker)
    let mut bytes = [0u8; 10];
    bytes[0] = seed; // Use seed for uniqueness
    bytes[9] = 0x01; // II marker
    Principal::from_slice(&bytes)
}

// Setup function
fn setup() -> (PocketIc, Principal, Principal) {
    // Let PocketIC handle the binary download automatically
    let pic = PocketIc::new();

    // Deploy Chronolock canister
    let backend_canister = pic.create_canister();
    pic.add_cycles(backend_canister, 2_000_000_000_000);
    let wasm = fs::read(BACKEND_WASM).expect("Wasm file not found, run 'cargo build'.");

    // Create an admin principal (II principal)
    let admin = create_ii_principal(1);

    let init_args =
        encode_args((admin, Some("local".to_string()))).expect("Failed to encode init arguments");
    pic.install_canister(backend_canister, wasm, init_args, None);

    // Enable admin bypass to allow tests to work without complex authentication setup
    let _bypass_response = pic
        .update_call(
            backend_canister,
            admin,
            "set_admin_bypass",
            encode_args((true,)).unwrap(),
        )
        .expect("Failed to enable admin bypass");

    (pic, backend_canister, admin)
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
    let (pic, backend_canister, admin) = setup();

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
    assert_eq!(logs.len(), 2); // Init log + admin bypass enabled log
    let expected_log = format!("Canister initialized with admin: {}", admin);
    assert_eq!(logs[0].activity, expected_log);
    assert_eq!(logs[1].activity, "Admin bypass enabled: true");
}

// Admin Function Tests
#[test]
fn test_set_max_metadata_size() {
    let (pic, backend_canister, admin) = setup();

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
    let (pic, backend_canister, admin) = setup();

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
    assert_eq!(
        logs.len(),
        2,
        "Expected 2 log entries (init + admin bypass), got {}",
        logs.len()
    );
}

// ICRC-7 Query Tests
#[test]
fn test_icrc7_symbol() {
    let (pic, backend_canister, _) = setup();

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
    let (pic, backend_canister, _) = setup();

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
    let (pic, backend_canister, _) = setup();

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
    let (pic, backend_canister, _) = setup();

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
    let (pic, backend_canister, admin) = setup();

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
    let (pic, backend_canister, admin) = setup();

    let unlock_time = (pic.get_time().as_nanos_since_unix_epoch() / 1_000_000_000) + 3600;
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
    let (pic, backend_canister, admin) = setup();

    // Step 1: Create a chronolock token
    let unlock_time = (pic.get_time().as_nanos_since_unix_epoch() / 1_000_000_000) + 3600;
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
    let (pic, backend_canister, admin) = setup();

    let unlock_time = (pic.get_time().as_nanos_since_unix_epoch() / 1_000_000_000) + 3600;
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
    let (pic, backend_canister, admin) = setup();

    let unlock_time = (pic.get_time().as_nanos_since_unix_epoch() / 1_000_000_000) + 3600;
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

    let new_unlock_time = (pic.get_time().as_nanos_since_unix_epoch() / 1_000_000_000) + 7200;
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
    let (pic, backend_canister, admin) = setup();

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
    let start_result: Result<String, ChronoError> = decode_one(&start_response).unwrap();
    let media_id = start_result.expect("Failed to start media upload");

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

    // Use chunked retrieval
    let mut retrieved_data = vec![];
    let chunk_size = 10;
    let mut offset = 0;
    loop {
        let get_response = pic
            .query_call(
                backend_canister,
                Principal::anonymous(),
                "get_media_chunk",
                encode_args((media_id_from_url.clone(), offset as u32, chunk_size as u32)).unwrap(),
            )
            .expect("Failed to query get_media_chunk");
        let chunk_result: Result<Vec<u8>, ChronoError> = decode_one(&get_response).unwrap();
        let chunk = chunk_result.expect("Failed to get media chunk");
        if chunk.is_empty() {
            break;
        }
        retrieved_data.extend_from_slice(&chunk);
        offset += chunk.len();
    }
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

#[test]
fn test_get_total_chronolocks_count() {
    let (pic, backend_canister, admin) = setup();

    // Initially should be 0
    let response = pic
        .query_call(
            backend_canister,
            admin,
            "get_total_chronolocks_count",
            encode_args(()).unwrap(),
        )
        .expect("Failed to query get_total_chronolocks_count");
    let count: u64 = decode_one(&response).unwrap();
    assert_eq!(count, 0, "Initial count should be 0");

    // Create a few chronolocks
    for i in 0..3 {
        let metadata = format!("metadata_{}", i);
        pic.update_call(
            backend_canister,
            admin,
            "create_chronolock",
            encode_args((metadata,)).unwrap(),
        )
        .expect("Failed to create chronolock");
    }

    // Check count again
    let response = pic
        .query_call(
            backend_canister,
            admin,
            "get_total_chronolocks_count",
            encode_args(()).unwrap(),
        )
        .expect("Failed to query get_total_chronolocks_count");
    let count: u64 = decode_one(&response).unwrap();
    assert_eq!(count, 3, "Count should be 3 after creating 3 chronolocks");
}

#[test]
fn test_get_owner_chronolocks_count() {
    let (pic, backend_canister, admin) = setup();
    let user1 = Principal::self_authenticating(&[1, 2, 3]);
    let user2 = Principal::self_authenticating(&[4, 5, 6]);

    // Initially should be 0 for all users
    let response = pic
        .query_call(
            backend_canister,
            admin,
            "get_owner_chronolocks_count",
            encode_args((admin,)).unwrap(),
        )
        .expect("Failed to query get_owner_chronolocks_count");
    let count: u64 = decode_one(&response).unwrap();
    assert_eq!(count, 0, "Initial count should be 0 for admin");

    // Create chronolocks for admin
    for i in 0..2 {
        let metadata = format!("admin_metadata_{}", i);
        pic.update_call(
            backend_canister,
            admin,
            "create_chronolock",
            encode_args((metadata,)).unwrap(),
        )
        .expect("Failed to create chronolock for admin");
    }

    // Create chronolocks for user1
    for i in 0..3 {
        let metadata = format!("user1_metadata_{}", i);
        pic.update_call(
            backend_canister,
            user1,
            "create_chronolock",
            encode_args((metadata,)).unwrap(),
        )
        .expect("Failed to create chronolock for user1");
    }

    // Check counts
    let response = pic
        .query_call(
            backend_canister,
            admin,
            "get_owner_chronolocks_count",
            encode_args((admin,)).unwrap(),
        )
        .expect("Failed to query admin count");
    let admin_count: u64 = decode_one(&response).unwrap();
    assert_eq!(admin_count, 2, "Admin should have 2 chronolocks");

    let response = pic
        .query_call(
            backend_canister,
            admin,
            "get_owner_chronolocks_count",
            encode_args((user1,)).unwrap(),
        )
        .expect("Failed to query user1 count");
    let user1_count: u64 = decode_one(&response).unwrap();
    assert_eq!(user1_count, 3, "User1 should have 3 chronolocks");

    let response = pic
        .query_call(
            backend_canister,
            admin,
            "get_owner_chronolocks_count",
            encode_args((user2,)).unwrap(),
        )
        .expect("Failed to query user2 count");
    let user2_count: u64 = decode_one(&response).unwrap();
    assert_eq!(user2_count, 0, "User2 should have 0 chronolocks");
}

#[test]
fn test_get_all_chronolocks_paginated() {
    let (pic, backend_canister, admin) = setup();

    // Create 5 chronolocks
    let mut created_ids = Vec::new();
    for i in 0..5 {
        let metadata = format!("metadata_{}", i);
        let create_response = pic
            .update_call(
                backend_canister,
                admin,
                "create_chronolock",
                encode_args((metadata,)).unwrap(),
            )
            .expect("Failed to create chronolock");
        let token_id_result: Result<String, ChronoError> = decode_one(&create_response).unwrap();
        let token_id = token_id_result.expect("Failed to create chronolock");
        created_ids.push(token_id);
    }

    // Test pagination
    let response = pic
        .query_call(
            backend_canister,
            admin,
            "get_all_chronolocks_paginated",
            encode_args((0u64, 3u64)).unwrap(),
        )
        .expect("Failed to query paginated chronolocks");
    let result: Result<Vec<Chronolock>, ChronoError> = decode_one(&response).unwrap();
    let chronolocks = result.expect("Failed to get chronolocks");
    assert_eq!(chronolocks.len(), 3, "Should return 3 chronolocks");

    // Test second page
    let response = pic
        .query_call(
            backend_canister,
            admin,
            "get_all_chronolocks_paginated",
            encode_args((3u64, 3u64)).unwrap(),
        )
        .expect("Failed to query paginated chronolocks");
    let result: Result<Vec<Chronolock>, ChronoError> = decode_one(&response).unwrap();
    let chronolocks = result.expect("Failed to get chronolocks");
    assert_eq!(
        chronolocks.len(),
        2,
        "Should return 2 chronolocks on second page"
    );

    // Test limit enforcement (max 100)
    let response = pic
        .query_call(
            backend_canister,
            admin,
            "get_all_chronolocks_paginated",
            encode_args((0u64, 150u64)).unwrap(),
        )
        .expect("Failed to query paginated chronolocks");
    let result: Result<Vec<Chronolock>, ChronoError> = decode_one(&response).unwrap();
    let chronolocks = result.expect("Failed to get chronolocks");
    assert_eq!(
        chronolocks.len(),
        5,
        "Should respect max limit and return all 5"
    );
}

#[test]
fn test_get_owner_chronolocks_paginated() {
    let (pic, backend_canister, admin) = setup();
    let user1 = Principal::self_authenticating(&[1, 2, 3]);

    // Create chronolocks for user1
    let mut created_ids = Vec::new();
    for i in 0..4 {
        let metadata = format!("user1_metadata_{}", i);
        let create_response = pic
            .update_call(
                backend_canister,
                user1,
                "create_chronolock",
                encode_args((metadata,)).unwrap(),
            )
            .expect("Failed to create chronolock");
        let token_id_result: Result<String, ChronoError> = decode_one(&create_response).unwrap();
        let token_id = token_id_result.expect("Failed to create chronolock");
        created_ids.push(token_id);
    }

    // Create some chronolocks for admin to ensure filtering works
    pic.update_call(
        backend_canister,
        admin,
        "create_chronolock",
        encode_args(("admin_metadata".to_string(),)).unwrap(),
    )
    .expect("Failed to create admin chronolock");

    // Test pagination for user1
    let response = pic
        .query_call(
            backend_canister,
            admin,
            "get_owner_chronolocks_paginated",
            encode_args((user1, 0u64, 2u64)).unwrap(),
        )
        .expect("Failed to query owner chronolocks");
    let result: Result<Vec<Chronolock>, ChronoError> = decode_one(&response).unwrap();
    let chronolocks = result.expect("Failed to get chronolocks");
    assert_eq!(
        chronolocks.len(),
        2,
        "Should return 2 chronolocks for user1"
    );

    // Verify all returned chronolocks belong to user1
    for chronolock in &chronolocks {
        assert_eq!(
            chronolock.owner, user1,
            "All chronolocks should belong to user1"
        );
    }

    // Test second page
    let response = pic
        .query_call(
            backend_canister,
            admin,
            "get_owner_chronolocks_paginated",
            encode_args((user1, 2u64, 2u64)).unwrap(),
        )
        .expect("Failed to query owner chronolocks");
    let result: Result<Vec<Chronolock>, ChronoError> = decode_one(&response).unwrap();
    let chronolocks = result.expect("Failed to get chronolocks");
    assert_eq!(
        chronolocks.len(),
        2,
        "Should return 2 chronolocks on second page"
    );

    // Test empty result for user with no chronolocks
    let user2 = Principal::self_authenticating(&[7, 8, 9]);
    let response = pic
        .query_call(
            backend_canister,
            admin,
            "get_owner_chronolocks_paginated",
            encode_args((user2, 0u64, 10u64)).unwrap(),
        )
        .expect("Failed to query owner chronolocks");
    let result: Result<Vec<Chronolock>, ChronoError> = decode_one(&response).unwrap();
    let chronolocks = result.expect("Failed to get chronolocks");
    assert_eq!(
        chronolocks.len(),
        0,
        "Should return empty for user with no chronolocks"
    );
}

#[test]
fn test_get_user_accessible_chronolocks_functions() {
    let (pic, backend_canister, admin) = setup();
    let user1 = Principal::self_authenticating(&[1, 2, 3]);

    // Get current time and set unlock times
    let current_time = pic.get_time().as_nanos_since_unix_epoch() / 1_000_000_000;
    let past_time = current_time - 3600; // 1 hour ago
    let future_time = current_time + 3600; // 1 hour in future

    // Create public chronolock (accessible to everyone after unlock)
    let public_metadata = serde_json::json!({
        "title": "Public Chronolock",
        "lockTime": past_time,
        "userKeys": [{"user": "public", "key": "public_key"}],
        "encryptedMetaData": "public_encrypted_data"
    })
    .to_string();
    let public_metadata_b64 = general_purpose::STANDARD.encode(public_metadata);

    let create_response = pic
        .update_call(
            backend_canister,
            admin,
            "create_chronolock",
            encode_args((public_metadata_b64,)).unwrap(),
        )
        .expect("Failed to create public chronolock");
    let _public_token_id: Result<String, ChronoError> = decode_one(&create_response).unwrap();

    // Create user-specific chronolock (accessible only to user1 after unlock)
    let user_metadata = serde_json::json!({
        "title": "User Specific Chronolock",
        "lockTime": past_time,
        "userKeys": [{"user": format!("{}:{}", user1.to_text(), past_time), "key": "user_key"}],
        "encryptedMetaData": "user_encrypted_data"
    })
    .to_string();
    let user_metadata_b64 = general_purpose::STANDARD.encode(user_metadata);

    let create_response = pic
        .update_call(
            backend_canister,
            admin,
            "create_chronolock",
            encode_args((user_metadata_b64,)).unwrap(),
        )
        .expect("Failed to create user chronolock");
    let _user_token_id: Result<String, ChronoError> = decode_one(&create_response).unwrap();

    // Create locked chronolock (not yet unlockable)
    let locked_metadata = serde_json::json!({
        "title": "Locked Chronolock",
        "lockTime": future_time,
        "userKeys": [{"user": "public", "key": "locked_key"}],
        "encryptedMetaData": "locked_encrypted_data"
    })
    .to_string();
    let locked_metadata_b64 = general_purpose::STANDARD.encode(locked_metadata);

    let create_response = pic
        .update_call(
            backend_canister,
            admin,
            "create_chronolock",
            encode_args((locked_metadata_b64,)).unwrap(),
        )
        .expect("Failed to create locked chronolock");
    let _locked_token_id: Result<String, ChronoError> = decode_one(&create_response).unwrap();

    // Test count function
    let response = pic
        .query_call(
            backend_canister,
            admin,
            "get_user_accessible_chronolocks_count",
            encode_args((user1,)).unwrap(),
        )
        .expect("Failed to query accessible count");
    let count: u64 = decode_one(&response).unwrap();
    assert_eq!(
        count, 2,
        "User1 should have access to 2 chronolocks (public + user-specific)"
    );

    // Test pagination function
    let response = pic
        .query_call(
            backend_canister,
            admin,
            "get_user_accessible_chronolocks_paginated",
            encode_args((user1, 0u64, 10u64)).unwrap(),
        )
        .expect("Failed to query accessible chronolocks");
    let result: Result<Vec<Chronolock>, ChronoError> = decode_one(&response).unwrap();
    let accessible_chronolocks = result.expect("Failed to get accessible chronolocks");
    assert_eq!(
        accessible_chronolocks.len(),
        2,
        "Should return 2 accessible chronolocks"
    );

    // Test with a different user (should only see public chronolock)
    let user2 = Principal::self_authenticating(&[4, 5, 6]);
    let response = pic
        .query_call(
            backend_canister,
            admin,
            "get_user_accessible_chronolocks_count",
            encode_args((user2,)).unwrap(),
        )
        .expect("Failed to query accessible count for user2");
    let count: u64 = decode_one(&response).unwrap();
    assert_eq!(
        count, 1,
        "User2 should only have access to 1 chronolock (public)"
    );

    // Test pagination with limit
    let response = pic
        .query_call(
            backend_canister,
            admin,
            "get_user_accessible_chronolocks_paginated",
            encode_args((user1, 0u64, 1u64)).unwrap(),
        )
        .expect("Failed to query accessible chronolocks with limit");
    let result: Result<Vec<Chronolock>, ChronoError> = decode_one(&response).unwrap();
    let accessible_chronolocks = result.expect("Failed to get accessible chronolocks");
    assert_eq!(
        accessible_chronolocks.len(),
        1,
        "Should return only 1 chronolock due to limit"
    );
}

#[test]
fn test_pagination_edge_cases() {
    let (pic, backend_canister, admin) = setup();

    // Test with no chronolocks
    let response = pic
        .query_call(
            backend_canister,
            admin,
            "get_all_chronolocks_paginated",
            encode_args((0u64, 10u64)).unwrap(),
        )
        .expect("Failed to query empty pagination");
    let result: Result<Vec<Chronolock>, ChronoError> = decode_one(&response).unwrap();
    let chronolocks = result.expect("Failed to get chronolocks");
    assert_eq!(
        chronolocks.len(),
        0,
        "Should return empty array when no chronolocks exist"
    );

    // Create one chronolock
    pic.update_call(
        backend_canister,
        admin,
        "create_chronolock",
        encode_args(("test_metadata".to_string(),)).unwrap(),
    )
    .expect("Failed to create chronolock");

    // Test offset beyond available items
    let response = pic
        .query_call(
            backend_canister,
            admin,
            "get_all_chronolocks_paginated",
            encode_args((10u64, 5u64)).unwrap(),
        )
        .expect("Failed to query with large offset");
    let result: Result<Vec<Chronolock>, ChronoError> = decode_one(&response).unwrap();
    let chronolocks = result.expect("Failed to get chronolocks");
    assert_eq!(
        chronolocks.len(),
        0,
        "Should return empty when offset exceeds available items"
    );

    // Test zero limit
    let response = pic
        .query_call(
            backend_canister,
            admin,
            "get_all_chronolocks_paginated",
            encode_args((0u64, 0u64)).unwrap(),
        )
        .expect("Failed to query with zero limit");
    let result: Result<Vec<Chronolock>, ChronoError> = decode_one(&response).unwrap();
    let chronolocks = result.expect("Failed to get chronolocks");
    assert_eq!(chronolocks.len(), 0, "Should return empty when limit is 0");
}
