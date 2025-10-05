// src/backend/crnl_ledger/tests/canister_tests.rs

use candid::{decode_one, encode_args, CandidType, Nat, Principal};
use pocket_ic::PocketIc;
use std::fs;

const COMMUNITY_POOL_SUBACCOUNT: [u8; 32] = [1u8; 32];
const TEAM_VESTING_POOL_SUBACCOUNT: [u8; 32] = [2u8; 32];
const RESERVE_POOL_SUBACCOUNT: [u8; 32] = [3u8; 32];

const BACKEND_WASM: &str =
    "../../../target/wasm32-unknown-unknown/release/crnl_ledger_canister.wasm";

#[derive(CandidType, serde::Deserialize, Clone, Debug, PartialEq)]
struct Account {
    owner: Principal,
    subaccount: Option<[u8; 32]>,
}

#[derive(CandidType, serde::Deserialize, Clone, Debug)]
struct TransferArgs {
    from_subaccount: Option<[u8; 32]>,
    to: Account,
    amount: Nat,
}

#[derive(CandidType, serde::Deserialize, Clone, Debug)]
struct ClaimReferralArgs {
    referral_code: String,
}

#[derive(CandidType, serde::Deserialize, Clone, Debug)]
struct DeductBalanceArgs {
    caller: Account,
    amount: Nat,
    description: String,
}

#[derive(CandidType, serde::Deserialize, Clone, Debug)]
struct PoolTransferArgs {
    from_pool: String,
    to_pool: Option<String>,
    to_principal: Option<Account>,
    amount: Nat,
    description: String,
}

#[derive(CandidType, serde::Deserialize, Clone, Debug)]
struct ApproveArgs {
    from_subaccount: Option<[u8; 32]>,
    spender: Account,
    amount: Nat,
}

#[derive(CandidType, serde::Deserialize, Clone, Debug)]
struct TransferFromArgs {
    spender: Account,
    from: Account,
    to: Account,
    amount: Nat,
}

#[derive(CandidType, serde::Deserialize, Clone)]
struct TransactionEvent {
    tx_id: [u8; 32],
    timestamp: u64,
    event_type: String,
    from: Account,
    to: Option<Account>,
    spender: Option<Account>,
    amount: Nat,
    fee: Option<Nat>,
}

#[derive(CandidType, serde::Deserialize, Clone, Debug, PartialEq)]
struct LogEntry {
    timestamp: u64,
    event_type: String,
    details: String,
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

// Helper function to enable admin bypass for testing
fn enable_admin_bypass(pic: &PocketIc, canister_id: Principal, admin: Principal) {
    let args = encode_args((true,)).unwrap();
    pic.update_call(canister_id, admin, "set_admin_bypass", args)
        .expect("Failed to enable admin bypass");
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
    enable_admin_bypass(&pic, backend_canister, admin);

    (pic, backend_canister, admin)
}

// Query Tests
#[test]
fn test_icrc1_name() {
    let (pic, backend_canister, _) = setup();
    let response = pic
        .query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_name",
            encode_args(()).unwrap(),
        )
        .expect("Failed to query icrc1_name");
    let result: String = decode_one(&response).unwrap();
    assert_eq!(result, "Chronolock");
}

#[test]
fn test_icrc1_symbol() {
    let (pic, backend_canister, _) = setup();
    let response = pic
        .query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_symbol",
            encode_args(()).unwrap(),
        )
        .expect("Failed to query icrc1_symbol");
    let result: String = decode_one(&response).unwrap();
    assert_eq!(result, "CRNL");
}

#[test]
fn test_icrc1_decimals() {
    let (pic, backend_canister, _) = setup();
    let response = pic
        .query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_decimals",
            encode_args(()).unwrap(),
        )
        .expect("Failed to query icrc1_decimals");
    let result: u8 = decode_one(&response).unwrap();
    assert_eq!(result, 8);
}

#[test]
fn test_icrc1_total_supply() {
    let (pic, backend_canister, _) = setup();
    let response = pic
        .query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_total_supply",
            encode_args(()).unwrap(),
        )
        .expect("Failed to query icrc1_total_supply");
    let result: Nat = decode_one(&response).unwrap();
    assert_eq!(result, Nat::from(100_000_000_000_000_000_000_u128));
}

// Admin reset integration test
#[test]
fn test_admin_reset_ledger_via_canister() {
    let (pic, backend_canister, admin) = setup();

    // Ensure some balance exists (setup seeds pools)
    // Call admin reset via canister
    let resp = pic
        .update_call(
            backend_canister,
            admin,
            "admin_reset_stable_storage",
            encode_args(()).unwrap(),
        )
        .expect("Failed to call admin_reset_stable_storage");

    // Decode response (should be Ok(()))
    let result: Result<(), LedgerError> = decode_one(&resp).unwrap();
    assert!(result.is_ok());

    // Check logs - should have at most a small number (reset log may be present)
    let response = pic
        .query_call(
            backend_canister,
            admin,
            "get_logs_paginated",
            encode_args((0u64, 10u64)).unwrap(),
        )
        .expect("Failed to query get_logs_paginated");

    let logs_result: Result<Vec<LogEntry>, LedgerError> = decode_one(&response).unwrap();
    let logs = logs_result.expect("Failed to get logs");
    // After reset, we expect logs cleared then reset log inserted => length <= 1
    assert!(logs.len() <= 1);
}

#[test]
fn test_icrc1_fee() {
    let (pic, backend_canister, _) = setup();
    let response = pic
        .query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_fee",
            encode_args(()).unwrap(),
        )
        .expect("Failed to query icrc1_fee");
    let result: Nat = decode_one(&response).unwrap();
    assert_eq!(result, Nat::from(100_000_u128));
}

