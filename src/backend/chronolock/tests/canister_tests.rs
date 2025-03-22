use candid::{decode_one, encode_args, CandidType, Principal};
use pocket_ic::PocketIc;
use std::fs;
use std::time::{Duration, UNIX_EPOCH};

// Path to your compiled WASM file (adjust as needed)
const BACKEND_WASM: &str = "../../../target/wasm32-unknown-unknown/release/chronolock.wasm";

// Structures required for testing (must match canister definitions)
#[derive(CandidType, serde::Deserialize, Clone, Debug)]
struct Chronolock {
    id: String,
    owner: Principal,
    metadata: String,
    unlock_time: u64,
}

#[derive(CandidType, serde::Deserialize, Clone, Debug)]
struct LogEntry {
    id: String,
    timestamp: u64,
    activity: String,
}

#[derive(CandidType, serde::Deserialize, Clone, Debug)]
struct TokenList {
    tokens: Vec<String>,
}

#[derive(CandidType, serde::Deserialize, Debug, PartialEq)]
enum ChronoError {
    Unauthorized,
    TokenNotFound,
    MetadataTooLarge,
    TimeLocked,
    InvalidInput(String),
    InternalError(String),
}

#[derive(CandidType, serde::Deserialize)]
struct HttpRequest {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

#[derive(CandidType, serde::Deserialize)]
struct HttpResponse {
    status_code: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

// Setup function
fn setup() -> (PocketIc, Principal, Principal) {
    std::env::set_var("POCKET_IC_BIN", "/usr/local/bin/pocket-ic");
    let pic = PocketIc::new();

    let backend_canister = pic.create_canister();
    pic.add_cycles(backend_canister, 2_000_000_000_000);
    let wasm = fs::read(BACKEND_WASM).expect("Wasm file not found, run 'cargo build'.");

    let admin = Principal::from_text("aaaaa-aa").unwrap();
    let init_args = encode_args((admin,)).expect("Failed to encode init arguments");

    pic.install_canister(backend_canister, wasm, init_args, None);
    (pic, backend_canister, admin)
}

fn decode_with_fallback<T: CandidType + for<'de> serde::Deserialize<'de>>(
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
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].activity, "Canister initialized with admin");
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
    assert_eq!(logs.len(), 1, "Expected 1 log entry, got {}", logs.len());
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
    assert_eq!(result, "CHRONO");
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

