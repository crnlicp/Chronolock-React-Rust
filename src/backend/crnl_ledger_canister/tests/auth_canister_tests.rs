// src/backend/crnl_ledger_canister/tests/auth_canister_tests.rs

use candid::{decode_args, decode_one, encode_args, CandidType, Principal};
use pocket_ic::PocketIc;
use std::{fs, time::Duration};

const BACKEND_WASM: &str =
    "../../../target/wasm32-unknown-unknown/release/crnl_ledger_canister.wasm";

#[derive(CandidType, serde::Deserialize, Clone, Debug, PartialEq)]
struct Account {
    owner: Principal,
    subaccount: Option<[u8; 32]>,
}

#[derive(CandidType, serde::Deserialize, Debug, PartialEq)]
enum LedgerError {
    InsufficientBalance,
    InsufficientFee,
    InsufficientPoolFunds,
    InsufficientAllowance,
    AlreadyRegistered,
    InvalidReferral,
    TransferError,
    FeeProcessingError,
    Unauthorized,
    InvalidAccount,
    ArithmeticError,
    VestingLocked,
    // Authentication-related errors
    NotAuthenticated,
    InvalidPrincipal,
    UnauthorizedCaller,
    AdminRequired,
}

// Helper function to create a mock Internet Identity principal (10 bytes ending with 0x01)
fn create_mock_ii_principal(seed: u8) -> Principal {
    let mut bytes = [0u8; 10];
    bytes[0] = seed;
    bytes[9] = 0x01; // Internet Identity principals end with 0x01
    Principal::from_slice(&bytes)
}

// Helper function to create a self-authenticating principal (29 bytes ending with 0x02)
fn create_self_auth_principal(seed: u8) -> Principal {
    let mut bytes = [0u8; 29];
    bytes[0] = seed;
    bytes[28] = 0x02; // Self-authenticating principals end with 0x02
    Principal::from_slice(&bytes)
}

// Helper function to enable admin bypass for testing
fn enable_admin_bypass(
    pic: &PocketIc,
    canister_id: Principal,
    admin: Principal,
) -> Result<(), LedgerError> {
    let args = encode_args((true,)).unwrap();
    let response = pic
        .update_call(canister_id, admin, "set_admin_bypass", args)
        .expect("Failed to make canister call");
    let result: Result<(), LedgerError> = decode_one(&response).unwrap();
    result
}

fn setup() -> (PocketIc, Principal, Principal) {
    std::env::set_var("POCKET_IC_BIN", "/usr/local/bin/pocket-ic");
    let pic = PocketIc::new();

    let backend_canister = pic.create_canister();
    pic.add_cycles(backend_canister, 2_000_000_000_000);
    let wasm = fs::read(BACKEND_WASM).expect("Wasm file not found, run 'cargo build'.");

    // Create a proper Internet Identity principal for admin
    let admin = create_mock_ii_principal(1);
    let init_args = encode_args((
        "Chronolock".to_string(),
        "CRNL".to_string(),
        100_000_000_000_000_000_000_u128,
        31_536_000_u64,
        100_000_u128,
        admin,
    ))
    .expect("Failed to encode init arguments");

    pic.install_canister(backend_canister, wasm, init_args, None);

    // Enable admin bypass for easier testing
    enable_admin_bypass(&pic, backend_canister, admin).expect("Failed to enable admin bypass");

    (pic, backend_canister, admin)
}

// Authentication Tests
#[test]
fn test_authentication_ii_principal_success() {
    let (pic, backend_canister, _admin) = setup();

    let ii_principal = create_mock_ii_principal(2);
    let user = Account {
        owner: ii_principal,
        subaccount: None,
    };

    let args = encode_args((user,)).unwrap();
    let response = pic
        .update_call(backend_canister, ii_principal, "register_user", args)
        .expect("Failed to call register_user");

    let result: Result<String, LedgerError> = decode_one(&response).unwrap();
    assert!(result.is_ok());
    assert!(result.unwrap().contains("User registered with 200 CRNL"));
}