#[test]
fn test_balance_of() {
    let (pic, backend_canister, admin) = setup();
    let admin_account = Account {
        owner: admin,
        subaccount: None,
    };
    let community_account = Account {
        owner: admin,
        subaccount: Some(COMMUNITY_POOL_SUBACCOUNT),
    };
    let team_account = Account {
        owner: admin,
        subaccount: Some(TEAM_VESTING_POOL_SUBACCOUNT),
    };
    let reserve_account = Account {
        owner: admin,
        subaccount: Some(RESERVE_POOL_SUBACCOUNT),
    };

    let admin_balance: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_balance_of",
            encode_args((admin_account,)).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let community_balance: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_balance_of",
            encode_args((community_account,)).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let team_balance: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_balance_of",
            encode_args((team_account,)).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let reserve_balance: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_balance_of",
            encode_args((reserve_account,)).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    println!("Admin balance: {}", admin_balance);
    println!("Community balance: {}", community_balance);
    println!("Team vesting balance: {}", team_balance);
    println!("Reserve balance: {}", reserve_balance);

    assert_eq!(
        admin_balance + community_balance + team_balance + reserve_balance,
        Nat::from(100_000_000_000_000_000_000_u128)
    );
}

#[test]
fn test_get_logs() {
    let (pic, backend_canister, admin) = setup();
    let args = encode_args((0_u64, 100_u64)).expect("Failed to encode args");
    let response = pic
        .query_call(backend_canister, admin, "get_logs_paginated", args)
        .expect("Failed to query get_logs_paginated");
    let result: Result<Vec<LogEntry>, LedgerError> = decode_one(&response).unwrap();
    let logs = result.expect("Should succeed with admin");
    assert_eq!(logs.len(), 2); // Init + admin bypass enabled
    assert_eq!(logs[0].event_type, "Init");
    assert_eq!(logs[1].event_type, "AdminBypassChanged");
}

#[test]
fn test_get_logs_by_range() {
    let (pic, backend_canister, admin) = setup();
    let args = encode_args((0_u64, u64::MAX)).expect("Failed to encode args");
    let response = pic
        .query_call(backend_canister, admin, "get_logs_by_range", args)
        .expect("Failed to query get_logs_by_range");
    let result: Result<Vec<LogEntry>, LedgerError> = decode_one(&response).unwrap();
    let logs = result.expect("Should succeed with admin");
    assert_eq!(logs.len(), 2); // Init + admin bypass enabled
}

#[test]
fn test_icrc1_allowance() {
    let (pic, backend_canister, admin) = setup();
    let owner = Account {
        owner: admin,
        subaccount: None,
    };
    let spender = Account {
        owner: Principal::anonymous(),
        subaccount: None,
    };
    let args = encode_args((owner, spender)).expect("Failed to encode args");
    let response = pic
        .query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_allowance",
            args,
        )
        .expect("Failed to query icrc1_allowance");
    let result: Nat = decode_one(&response).unwrap();
    assert_eq!(result, Nat::from(0_u128));
}

// Update Tests
#[test]
fn test_register_user() {
    let (pic, backend_canister, admin) = setup();

    // Enable admin bypass for this test to work with existing setup
    enable_admin_bypass(&pic, backend_canister, admin);

    let ii_principal = create_mock_ii_principal(1);
    let user = Account {
        owner: ii_principal,
        subaccount: None,
    };
    let args = encode_args((user.clone(),)).expect("Failed to encode args");
    let response = pic
        .update_call(backend_canister, ii_principal, "register_user", args)
        .expect("Failed to call register_user");
    let result: Result<String, LedgerError> = decode_one(&response).unwrap();
    assert!(result.unwrap().starts_with("User registered with 200 CRNL"));
}

#[test]
fn test_claim_referral() {
    let (pic, backend_canister, _) = setup();
    let referrer = Account {
        owner: create_mock_ii_principal(2),
        subaccount: None,
    };
    let referee = Account {
        owner: create_mock_ii_principal(3),
        subaccount: None,
    };

    let reg_args = encode_args((referrer.clone(),)).expect("Failed to encode args");
    let reg_response = pic
        .update_call(backend_canister, referrer.owner, "register_user", reg_args)
        .expect("Failed to register referrer");
    let reg_result: Result<String, LedgerError> = decode_one(&reg_response).unwrap();
    let referral_code = reg_result
        .unwrap()
        .split("Your referral code is: ")
        .nth(1)
        .unwrap()
        .to_string();

    let args = encode_args((ClaimReferralArgs {
        referral_code: referral_code.clone(),
    },))
    .expect("Failed to encode args");
    let response = pic
        .update_call(backend_canister, referee.owner, "claim_referral", args)
        .expect("Failed to call claim_referral");
    let result: Result<String, LedgerError> = decode_one(&response).unwrap();
    assert_eq!(result.unwrap(), "Referral reward of 20 CRNL credited");
}

#[test]
fn test_icrc1_transfer() {
    let (pic, backend_canister, admin) = setup();
    let from = Account {
        owner: admin,
        subaccount: Some(COMMUNITY_POOL_SUBACCOUNT),
    };
    let to = Account {
        owner: Principal::from_text("2vxsx-fae").unwrap(),
        subaccount: None,
    };
    let amount = Nat::from(1_000_000_000_u128); // 10 CRNL
    let transfer_fee = 100_000_u128; // 0.001 CRNL from init

    let from_balance: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_balance_of",
            encode_args((from.clone(),)).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    println!("Transfer from balance before: {}", from_balance);

    let args = encode_args((TransferArgs {
        from_subaccount: from.subaccount,
        to: to.clone(),
        amount: amount.clone(),
    },))
    .expect("Failed to encode args");
    let response = pic
        .update_call(backend_canister, admin, "icrc1_transfer", args)
        .expect("Failed to call icrc1_transfer");
    let result: Result<Nat, LedgerError> = decode_one(&response).unwrap();
    if let Err(e) = &result {
        println!("Transfer failed with: {:?}", e);
    }
    assert!(result.is_ok());

    let to_balance: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_balance_of",
            encode_args((to,)).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        to_balance,
        amount - Nat::from(transfer_fee),
        "Recipient should receive amount minus fee"
    );
}