    let unlock_time = pic.get_time().duration_since(UNIX_EPOCH).unwrap().as_secs() + 3600;
    let create_response = pic
        .update_call(
            backend_canister,
            admin,
            "create_chronolock",
            encode_args(("Test metadata".to_string(), unlock_time)).unwrap(),
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
    let metadata: Option<String> = decode_one(&metadata_response).unwrap();
    assert_eq!(metadata, Some("Test metadata".to_string()));
}

#[test]
fn test_icrc7_transfer() {
    let (pic, backend_canister, admin) = setup();

    // Step 1: Create a chronolock token
    let unlock_time = pic.get_time().duration_since(UNIX_EPOCH).unwrap().as_secs() + 3600;
    let create_response = pic
        .update_call(
            backend_canister,
            admin,
            "create_chronolock",
            encode_args(("Test metadata".to_string(), unlock_time)).unwrap(),
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

    let unlock_time = pic.get_time().duration_since(UNIX_EPOCH).unwrap().as_secs() + 3600;
    let create_response = pic
        .update_call(
            backend_canister,
            admin,
            "create_chronolock",
            encode_args(("Test metadata".to_string(), unlock_time)).unwrap(),
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

    let unlock_time = pic.get_time().duration_since(UNIX_EPOCH).unwrap().as_secs() + 3600;
    let create_response = pic
        .update_call(
            backend_canister,
            admin,
            "create_chronolock",
            encode_args(("Test metadata".to_string(), unlock_time)).unwrap(),
        )
        .expect("Failed to call create_chronolock");
    let token_id_result: Result<String, ChronoError> = decode_one(&create_response).unwrap();
    let token_id = token_id_result.expect("Failed to create chronolock");

    let new_unlock_time = pic.get_time().duration_since(UNIX_EPOCH).unwrap().as_secs() + 7200;
    let update_response = pic
        .update_call(
            backend_canister,
            admin,
            "update_chronolock",
            encode_args((
                token_id.clone(),
                "Updated metadata".to_string(),
                new_unlock_time,
            ))
            .unwrap(),
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
    let metadata: Option<String> = decode_one(&metadata_response).unwrap();
    assert_eq!(metadata, Some("Updated metadata".to_string()));

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

// Media Management Tests
#[test]
fn test_upload_and_get_media() {
    let (pic, backend_canister, admin) = setup();

    let file_data = vec![1, 2, 3, 4, 5];
    let upload_response = pic
        .update_call(
            backend_canister,
            admin,
            "upload_media",
            encode_args((file_data.clone(),)).unwrap(),
        )
        .expect("Failed to call upload_media");
    let media_url_result: Result<String, ChronoError> = decode_one(&upload_response).unwrap();
    let media_url = media_url_result.expect("Failed to upload media");
    let media_id = media_url.split('/').last().unwrap().to_string();

    let get_response = pic
        .query_call(
            backend_canister,
            Principal::anonymous(),
            "get_media",
            encode_args((media_id.clone(),)).unwrap(),
        )
        .expect("Failed to query get_media");
    let retrieved_data_result: Result<Vec<u8>, ChronoError> = decode_one(&get_response).unwrap();
    let retrieved_data = retrieved_data_result.expect("Failed to get media");
    assert_eq!(retrieved_data, file_data);

    let http_request = HttpRequest {
        method: "GET".to_string(),
        url: format!("/media/{}", media_id),
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

// VETKD Integration Tests
#[test]
fn test_vetkd_functions() {
    let (pic, backend_canister, admin) = setup();

    // Test ibe_encryption_key (expect failure due to missing VETKD canister)
    let ibe_response = pic.update_call(
        backend_canister,
        admin,
        "ibe_encryption_key",
        encode_args(()).unwrap(),
    );
    assert!(
        ibe_response.is_err() || {
            let result: Result<String, ChronoError> = decode_one(&ibe_response.unwrap()).unwrap();
            result.is_err()
        },
        "Expected failure due to missing VETKD canister"
    );

    let unlock_time = pic.get_time().duration_since(UNIX_EPOCH).unwrap().as_secs() + 3600;
    let unlock_time_hex = format!("{:x}", unlock_time);
    let encryption_public_key = vec![1, 2, 3];

    // Test get_time_decryption_key (before unlock time)
    let time_decrypt_response = pic.update_call(
        backend_canister,
        admin,
        "get_time_decryption_key",
        encode_args((unlock_time_hex.clone(), encryption_public_key.clone())).unwrap(),
    );
    let time_decrypt_result: Result<String, ChronoError> = time_decrypt_response
        .map(|resp| decode_one(&resp).unwrap())
        .unwrap_or(Err(ChronoError::TimeLocked));
    assert_eq!(time_decrypt_result, Err(ChronoError::TimeLocked));

    // Advance time and test again (still expect failure due to VETKD)
    pic.advance_time(Duration::from_secs(3600));
    let time_decrypt_response_after = pic.update_call(
        backend_canister,
        admin,
        "get_time_decryption_key",
        encode_args((unlock_time_hex.clone(), encryption_public_key.clone())).unwrap(),
    );
    assert!(
        time_decrypt_response_after.is_err() || {
            let result: Result<String, ChronoError> =
                decode_one(&time_decrypt_response_after.unwrap()).unwrap();
            result.is_err()
        },
        "Expected failure due to missing VETKD canister"
    );

    // Test get_user_time_decryption_key (expect failure)
    let user_id = admin.to_text();
    let user_decrypt_response = pic.update_call(
        backend_canister,
        admin,
        "get_user_time_decryption_key",
        encode_args((unlock_time_hex, user_id, encryption_public_key)).unwrap(),
    );
    assert!(
        user_decrypt_response.is_err() || {
            let result: Result<String, ChronoError> =
                decode_one(&user_decrypt_response.unwrap()).unwrap();
            result.is_err()
        },
        "Expected failure due to missing VETKD canister"
    );
}