#[test]
fn test_authentication_anonymous_rejection() {
    let (pic, backend_canister, _admin) = setup();

    // Disable admin bypass for this test
    let args = encode_args((false,)).unwrap();
    pic.update_call(
        backend_canister,
        create_mock_ii_principal(1),
        "set_admin_bypass",
        args,
    )
    .expect("Failed to disable admin bypass");

    let user = Account {
        owner: Principal::anonymous(),
        subaccount: None,
    };

    let args = encode_args((user,)).unwrap();
    let response = pic
        .update_call(
            backend_canister,
            Principal::anonymous(),
            "register_user",
            args,
        )
        .expect("Failed to call register_user");

    let result: Result<String, LedgerError> = decode_one(&response).unwrap();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), LedgerError::NotAuthenticated);
}

#[test]
fn test_authentication_self_auth_rejection() {
    let (pic, backend_canister, _admin) = setup();

    // Disable admin bypass for this test
    let args = encode_args((false,)).unwrap();
    pic.update_call(
        backend_canister,
        create_mock_ii_principal(1),
        "set_admin_bypass",
        args,
    )
    .expect("Failed to disable admin bypass");

    let self_auth_principal = create_self_auth_principal(1);
    let user = Account {
        owner: self_auth_principal,
        subaccount: None,
    };

    let args = encode_args((user,)).unwrap();
    let response = pic
        .update_call(backend_canister, self_auth_principal, "register_user", args)
        .expect("Failed to call register_user");

    let result: Result<String, LedgerError> = decode_one(&response).unwrap();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), LedgerError::NotAuthenticated);
}

#[test]
fn test_authentication_unauthorized_caller_mismatch() {
    let (pic, backend_canister, _admin) = setup();

    // Disable admin bypass for this test
    let args = encode_args((false,)).unwrap();
    pic.update_call(
        backend_canister,
        create_mock_ii_principal(1),
        "set_admin_bypass",
        args,
    )
    .expect("Failed to disable admin bypass");

    let ii_principal1 = create_mock_ii_principal(2);
    let ii_principal2 = create_mock_ii_principal(3);

    let user = Account {
        owner: ii_principal1,
        subaccount: None,
    };

    let args = encode_args((user,)).unwrap();
    let response = pic
        .update_call(backend_canister, ii_principal2, "register_user", args) // Different principal calling
        .expect("Failed to call register_user");

    let result: Result<String, LedgerError> = decode_one(&response).unwrap();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), LedgerError::UnauthorizedCaller);
}

#[test]
fn test_authentication_query_functions() {
    let (pic, backend_canister, _admin) = setup();

    let ii_principal = create_mock_ii_principal(2);

    // Test is_caller_authenticated with II principal
    let response = pic
        .query_call(
            backend_canister,
            ii_principal,
            "is_caller_authenticated",
            encode_args(()).unwrap(),
        )
        .expect("Failed to query is_caller_authenticated");
    let result: bool = decode_one(&response).unwrap();
    assert_eq!(result, true); // Should be authenticated due to admin bypass

    // Test is_caller_authenticated with anonymous
    let response = pic
        .query_call(
            backend_canister,
            Principal::anonymous(),
            "is_caller_authenticated",
            encode_args(()).unwrap(),
        )
        .expect("Failed to query is_caller_authenticated");
    let result: bool = decode_one(&response).unwrap();
    assert_eq!(result, false); // Anonymous should not be authenticated

    // Test is_valid_ii_principal
    let args = encode_args((ii_principal,)).unwrap();
    let response = pic
        .query_call(
            backend_canister,
            Principal::anonymous(),
            "is_valid_ii_principal",
            args,
        )
        .expect("Failed to query is_valid_ii_principal");
    let result: bool = decode_one(&response).unwrap();
    assert_eq!(result, true);

    // Test with self-authenticating principal
    let self_auth = create_self_auth_principal(1);
    let args = encode_args((self_auth,)).unwrap();
    let response = pic
        .query_call(
            backend_canister,
            Principal::anonymous(),
            "is_valid_ii_principal",
            args,
        )
        .expect("Failed to query is_valid_ii_principal");
    let result: bool = decode_one(&response).unwrap();
    assert_eq!(result, false);
}