#[test]
fn test_icrc1_approve_and_transfer_from() {
    let (pic, backend_canister, admin) = setup();
    let owner = Account {
        owner: admin,
        subaccount: Some(COMMUNITY_POOL_SUBACCOUNT),
    };
    let spender = Account {
        owner: create_mock_ii_principal(2),
        subaccount: None,
    };
    let to = Account {
        owner: Principal::anonymous(),
        subaccount: None,
    };
    let amount = Nat::from(1_000_000_000_u128); // 10 CRNL
    let transfer_fee = Nat::from(100_000_u128); // 0.001 CRNL

    let owner_balance: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_balance_of",
            encode_args((owner.clone(),)).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    println!("Owner balance before approve: {}", owner_balance);

    let approve_args = encode_args(((ApproveArgs {
        from_subaccount: owner.subaccount,
        spender: spender.clone(),
        amount: amount.clone(),
    }),))
    .expect("Failed to encode args");
    let approve_response = pic
        .update_call(backend_canister, admin, "icrc1_approve", approve_args)
        .expect("Failed to call icrc1_approve");
    let approve_result: Result<Nat, LedgerError> = decode_one(&approve_response).unwrap();
    assert!(approve_result.is_ok());

    let transfer_args = encode_args((TransferFromArgs {
        spender: spender.clone(),
        from: owner.clone(),
        to: to.clone(),
        amount: amount.clone(),
    },))
    .expect("Failed to encode args");
    let transfer_response = pic
        .update_call(
            backend_canister,
            spender.owner,
            "icrc1_transfer_from",
            transfer_args,
        )
        .expect("Failed to call icrc1_transfer_from");
    let transfer_result: Result<Nat, LedgerError> = decode_one(&transfer_response).unwrap();
    assert!(transfer_result.is_ok());

    let to_balance: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_balance_of",
            encode_args((to,)).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        to_balance,
        amount - transfer_fee,
        "Recipient should receive amount minus fee"
    );
}

#[test]
fn test_deduct_from_balance() {
    let (pic, backend_canister, admin) = setup();
    let caller = Account {
        owner: admin,
        subaccount: Some(RESERVE_POOL_SUBACCOUNT),
    };

    let caller_balance_args = encode_args((caller.clone(),)).expect("Failed to encode args");
    let caller_balance: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_balance_of",
            caller_balance_args.clone(),
        )
        .unwrap(),
    )
    .unwrap();
    println!("Caller balance before deduction: {}", caller_balance);

    let deduction_amount = Nat::from(2_000_000_000_u128); // 20 CRNL
    let args = encode_args((DeductBalanceArgs {
        caller: caller.clone(),
        amount: deduction_amount.clone(),
        description: "Media ChronoLock Creation".to_string(),
    },))
    .expect("Failed to encode args");
    let response = pic
        .update_call(backend_canister, admin, "deduct_from_balance", args)
        .expect("Failed to call deduct_from_balance");
    let result: Result<String, LedgerError> = decode_one(&response).unwrap();
    if let Err(e) = &result {
        println!("Deduct from balance failed with: {:?}", e);
    }
    assert!(result.is_ok());
    let result_msg = result.unwrap();
    assert!(
        result_msg.contains("Deducted 20 CRNL"),
        "Result message should contain deduction amount: {}",
        result_msg
    );

    let post_balance: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_balance_of",
            caller_balance_args,
        )
        .unwrap(),
    )
    .unwrap();
    println!("Caller balance after deduction: {}", post_balance);
    assert_eq!(post_balance, caller_balance - deduction_amount);
}

#[test]
fn test_set_transfer_fee() {
    let (pic, backend_canister, admin) = setup();
    let args = encode_args((200_000_u128,)).expect("Failed to encode args");
    let response = pic
        .update_call(backend_canister, admin, "set_transfer_fee", args)
        .expect("Failed to call set_transfer_fee");
    let result: Result<(), LedgerError> = decode_one(&response).unwrap();
    assert!(result.is_ok());

    let fee_response = pic
        .query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_fee",
            encode_args(()).unwrap(),
        )
        .expect("Failed to query icrc1_fee");
    let fee: Nat = decode_one(&fee_response).unwrap();
    assert_eq!(fee, Nat::from(200_000_u128));
}

#[test]
fn test_set_transfer_fee_authorization() {
    let (pic, backend_canister, admin) = setup();
    let non_admin = create_mock_ii_principal(2);

    let args = encode_args((200_000_u128,)).expect("Failed to encode args");
    let response = pic
        .update_call(backend_canister, non_admin, "set_transfer_fee", args)
        .expect("Failed to call set_transfer_fee with non-admin");
    let result: Result<(), LedgerError> = decode_one(&response).expect("Failed to decode response");
    assert!(
        result.is_err(),
        "Non-admin should not be able to set transfer fee"
    );
    if let Err(e) = result {
        assert_eq!(
            e,
            LedgerError::AdminRequired,
            "Error should be AdminRequired"
        );
    }

    let args = encode_args((200_000_u128,)).expect("Failed to encode args");
    let response = pic
        .update_call(backend_canister, admin, "set_transfer_fee", args)
        .expect("Failed to call set_transfer_fee");
    let result: Result<(), LedgerError> = decode_one(&response).unwrap();
    assert!(result.is_ok(), "Admin should be able to set transfer fee");

    let fee_response = pic
        .query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_fee",
            encode_args(()).unwrap(),
        )
        .expect("Failed to query icrc1_fee");
    let fee: Nat = decode_one(&fee_response).unwrap();
    assert_eq!(
        fee,
        Nat::from(200_000_u128),
        "Transfer fee should be updated to 200,000"
    );
}