// Complex authentication tests that need investigation (ignored for now)

#[test]
fn test_authentication_trusted_principal_management() {
    let (pic, backend_canister, admin) = setup();
    let test_principal = create_self_auth_principal(8);
    let non_trusted_principal = create_self_auth_principal(7);

    // Disable admin bypass to simulate production rules
    let response = pic
        .update_call(
            backend_canister,
            admin,
            "set_admin_bypass",
            encode_args((false,)).unwrap(),
        )
        .expect("Failed to disable admin bypass");
    let result: Result<(), LedgerError> = decode_one(&response).unwrap();
    assert!(result.is_ok());
    pic.advance_time(Duration::from_secs(1));

    // Non-trusted principal should fail to register
    let response = pic
        .update_call(
            backend_canister,
            non_trusted_principal,
            "register_user",
            encode_args((Account {
                owner: non_trusted_principal,
                subaccount: None,
            },))
            .unwrap(),
        )
        .expect("Failed to call register_user with non-trusted principal");
    let result: Result<String, LedgerError> = decode_one(&response).unwrap();
    assert_eq!(result.unwrap_err(), LedgerError::NotAuthenticated);
    pic.advance_time(Duration::from_secs(1));

    // Add trusted principal
    let response = pic
        .update_call(
            backend_canister,
            admin,
            "add_trusted_principal",
            encode_args((test_principal,)).unwrap(),
        )
        .expect("Failed to add trusted principal");
    let result: Result<(), LedgerError> = decode_one(&response).unwrap();
    assert!(result.is_ok());
    pic.advance_time(Duration::from_secs(1));

    // Trusted principal should now register successfully
    let response = pic
        .update_call(
            backend_canister,
            test_principal,
            "register_user",
            encode_args((Account {
                owner: test_principal,
                subaccount: None,
            },))
            .unwrap(),
        )
        .expect("Failed to register trusted principal");
    let result: Result<String, LedgerError> = decode_one(&response).unwrap();
    assert!(result.is_ok());
    pic.advance_time(Duration::from_secs(1));

    // Remove trusted principal
    let response = pic
        .update_call(
            backend_canister,
            admin,
            "remove_trusted_principal",
            encode_args((test_principal,)).unwrap(),
        )
        .expect("Failed to remove trusted principal");
    let result: Result<(), LedgerError> = decode_one(&response).unwrap();
    assert!(result.is_ok());
    pic.advance_time(Duration::from_secs(1));

    // Attempting to register with the same principal using a different subaccount should now fail again
    let response = pic
        .update_call(
            backend_canister,
            test_principal,
            "register_user",
            encode_args((Account {
                owner: test_principal,
                subaccount: Some([42u8; 32]),
            },))
            .unwrap(),
        )
        .expect("Failed to call register_user after removing trusted principal");
    let result: Result<String, LedgerError> = decode_one(&response).unwrap();
    assert_eq!(result.unwrap_err(), LedgerError::NotAuthenticated);
}