#[test]
fn test_admin_mint() {
    let (pic, backend_canister, admin) = setup();

    // Recipient principal
    let recipient =
        Principal::from_text("dmp4o-pkoo3-lnzzj-cystz-2jlkk-v4zcv-yc5h4-iqoeg-v5arm-avsbm-bae")
            .expect("valid principal text");
    let recipient_account = Account {
        owner: recipient,
        subaccount: None,
    };

    // Query total supply before mint
    let before_supply: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_total_supply",
            encode_args(()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    // Mint amount
    let mint_amount = Nat::from(1_000_000_000_u128); // 10 CRNL
    let args = encode_args((
        recipient_account.clone(),
        mint_amount.clone(),
        Some("test mint".to_string()),
        None::<Vec<u8>>,
    ))
    .expect("Failed to encode args");

    let response = pic
        .update_call(backend_canister, admin, "admin_mint", args)
        .expect("Failed to call admin_mint");
    let result: Result<Nat, LedgerError> = decode_one(&response).unwrap();
    assert!(result.is_ok());

    // Verify recipient balance increased
    let bal: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_balance_of",
            encode_args((recipient_account.clone(),)).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(bal, mint_amount);

    // Verify total supply increased
    let after_supply: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_total_supply",
            encode_args(()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(after_supply, before_supply + mint_amount);
}

#[test]
fn test_unique_referral_codes() {
    let (pic, backend_canister, _) = setup();
    let user1 = Account {
        owner: create_mock_ii_principal(2),
        subaccount: None,
    };
    let user2 = Account {
        owner: create_mock_ii_principal(3),
        subaccount: None,
    };

    pic.update_call(
        backend_canister,
        user1.owner,
        "register_user",
        encode_args((user1.clone(),)).unwrap(),
    )
    .expect("Failed to register user1");
    pic.update_call(
        backend_canister,
        user2.owner,
        "register_user",
        encode_args((user2.clone(),)).unwrap(),
    )
    .expect("Failed to register user2");

    let code1: Option<String> = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "get_referral_code",
            encode_args((user1,)).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let code2: Option<String> = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "get_referral_code",
            encode_args((user2,)).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    assert!(
        code1.is_some() && code2.is_some(),
        "Referral codes should be generated"
    );
    assert_ne!(
        code1.unwrap(),
        code2.unwrap(),
        "Referral codes should be unique"
    );
}

#[test]
fn test_referral_reward() {
    let (pic, backend_canister, _admin) = setup();
    let referrer = Account {
        owner: create_mock_ii_principal(2),
        subaccount: None,
    };
    let referee = Account {
        owner: create_mock_ii_principal(3),
        subaccount: None,
    };

    let reg_response = pic
        .update_call(
            backend_canister,
            referrer.owner,
            "register_user",
            encode_args((referrer.clone(),)).unwrap(),
        )
        .expect("Failed to register referrer");
    let referral_code = decode_one::<Result<String, LedgerError>>(&reg_response)
        .unwrap()
        .unwrap()
        .split("Your referral code is: ")
        .nth(1)
        .unwrap()
        .to_string();

    pic.update_call(
        backend_canister,
        referee.owner,
        "claim_referral",
        encode_args((ClaimReferralArgs {
            referral_code: referral_code.clone(),
        },))
        .unwrap(),
    )
    .expect("Failed to claim referral");

    let referrer_balance: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_balance_of",
            encode_args((referrer,)).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let community_pool: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "get_community_pool_balance",
            encode_args(()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(
        referrer_balance,
        Nat::from(20_000_000_000_u128 + 2_000_000_000_u128),
        "Referrer should receive reward"
    );
    assert_eq!(
        community_pool,
        Nat::from(50_000_000_000_000_000_000_u128 - 20_000_000_000_u128 - 2_000_000_000_u128),
        "Community pool should be deducted"
    );
}

#[test]
fn test_transfer_insufficient_balance() {
    let (pic, backend_canister, admin) = setup();
    let from = Account {
        owner: admin,
        subaccount: Some(COMMUNITY_POOL_SUBACCOUNT),
    };
    let to = Account {
        owner: Principal::from_text("2vxsx-fae").unwrap(),
        subaccount: None,
    };
    let amount = Nat::from(50_000_000_000_000_000_001_u128); // Exceeds pool

    let response = pic
        .update_call(
            backend_canister,
            admin,
            "icrc1_transfer",
            encode_args(((TransferArgs {
                from_subaccount: from.subaccount,
                to,
                amount,
            }),))
            .unwrap(),
        )
        .expect("Failed to call icrc1_transfer");
    let result: Result<Nat, LedgerError> =
        decode_one(&response).expect("Failed to decode response");
    assert!(
        result.is_err(),
        "Transfer with insufficient balance should fail"
    );
    if let Err(e) = result {
        assert_eq!(
            e,
            LedgerError::InsufficientBalance,
            "Error should be InsufficientBalance"
        );
    }
}

#[test]
fn test_icrc1_functions() {
    let (pic, backend_canister, admin) = setup();
    let owner = Account {
        owner: admin,
        subaccount: Some(COMMUNITY_POOL_SUBACCOUNT), // Match transfer source
    };
    let spender = Account {
        owner: create_mock_ii_principal(2), // Use proper II principal
        subaccount: None,
    };
    let to = Account {
        owner: Principal::anonymous(),
        subaccount: None,
    };
    let amount = Nat::from(1_000_000_000_u128);
    let transfer_fee = Nat::from(100_000_u128);

    pic.update_call(
        backend_canister,
        admin,
        "icrc1_approve",
        encode_args(((ApproveArgs {
            from_subaccount: owner.subaccount,
            spender: spender.clone(),
            amount: amount.clone(),
        }),))
        .unwrap(),
    )
    .expect("Failed to approve");

    let allowance: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_allowance",
            encode_args((owner.clone(), spender.clone())).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(allowance, amount, "Allowance should match approved amount");

    pic.update_call(
        backend_canister,
        spender.owner,
        "icrc1_transfer_from",
        encode_args((TransferFromArgs {
            spender: spender.clone(),
            from: owner.clone(),
            to: to.clone(),
            amount: amount.clone(),
        },))
        .unwrap(),
    )
    .expect("Failed to transfer from");

    let to_balance: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_balance_of",
            encode_args((to,)).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        to_balance,
        amount - transfer_fee,
        "Transferred amount should be received minus fee"
    );
}

#[test]
fn test_arithmetic_overflow() {
    let (pic, backend_canister, admin) = setup();
    let from = Account {
        owner: admin,
        subaccount: Some(COMMUNITY_POOL_SUBACCOUNT),
    };
    let to = Account {
        owner: Principal::from_text("2vxsx-fae").unwrap(),
        subaccount: None,
    };
    let amount = Nat::from(u128::MAX);

    let args = encode_args(((TransferArgs {
        from_subaccount: from.subaccount,
        to,
        amount,
    }),))
    .expect("Failed to encode args");
    let response = pic.update_call(backend_canister, admin, "icrc1_transfer", args);

    match response {
        Ok(resp) => {
            let result: Result<Nat, LedgerError> =
                decode_one(&resp).expect("Failed to decode response");
            assert!(
                result.is_err(),
                "Transfer with overflow amount should return an error"
            );
            if let Err(e) = result {
                assert_eq!(
                    e,
                    LedgerError::ArithmeticError,
                    "Error should be ArithmeticError"
                );
            }
        }
        Err(e) => {
            println!("Update call failed with: {:?}", e);
            assert!(true, "Transfer with overflow amount failed as expected");
        }
    }
}

#[test]
fn test_concurrent_transfers() {
    let (pic, backend_canister, admin) = setup();
    let from = Account {
        owner: admin,
        subaccount: Some(COMMUNITY_POOL_SUBACCOUNT),
    };
    let to1 = Account {
        owner: Principal::from_text("2vxsx-fae").unwrap(),
        subaccount: None,
    };
    let to2 = Account {
        owner: Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap(),
        subaccount: None,
    };
    let amount = Nat::from(1_000_000_000_u128);
    let transfer_fee = Nat::from(100_000_u128);

    pic.update_call(
        backend_canister,
        admin,
        "icrc1_transfer",
        encode_args(((TransferArgs {
            from_subaccount: from.subaccount,
            to: to1.clone(),
            amount: amount.clone(), // Clone here to avoid move in first transfer
        }),))
        .unwrap(),
    )
    .expect("Transfer 1 failed");
    pic.update_call(
        backend_canister,
        admin,
        "icrc1_transfer",
        encode_args(((TransferArgs {
            from_subaccount: from.subaccount,
            to: to2.clone(),
            amount: amount.clone(), // Clone here to avoid move in second transfer
        }),))
        .unwrap(),
    )
    .expect("Transfer 2 failed");

    let to1_balance: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_balance_of",
            encode_args((to1,)).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let to2_balance: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_balance_of",
            encode_args((to2,)).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(
        to1_balance,
        amount.clone() - transfer_fee.clone(), // Clone here to avoid move
        "First recipient should receive amount minus fee"
    );
    assert_eq!(
        to2_balance,
        amount - transfer_fee,
        "Second recipient should receive amount minus fee"
    );
}

#[test]
fn test_boundary_values() {
    let (pic, backend_canister, admin) = setup();
    let from = Account {
        owner: admin,
        subaccount: Some(COMMUNITY_POOL_SUBACCOUNT),
    };
    let to = Account {
        owner: Principal::from_text("2vxsx-fae").unwrap(),
        subaccount: None,
    };

    let response_zero = pic
        .update_call(
            backend_canister,
            admin,
            "icrc1_transfer",
            encode_args(((TransferArgs {
                from_subaccount: from.subaccount,
                to: to.clone(),
                amount: Nat::from(0_u128),
            }),))
            .unwrap(),
        )
        .expect("Failed to call icrc1_transfer with zero amount");
    let result_zero: Result<Nat, LedgerError> =
        decode_one(&response_zero).expect("Failed to decode zero response");
    assert!(
        result_zero.is_err(),
        "Transfer with zero amount should fail due to fee"
    );
    if let Err(e) = result_zero {
        assert_eq!(
            e,
            LedgerError::InsufficientFee,
            "Error should be InsufficientFee"
        );
    }

    let response_max = pic
        .update_call(
            backend_canister,
            admin,
            "icrc1_transfer",
            encode_args(((TransferArgs {
                from_subaccount: from.subaccount,
                to: to.clone(),
                amount: Nat::from(u128::MAX),
            }),))
            .unwrap(),
        )
        .expect("Failed to call icrc1_transfer with max amount");
    let result_max: Result<Nat, LedgerError> =
        decode_one(&response_max).expect("Failed to decode max response");
    assert!(
        result_max.is_err(),
        "Transfer with maximum amount should fail"
    );
    if let Err(e) = result_max {
        assert_eq!(
            e,
            LedgerError::ArithmeticError,
            "Error should be ArithmeticError"
        );
    }
}

#[test]
fn test_process_fee() {
    let (pic, backend_canister, admin) = setup();
    let from = Account {
        owner: admin,
        subaccount: Some(RESERVE_POOL_SUBACCOUNT),
    };
    let to = Account {
        owner: Principal::from_text("2vxsx-fae").unwrap(),
        subaccount: None,
    };
    let amount = Nat::from(1_000_000_000_u128); // 10 CRNL
    let transfer_fee = Nat::from(100_000_u128); // 0.001 CRNL

    let initial_total_supply: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_total_supply",
            encode_args(()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    let initial_total_burned: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "get_total_burned",
            encode_args(()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    let initial_community_pool: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "get_community_pool_balance",
            encode_args(()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    let initial_dapp_funds: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "get_dapp_funds",
            encode_args(()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    let initial_from_balance: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_balance_of",
            encode_args((from.clone(),)).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    let transfer_result: Result<Nat, LedgerError> = decode_one(
        &pic.update_call(
            backend_canister,
            admin,
            "icrc1_transfer",
            encode_args(((TransferArgs {
                from_subaccount: from.subaccount,
                to: to.clone(),
                amount: amount.clone(),
            }),))
            .unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(transfer_result.is_ok(), "Transfer should succeed");

    let burn_amount = Nat::from(transfer_fee.clone() * Nat::from(20_u128) / Nat::from(100_u128));
    let pool_amount = Nat::from(transfer_fee.clone() * Nat::from(10_u128) / Nat::from(100_u128));
    let dapp_amount = Nat::from(transfer_fee.clone() * Nat::from(70_u128) / Nat::from(100_u128));
    let amount_after_fee = amount.clone() - transfer_fee.clone();

    let final_total_supply: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_total_supply",
            encode_args(()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    let final_total_burned: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "get_total_burned",
            encode_args(()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    let final_community_pool: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "get_community_pool_balance",
            encode_args(()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    let final_dapp_funds: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "get_dapp_funds",
            encode_args(()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    let final_from_balance: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_balance_of",
            encode_args((from.clone(),)).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    let final_to_balance: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_balance_of",
            encode_args((to.clone(),)).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(
        final_total_supply,
        initial_total_supply - burn_amount.clone(),
        "Total supply should decrease by burn amount"
    );
    assert_eq!(
        final_total_burned,
        initial_total_burned + burn_amount,
        "Total burned should increase by burn amount"
    );
    assert_eq!(
        final_community_pool,
        initial_community_pool + pool_amount.clone(),
        "Community pool should increase by pool amount"
    );
    assert_eq!(
        final_dapp_funds,
        initial_dapp_funds + dapp_amount,
        "Dapp funds should increase by dapp amount"
    );
    assert_eq!(
        final_from_balance,
        initial_from_balance - amount,
        "Sender balance should decrease by amount only"
    );
    assert_eq!(
        final_to_balance, amount_after_fee,
        "Recipient balance should equal amount minus fee"
    );
}

#[test]
fn test_icrc1_and_icrc2_compliance() {
    let (pic, backend_canister, admin) = setup();
    let user = Account {
        owner: create_mock_ii_principal(2),
        subaccount: None,
    };
    let spender = Account {
        owner: Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap(),
        subaccount: None,
    };

    // ICRC-1 Compliance Checks
    let supported_standards: Vec<String> = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_supported_standards",
            encode_args(()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(
        supported_standards.contains(&"ICRC-1".to_string()),
        "Canister must support ICRC-1"
    );
    assert!(
        supported_standards.contains(&"ICRC-2".to_string()),
        "Canister must support ICRC-2"
    );

    let name: String = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_name",
            encode_args(()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(name, "Chronolock", "ICRC-1 name must match");

    let symbol: String = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_symbol",
            encode_args(()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(symbol, "CRNL", "ICRC-1 symbol must match");

    let decimals: u8 = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_decimals",
            encode_args(()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(decimals, 8, "ICRC-1 decimals must match");

    let total_supply: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_total_supply",
            encode_args(()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        total_supply,
        Nat::from(100_000_000_000_000_000_000_u128),
        "ICRC-1 total supply must match"
    );

    let fee: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_fee",
            encode_args(()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(fee, Nat::from(100_000_u128), "ICRC-1 fee must match");

    // Register a user to give them some tokens
    pic.update_call(
        backend_canister,
        user.owner, // Use the actual user's principal for authentication
        "register_user",
        encode_args((user.clone(),)).unwrap(),
    )
    .expect("Failed to register user");

    let user_balance: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_balance_of",
            encode_args((user.clone(),)).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        user_balance,
        Nat::from(20_000_000_000_u128),
        "ICRC-1 balance_of should reflect registration"
    );

    let transfer_amount = Nat::from(1_000_000_000_u128);
    let transfer_fee = Nat::from(100_000_u128);
    let transfer_result: Result<Nat, LedgerError> = decode_one(
        &pic.update_call(
            backend_canister,
            user.owner,
            "icrc1_transfer",
            encode_args((TransferArgs {
                from_subaccount: user.subaccount,
                to: spender.clone(),
                amount: transfer_amount.clone(),
            },))
            .unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(transfer_result.is_ok(), "ICRC-1 transfer should succeed");

    let spender_balance: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_balance_of",
            encode_args((spender.clone(),)).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        spender_balance,
        transfer_amount - transfer_fee,
        "ICRC-1 transfer should deduct fee"
    );

    // ICRC-2 Compliance Checks
    let approve_amount = Nat::from(1_000_000_000_u128);
    let approve_result: Result<Nat, LedgerError> = decode_one(
        &pic.update_call(
            backend_canister,
            user.owner,
            "icrc1_approve",
            encode_args((ApproveArgs {
                from_subaccount: spender.subaccount,
                spender: spender.clone(),
                amount: approve_amount.clone(),
            },))
            .unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(approve_result.is_ok(), "ICRC-2 approve should succeed");

    let allowance: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_allowance",
            encode_args((user.clone(), spender.clone())).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        allowance, approve_amount,
        "ICRC-2 allowance should reflect approval"
    );

    let transfer_from_amount = Nat::from(500_000_000_u128);
    let transfer_from_result: Result<Nat, LedgerError> = decode_one(
        &pic.update_call(
            backend_canister,
            spender.owner,
            "icrc1_transfer_from",
            encode_args((TransferFromArgs {
                spender: spender.clone(),
                from: user.clone(),
                to: Account {
                    owner: admin,
                    subaccount: None,
                },
                amount: transfer_from_amount.clone(),
            },))
            .unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(
        transfer_from_result.is_ok(),
        "ICRC-2 transfer_from should succeed"
    );

    let updated_allowance: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_allowance",
            encode_args((user.clone(), spender.clone())).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        updated_allowance,
        approve_amount - transfer_from_amount,
        "ICRC-2 allowance should decrease after transfer_from"
    );

    let metadata: Vec<(String, String)> = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_metadata",
            encode_args(()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(
        metadata
            .iter()
            .any(|(k, v)| k == "icrc1:name" && v == "Chronolock"),
        "Metadata should include name"
    );
    assert!(
        metadata
            .iter()
            .any(|(k, v)| k == "icrc1:symbol" && v == "CRNL"),
        "Metadata should include symbol"
    );
}

#[test]
fn test_transaction_recording() {
    let (pic, backend_canister, admin) = setup();

    let from = Account {
        owner: admin,
        subaccount: Some(COMMUNITY_POOL_SUBACCOUNT),
    };

    let to = Account {
        owner: Principal::from_text("2vxsx-fae").unwrap(),
        subaccount: None,
    };

    // Perform a transfer
    let transfer_args = TransferArgs {
        from_subaccount: from.subaccount,
        to: to.clone(),
        amount: Nat::from(1_000_000_000_u128), // 10 CRNL
    };

    let transfer_response = pic
        .update_call(
            backend_canister,
            admin,
            "icrc1_transfer",
            encode_args((transfer_args,)).unwrap(),
        )
        .expect("Failed to call icrc1_transfer");

    let transfer_result: Result<Nat, LedgerError> = decode_one(&transfer_response).unwrap();
    assert!(transfer_result.is_ok());

    let start_tx_id: [u8; 32] = [0; 32];
    let limit: u64 = 10;

    let query_args = encode_args((from.owner, start_tx_id, limit))
        .expect("Failed to encode args for transaction query");

    let response = pic
        .query_call(
            backend_canister,
            Principal::anonymous(),
            "get_transactions_by_principal",
            query_args,
        )
        .expect("Failed to query transactions");

    let transactions: Vec<TransactionEvent> =
        decode_one(&response).expect("Failed to decode transactions");

    assert!(
        !transactions.is_empty(),
        "Transaction history should not be empty"
    );

    let tx = &transactions[0];
    assert_eq!(tx.event_type, "Transfer");
    assert_eq!(tx.from.owner, admin);
    assert_eq!(
        tx.to.as_ref().unwrap().owner,
        Principal::from_text("2vxsx-fae").unwrap()
    );
    assert_eq!(tx.amount, Nat::from(1_000_000_000_u128));
    assert_eq!(
        tx.tx_id.len(),
        32,
        "tx_id should be a SHA-256 hash of 32 bytes"
    );
}

#[test]
fn test_pool_to_pool_transfer() {
    let (pic, backend_canister, admin) = setup();

    // Get initial balances
    let community_balance_before: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "get_community_pool_balance",
            encode_args(()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    let reserve_balance_before: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "get_reserve_pool_balance",
            encode_args(()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    println!("Community balance before: {}", community_balance_before);
    println!("Reserve balance before: {}", reserve_balance_before);

    // Transfer 1000 CRNL from reserve to community pool
    let transfer_amount = Nat::from(100_000_000_000_u128); // 1000 CRNL
    let args = encode_args((PoolTransferArgs {
        from_pool: "reserve".to_string(),
        to_pool: Some("community".to_string()),
        to_principal: None,
        amount: transfer_amount.clone(),
        description: "Pool rebalancing".to_string(),
    },))
    .expect("Failed to encode args");

    let response = pic
        .update_call(backend_canister, admin, "admin_transfer", args)
        .expect("Failed to call admin_transfer");
    let result: Result<String, LedgerError> = decode_one(&response).unwrap();
    assert!(result.is_ok(), "Transfer should succeed: {:?}", result);

    // Verify balances changed correctly
    let community_balance_after: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "get_community_pool_balance",
            encode_args(()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    let reserve_balance_after: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "get_reserve_pool_balance",
            encode_args(()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    println!("Community balance after: {}", community_balance_after);
    println!("Reserve balance after: {}", reserve_balance_after);

    assert_eq!(
        community_balance_after,
        community_balance_before.clone() + transfer_amount.clone()
    );
    assert_eq!(
        reserve_balance_after,
        reserve_balance_before.clone() - transfer_amount.clone()
    );
}

#[test]
fn test_pool_to_principal_transfer() {
    let (pic, backend_canister, admin) = setup();

    // Create a recipient account
    let recipient = Account {
        owner: create_mock_ii_principal(5),
        subaccount: None,
    };

    // Get initial balances
    let reserve_balance_before: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "get_reserve_pool_balance",
            encode_args(()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    let recipient_balance_before: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_balance_of",
            encode_args((recipient.clone(),)).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    println!("Reserve balance before: {}", reserve_balance_before);
    println!("Recipient balance before: {}", recipient_balance_before);

    // Transfer 500 CRNL from reserve to recipient
    let transfer_amount = Nat::from(50_000_000_000_u128); // 500 CRNL
    let args = encode_args((PoolTransferArgs {
        from_pool: "reserve".to_string(),
        to_pool: None,
        to_principal: Some(recipient.clone()),
        amount: transfer_amount.clone(),
        description: "Grant payment".to_string(),
    },))
    .expect("Failed to encode args");

    let response = pic
        .update_call(backend_canister, admin, "admin_transfer", args)
        .expect("Failed to call admin_transfer");
    let result: Result<String, LedgerError> = decode_one(&response).unwrap();
    assert!(result.is_ok(), "Transfer should succeed: {:?}", result);

    // Verify balances changed correctly
    let reserve_balance_after: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "get_reserve_pool_balance",
            encode_args(()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    let recipient_balance_after: Nat = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_balance_of",
            encode_args((recipient.clone(),)).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    println!("Reserve balance after: {}", reserve_balance_after);
    println!("Recipient balance after: {}", recipient_balance_after);

    assert_eq!(
        reserve_balance_after,
        reserve_balance_before.clone() - transfer_amount.clone()
    );
    assert_eq!(
        recipient_balance_after,
        recipient_balance_before.clone() + transfer_amount.clone()
    );
}

#[test]
fn test_pool_transfer_insufficient_balance() {
    let (pic, backend_canister, admin) = setup();

    // Try to transfer more than the pool has
    let excessive_amount = Nat::from(100_000_000_000_000_000_000_u128); // More than any pool
    let args = encode_args((PoolTransferArgs {
        from_pool: "community".to_string(),
        to_pool: Some("reserve".to_string()),
        to_principal: None,
        amount: excessive_amount,
        description: "Should fail".to_string(),
    },))
    .expect("Failed to encode args");

    let response = pic
        .update_call(backend_canister, admin, "admin_transfer", args)
        .expect("Failed to call admin_transfer");
    let result: Result<String, LedgerError> = decode_one(&response).unwrap();
    assert!(
        matches!(result, Err(LedgerError::InsufficientBalance)),
        "Should fail with insufficient balance"
    );
}

#[test]
fn test_pool_transfer_authorization() {
    let (pic, backend_canister, _admin) = setup();
    let non_admin = create_mock_ii_principal(10);

    // Try to transfer as non-admin
    let transfer_amount = Nat::from(1_000_000_000_u128); // 10 CRNL
    let args = encode_args((PoolTransferArgs {
        from_pool: "reserve".to_string(),
        to_pool: Some("community".to_string()),
        to_principal: None,
        amount: transfer_amount,
        description: "Unauthorized attempt".to_string(),
    },))
    .expect("Failed to encode args");

    let response = pic
        .update_call(backend_canister, non_admin, "admin_transfer", args)
        .expect("Failed to call admin_transfer");
    let result: Result<String, LedgerError> = decode_one(&response).unwrap();
    assert!(
        matches!(
            result,
            Err(LedgerError::AdminRequired) | Err(LedgerError::NotAuthenticated)
        ),
        "Non-admin should not be able to transfer from pools: {:?}",
        result
    );
}

#[test]
fn test_pool_transfer_invalid_pool_name() {
    let (pic, backend_canister, admin) = setup();

    // Try to transfer from invalid pool
    let transfer_amount = Nat::from(1_000_000_000_u128);
    let args = encode_args((PoolTransferArgs {
        from_pool: "invalid_pool".to_string(),
        to_pool: Some("community".to_string()),
        to_principal: None,
        amount: transfer_amount,
        description: "Invalid pool".to_string(),
    },))
    .expect("Failed to encode args");

    let response = pic
        .update_call(backend_canister, admin, "admin_transfer", args)
        .expect("Failed to call admin_transfer");
    let result: Result<String, LedgerError> = decode_one(&response).unwrap();
    assert!(
        matches!(result, Err(LedgerError::InvalidAccount)),
        "Should fail with invalid account"
    );
}

#[test]
fn test_pool_transfer_all_pool_combinations() {
    let (pic, backend_canister, admin) = setup();

    let pools = vec!["community", "reserve", "dapp"];
    let transfer_amount = Nat::from(1_000_000_000_u128); // 10 CRNL

    for from_pool in &pools {
        for to_pool in &pools {
            if from_pool == to_pool {
                continue; // Skip same pool transfers
            }

            println!("Testing transfer from {} to {}", from_pool, to_pool);

            let args = encode_args((PoolTransferArgs {
                from_pool: from_pool.to_string(),
                to_pool: Some(to_pool.to_string()),
                to_principal: None,
                amount: transfer_amount.clone(),
                description: format!("Test {} to {}", from_pool, to_pool),
            },))
            .expect("Failed to encode args");

            let response = pic
                .update_call(backend_canister, admin, "admin_transfer", args)
                .expect("Failed to call admin_transfer");
            let result: Result<String, LedgerError> = decode_one(&response).unwrap();
            assert!(
                result.is_ok(),
                "Transfer from {} to {} should succeed: {:?}",
                from_pool,
                to_pool,
                result
            );
        }
    }
}