#[test]
fn test_authentication_trusted_principal_success() {
    let (pic, backend_canister, admin) = setup();
    let trusted_principal = create_self_auth_principal(9);

    // Ensure admin bypass enabled for initial setup actions (already enabled in setup but decode for clarity)
    let response = pic
        .update_call(
            backend_canister,
            admin,
            "set_admin_bypass",
            encode_args((true,)).unwrap(),
        )
        .expect("Failed to enable admin bypass");
    let result: Result<(), LedgerError> = decode_one(&response).unwrap();
    assert!(result.is_ok());
    pic.advance_time(Duration::from_secs(1));

    // Add trusted principal
    let response = pic
        .update_call(
            backend_canister,
            admin,
            "add_trusted_principal",
            encode_args((trusted_principal,)).unwrap(),
        )
        .expect("Failed to add trusted principal");
    let result: Result<(), LedgerError> = decode_one(&response).unwrap();
    assert!(result.is_ok());
    pic.advance_time(Duration::from_secs(1));

    // Disable admin bypass to test trusted principal flow under normal auth rules
    let response = pic
        .update_call(
            backend_canister,
            admin,
            "set_admin_bypass",
            encode_args((false,)).unwrap(),
        )
        .expect("Failed to disable admin bypass");
    let result: Result<(), LedgerError> = decode_one(&response).unwrap();
    assert!(result.is_ok());
    pic.advance_time(Duration::from_secs(1));

    // Trusted principal should be able to register even with bypass disabled
    let response = pic
        .update_call(
            backend_canister,
            trusted_principal,
            "register_user",
            encode_args((Account {
                owner: trusted_principal,
                subaccount: None,
            },))
            .unwrap(),
        )
        .expect("Failed to register trusted principal");
    let result: Result<String, LedgerError> = decode_one(&response).unwrap();
    assert!(result.is_ok());
    pic.advance_time(Duration::from_secs(1));

    // Re-enable admin bypass for subsequent operations
    let response = pic
        .update_call(
            backend_canister,
            admin,
            "set_admin_bypass",
            encode_args((true,)).unwrap(),
        )
        .expect("Failed to re-enable admin bypass");
    let result: Result<(), LedgerError> = decode_one(&response).unwrap();
    assert!(result.is_ok());
    pic.advance_time(Duration::from_secs(1));

    // Additional register attempt (different subaccount) should continue to succeed for trusted principal
    let response = pic
        .update_call(
            backend_canister,
            trusted_principal,
            "register_user",
            encode_args((Account {
                owner: trusted_principal,
                subaccount: Some([1u8; 32]),
            },))
            .unwrap(),
        )
        .expect("Failed to register trusted principal with subaccount");
    let result: Result<String, LedgerError> = decode_one(&response).unwrap();
    assert!(result.is_ok());
}

#[test]
fn test_authentication_admin_bypass_functionality() {
    // Create special setup without admin bypass enabled
    std::env::set_var("POCKET_IC_BIN", "/usr/local/bin/pocket-ic");
    let pic = PocketIc::new();
    let backend_canister = pic.create_canister();
    pic.add_cycles(backend_canister, 2_000_000_000_000);
    let wasm = fs::read(BACKEND_WASM).expect("Wasm file not found, run 'cargo build'.");
    let admin = create_mock_ii_principal(1);
    let init_args = encode_args((
        "Chronolock".to_string(),
        "CRNL".to_string(),
        100_000_000_000_000_000_000_u128,
        31_536_000_u64,
        100_000_u128,
        admin,
    ))
    .expect("Failed to encode init arguments");
    pic.install_canister(backend_canister, wasm, init_args, None);
    // Don't enable admin bypass here

    // Create a valid II principal (not self-auth) to test with
    let regular_principal = create_mock_ii_principal(2);

    // Regular II principal should succeed even without bypass (II principals are valid by default)
    let user = Account {
        owner: regular_principal,
        subaccount: None,
    };
    let args = encode_args((user,)).unwrap();
    let response = pic
        .update_call(backend_canister, regular_principal, "register_user", args)
        .expect("Failed to call register_user");

    let result: Result<String, LedgerError> = decode_one(&response).unwrap();
    // Should succeed - II principals are authenticated by default
    assert!(result.is_ok());

    // Enable admin bypass to show it still works
    let args = encode_args((true,)).unwrap();
    let response = pic
        .update_call(backend_canister, admin, "set_admin_bypass", args)
        .expect("Failed to enable admin bypass");
    let result: Result<(), LedgerError> = decode_one(&response).unwrap();
    assert!(result.is_ok());

    // Regular II principal should still work
    let user = Account {
        owner: regular_principal,
        subaccount: Some([1u8; 32]),
    };
    let args = encode_args((user,)).unwrap();
    let response = pic
        .update_call(backend_canister, regular_principal, "register_user", args)
        .expect("Failed to call register_user");

    let result: Result<String, LedgerError> = decode_one(&response).unwrap();
    assert!(result.is_ok());
}

#[test]
fn test_authentication_admin_required_operations() {
    let (pic, backend_canister, admin) = setup();

    let ii_principal = create_mock_ii_principal(2);

    // Try to call admin function with non-admin principal
    let args = encode_args((100_u128,)).unwrap();
    let response = pic
        .update_call(backend_canister, ii_principal, "set_transfer_fee", args)
        .expect("Call should succeed but return error");

    let result: Result<(), LedgerError> = decode_one(&response).unwrap();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), LedgerError::AdminRequired);

    // Verify that admin can call the function successfully
    let args = encode_args((100_u128,)).unwrap();
    let response = pic
        .update_call(backend_canister, admin, "set_transfer_fee", args)
        .expect("Admin call should succeed");

    let result: Result<(), LedgerError> = decode_one(&response).unwrap();
    assert!(result.is_ok());
}

#[test]
fn test_authentication_get_caller_principal_info() {
    let (pic, backend_canister, admin) = setup();

    let ii_principal = create_mock_ii_principal(2); // Different from admin

    // Test with admin (bypass already enabled in setup)
    // Use query_call to fetch tuple response directly and verify admin context
    let response = pic
        .query_call(
            backend_canister,
            admin,
            "get_caller_principal_info",
            encode_args(()).unwrap(),
        )
        .expect("Failed to call get_caller_principal_info");

    // Decode the tuple response
    let result: (Principal, bool, bool) = decode_args(&response).unwrap();
    assert_eq!(result.0, admin); // Correct principal
    assert_eq!(result.1, true); // Is authenticated (due to bypass)
    assert_eq!(result.2, true); // Is admin
    pic.advance_time(Duration::from_secs(1));

    // Test with different II principal (should be authenticated due to admin bypass)
    let response = pic
        .query_call(
            backend_canister,
            ii_principal,
            "get_caller_principal_info",
            encode_args(()).unwrap(),
        )
        .expect("Failed to call get_caller_principal_info");

    let result: (Principal, bool, bool) = decode_args(&response).unwrap();
    assert_eq!(result.0, ii_principal); // Correct principal
    assert_eq!(result.1, true); // Is authenticated (due to admin bypass)
    assert_eq!(result.2, false); // Is not admin
}

#[test]
fn test_principal_formats() {
    // Test to understand principal formats
    let mock_ii = create_mock_ii_principal(2);
    let self_auth = create_self_auth_principal(1);
    
    println!("Mock II Principal: {}", mock_ii.to_text());
    println!("  Segments: {:?}", mock_ii.to_text().split('-').collect::<Vec<_>>());
    println!("  Bytes len: {}", mock_ii.as_slice().len());
    println!("  Last byte: {:02x}", mock_ii.as_slice().last().unwrap());
    
    println!("Self-auth Principal: {}", self_auth.to_text());
    println!("  Segments: {:?}", self_auth.to_text().split('-').collect::<Vec<_>>());
    println!("  Bytes len: {}", self_auth.as_slice().len());
    println!("  Last byte: {:02x}", self_auth.as_slice().last().unwrap());
    
    let real_ii = Principal::from_text("4s3y7-25yvt-jbdte-vpvcq-n4ghs-j5jo6-beihs-om2zi-oqzu6-krbhf-gqe").unwrap();
    println!("Real II Principal: {}", real_ii.to_text());
    println!("  Segments: {:?}", real_ii.to_text().split('-').collect::<Vec<_>>());
    println!("  Bytes len: {}", real_ii.as_slice().len());
    println!("  Last byte: {:02x}", real_ii.as_slice().last().unwrap());
    
    let test_ii = Principal::from_text("dmp4o-pkoo3-lnzzj-cystz-2jlkk-v4zcv-yc5h4-iqoeg-v5arm-avsbm-bae").unwrap();
    println!("Test II Principal from lib.rs: {}", test_ii.to_text());
    println!("  Segments: {:?}", test_ii.to_text().split('-').collect::<Vec<_>>());
    println!("  Bytes len: {}", test_ii.as_slice().len());
    println!("  Last byte: {:02x}", test_ii.as_slice().last().unwrap());
}
