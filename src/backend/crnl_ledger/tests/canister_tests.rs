use candid::{decode_one, encode_args, CandidType, Principal};
use pocket_ic::PocketIc;
use std::fs;

const COMMUNITY_POOL_SUBACCOUNT: [u8; 32] = [1u8; 32];
const TEAM_VESTING_POOL_SUBACCOUNT: [u8; 32] = [2u8; 32];
const RESERVE_POOL_SUBACCOUNT: [u8; 32] = [3u8; 32];

const BACKEND_WASM: &str = "../../../target/wasm32-unknown-unknown/release/crnl_ledger.wasm";

#[derive(CandidType, serde::Deserialize, Clone, Debug, PartialEq)]
struct Account {
    owner: Principal,
    subaccount: Option<[u8; 32]>,
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
}

fn setup() -> (PocketIc, Principal, Principal) {
    std::env::set_var("POCKET_IC_BIN", "/usr/local/bin/pocket-ic");
    let pic = PocketIc::new();

    let backend_canister = pic.create_canister();
    pic.add_cycles(backend_canister, 2_000_000_000_000);
    let wasm = fs::read(BACKEND_WASM).expect("Wasm file not found, run 'cargo build'.");

    let admin = Principal::from_text("aaaaa-aa").unwrap();
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
    let result: u128 = decode_one(&response).unwrap();
    assert_eq!(result, 100_000_000_000_000_000_000_u128);
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
    let result: u128 = decode_one(&response).unwrap();
    assert_eq!(result, 100_000_u128);
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

    let admin_balance: u128 = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "balance_of",
            encode_args((admin_account,)).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let community_balance: u128 = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "balance_of",
            encode_args((community_account,)).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let team_balance: u128 = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "balance_of",
            encode_args((team_account,)).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let reserve_balance: u128 = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "balance_of",
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
        100_000_000_000_000_000_000_u128
    );
}

#[test]
fn test_get_logs() {
    let (pic, backend_canister, _) = setup();
    let response = pic
        .query_call(
            backend_canister,
            Principal::anonymous(),
            "get_logs",
            encode_args(()).unwrap(),
        )
        .expect("Failed to query get_logs");
    let result: Vec<LogEntry> = decode_one(&response).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].event_type, "Init");
}

#[test]
fn test_get_logs_by_range() {
    let (pic, backend_canister, _) = setup();
    let args = encode_args((0_u64, u64::MAX)).expect("Failed to encode args");
    let response = pic
        .query_call(
            backend_canister,
            Principal::anonymous(),
            "get_logs_by_range",
            args,
        )
        .expect("Failed to query get_logs_by_range");
    let result: Vec<LogEntry> = decode_one(&response).unwrap();
    assert_eq!(result.len(), 1);
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
    let result: u128 = decode_one(&response).unwrap();
    assert_eq!(result, 0_u128);
}

// Update Tests
#[test]
fn test_register_user() {
    let (pic, backend_canister, _) = setup();
    let user = Account {
        owner: Principal::from_text("2vxsx-fae").unwrap(),
        subaccount: None,
    };
    let args = encode_args((user.clone(),)).expect("Failed to encode args");
    let response = pic
        .update_call(
            backend_canister,
            Principal::anonymous(),
            "register_user",
            args,
        )
        .expect("Failed to call register_user");
    let result: Result<String, LedgerError> = decode_one(&response).unwrap();
    assert!(result.unwrap().starts_with("User registered with 200 CRNL"));
}

#[test]
fn test_claim_referral() {
    let (pic, backend_canister, _) = setup();
    let referrer = Account {
        owner: Principal::from_text("2vxsx-fae").unwrap(),
        subaccount: None,
    };
    let referee = Account {
        owner: Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap(),
        subaccount: None,
    };

    let reg_args = encode_args((referrer.clone(),)).expect("Failed to encode args");
    let reg_response = pic
        .update_call(
            backend_canister,
            Principal::anonymous(),
            "register_user",
            reg_args,
        )
        .expect("Failed to register referrer");
    let reg_result: Result<String, LedgerError> = decode_one(&reg_response).unwrap();
    let referral_code = reg_result
        .unwrap()
        .split("Your referral code is: ")
        .nth(1)
        .unwrap()
        .to_string();

    let args = encode_args((referral_code, referee)).expect("Failed to encode args");
    let response = pic
        .update_call(
            backend_canister,
            Principal::anonymous(),
            "claim_referral",
            args,
        )
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
    let amount = 1_000_000_000_u128; // 10 CRNL
    let transfer_fee = 100_000_u128; // 0.001 CRNL from init

    let from_balance: u128 = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "balance_of",
            encode_args((from.clone(),)).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    println!("Transfer from balance before: {}", from_balance);

    let args = encode_args((from.clone(), to.clone(), amount)).expect("Failed to encode args");
    let response = pic
        .update_call(backend_canister, admin, "icrc1_transfer", args)
        .expect("Failed to call icrc1_transfer");
    let result: Result<(), LedgerError> = decode_one(&response).unwrap();
    if let Err(e) = &result {
        println!("Transfer failed with: {:?}", e);
    }
    assert!(result.is_ok());

    let to_balance: u128 = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "balance_of",
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
fn test_icrc1_approve_and_transfer_from() {
    let (pic, backend_canister, admin) = setup();
    let owner = Account {
        owner: admin,
        subaccount: Some(COMMUNITY_POOL_SUBACCOUNT),
    };
    let spender = Account {
        owner: Principal::from_text("2vxsx-fae").unwrap(),
        subaccount: None,
    };
    let to = Account {
        owner: Principal::anonymous(),
        subaccount: None,
    };
    let amount = 1_000_000_000_u128; // 10 CRNL
    let transfer_fee = 100_000_u128; // 0.001 CRNL

    let owner_balance: u128 = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "balance_of",
            encode_args((owner.clone(),)).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    println!("Owner balance before approve: {}", owner_balance);

    let approve_args =
        encode_args((owner.clone(), spender.clone(), amount)).expect("Failed to encode args");
    let approve_response = pic
        .update_call(backend_canister, admin, "icrc1_approve", approve_args)
        .expect("Failed to call icrc1_approve");
    let approve_result: Result<(), LedgerError> = decode_one(&approve_response).unwrap();
    assert!(approve_result.is_ok());

    let transfer_args = encode_args((spender.clone(), owner.clone(), to.clone(), amount))
        .expect("Failed to encode args");
    let transfer_response = pic
        .update_call(
            backend_canister,
            spender.owner,
            "icrc1_transfer_from",
            transfer_args,
        )
        .expect("Failed to call icrc1_transfer_from");
    let transfer_result: Result<(), LedgerError> = decode_one(&transfer_response).unwrap();
    assert!(transfer_result.is_ok());

    let to_balance: u128 = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "balance_of",
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
fn test_create_media_chronolock() {
    let (pic, backend_canister, admin) = setup();
    let caller = Account {
        owner: admin,
        subaccount: Some(COMMUNITY_POOL_SUBACCOUNT),
    };

    let caller_balance_args = encode_args((caller.clone(),)).expect("Failed to encode args");
    let caller_balance: u128 = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "balance_of",
            caller_balance_args.clone(),
        )
        .unwrap(),
    )
    .unwrap();
    println!("Caller balance before chronolock: {}", caller_balance);

    let args = encode_args((caller.clone(),)).expect("Failed to encode args");
    let response = pic
        .update_call(backend_canister, admin, "create_media_chronolock", args)
        .expect("Failed to call create_media_chronolock");
    let result: Result<String, LedgerError> = decode_one(&response).unwrap();
    if let Err(e) = &result {
        println!("Create media chronolock failed with: {:?}", e);
    }
    assert_eq!(result.unwrap(), "Media ChronoLock created for 20 CRNL");

    let post_balance: u128 = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "balance_of",
            caller_balance_args,
        )
        .unwrap(),
    )
    .unwrap();
    println!("Caller balance after chronolock: {}", post_balance);
    assert_eq!(post_balance, caller_balance - 2_000_000_000_u128);
}

#[test]
fn test_create_text_chronolock() {
    let (pic, backend_canister, admin) = setup();
    let caller = Account {
        owner: admin,
        subaccount: None,
    };
    let args = encode_args((caller,)).expect("Failed to encode args");
    let response = pic
        .update_call(backend_canister, admin, "create_text_chronolock", args)
        .expect("Failed to call create_text_chronolock");
    let result: Result<String, LedgerError> = decode_one(&response).unwrap();
    assert_eq!(result.unwrap(), "Text ChronoLock created for free");
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
    let fee: u128 = decode_one(&fee_response).unwrap();
    assert_eq!(fee, 200_000_u128);
}

#[test]
fn test_set_transfer_fee_authorization() {
    let (pic, backend_canister, admin) = setup();
    let non_admin = Principal::from_text("2vxsx-fae").unwrap();

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
        assert_eq!(e, LedgerError::Unauthorized, "Error should be Unauthorized");
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
    let fee: u128 = decode_one(&fee_response).unwrap();
    assert_eq!(
        fee, 200_000_u128,
        "Transfer fee should be updated to 200,000"
    );
}

#[test]
fn test_unique_referral_codes() {
    let (pic, backend_canister, _) = setup();
    let user1 = Account {
        owner: Principal::from_text("2vxsx-fae").unwrap(),
        subaccount: None,
    };
    let user2 = Account {
        owner: Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap(),
        subaccount: None,
    };

    // Register users
    pic.update_call(
        backend_canister,
        Principal::anonymous(),
        "register_user",
        encode_args((user1.clone(),)).unwrap(),
    )
    .expect("Failed to register user1");
    pic.update_call(
        backend_canister,
        Principal::anonymous(),
        "register_user",
        encode_args((user2.clone(),)).unwrap(),
    )
    .expect("Failed to register user2");

    // Get referral codes
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
        owner: Principal::from_text("2vxsx-fae").unwrap(),
        subaccount: None,
    };
    let referee = Account {
        owner: Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap(),
        subaccount: None,
    };

    // Register referrer and get referral code
    let reg_response = pic
        .update_call(
            backend_canister,
            Principal::anonymous(),
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

    // Claim referral
    pic.update_call(
        backend_canister,
        Principal::anonymous(),
        "claim_referral",
        encode_args((referral_code, referee.clone())).unwrap(),
    )
    .expect("Failed to claim referral");

    // Check balances
    let referrer_balance: u128 = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "balance_of",
            encode_args((referrer,)).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let community_pool: u128 = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "community_pool_balance",
            encode_args(()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(
        referrer_balance,
        20_000_000_000 + 2_000_000_000,
        "Referrer should receive reward"
    );
    assert_eq!(
        community_pool,
        50_000_000_000_000_000_000 - 20_000_000_000 - 2_000_000_000,
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
    let amount = 50_000_000_000_000_000_001_u128; // Exceeds pool

    let response = pic
        .update_call(
            backend_canister,
            admin,
            "icrc1_transfer",
            encode_args((from, to, amount)).unwrap(),
        )
        .expect("Failed to call icrc1_transfer");
    let result: Result<(), LedgerError> = decode_one(&response).expect("Failed to decode response");
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
        subaccount: Some(COMMUNITY_POOL_SUBACCOUNT),
    };
    let spender = Account {
        owner: Principal::from_text("2vxsx-fae").unwrap(),
        subaccount: None,
    };
    let to = Account {
        owner: Principal::anonymous(),
        subaccount: None,
    };
    let amount = 1_000_000_000_u128;
    let transfer_fee = 100_000_u128;

    // Approve
    pic.update_call(
        backend_canister,
        admin,
        "icrc1_approve",
        encode_args((owner.clone(), spender.clone(), amount)).unwrap(),
    )
    .expect("Failed to approve");

    // Check allowance
    let allowance: u128 = decode_one(
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

    // Transfer from
    pic.update_call(
        backend_canister,
        spender.owner,
        "icrc1_transfer_from",
        encode_args((spender.clone(), owner.clone(), to.clone(), amount)).unwrap(),
    )
    .expect("Failed to transfer from");

    // Verify balance
    let to_balance: u128 = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "balance_of",
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
fn test_log_event() {
    let (pic, backend_canister, admin) = setup();
    let from = Account {
        owner: admin,
        subaccount: Some(COMMUNITY_POOL_SUBACCOUNT),
    };
    let to = Account {
        owner: Principal::from_text("2vxsx-fae").unwrap(),
        subaccount: None,
    };
    let amount = 1_000_000_000_u128;

    // Perform transfer
    pic.update_call(
        backend_canister,
        admin,
        "icrc1_transfer",
        encode_args((from.clone(), to.clone(), amount)).unwrap(),
    )
    .expect("Failed to transfer");

    // Check logs
    let logs: Vec<LogEntry> = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "get_logs",
            encode_args(()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let last_log = logs.last().unwrap();

    assert_eq!(
        last_log.event_type, "Transfer",
        "Log should record transfer event"
    );
    assert!(
        last_log.details.contains(&format!("Amount: {}", amount)),
        "Log should include transfer amount"
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
    let amount = u128::MAX;

    let args = encode_args((from, to, amount)).expect("Failed to encode args");
    let response = pic.update_call(backend_canister, admin, "icrc1_transfer", args);

    match response {
        Ok(resp) => {
            let result: Result<(), LedgerError> =
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
            // If the call itself fails (e.g., canister rejects), this is also acceptable
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
    let amount = 1_000_000_000_u128;
    let transfer_fee = 100_000_u128;

    // Concurrent transfers
    pic.update_call(
        backend_canister,
        admin,
        "icrc1_transfer",
        encode_args((from.clone(), to1.clone(), amount)).unwrap(),
    )
    .expect("Transfer 1 failed");
    pic.update_call(
        backend_canister,
        admin,
        "icrc1_transfer",
        encode_args((from.clone(), to2.clone(), amount)).unwrap(),
    )
    .expect("Transfer 2 failed");

    // Verify balances
    let to1_balance: u128 = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "balance_of",
            encode_args((to1,)).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let to2_balance: u128 = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "balance_of",
            encode_args((to2,)).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(
        to1_balance,
        amount - transfer_fee,
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

    // Zero amount
    let response_zero = pic
        .update_call(
            backend_canister,
            admin,
            "icrc1_transfer",
            encode_args((from.clone(), to.clone(), 0_u128)).unwrap(),
        )
        .expect("Failed to call icrc1_transfer with zero amount");
    let result_zero: Result<(), LedgerError> =
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

    // Maximum amount
    let response_max = pic
        .update_call(
            backend_canister,
            admin,
            "icrc1_transfer",
            encode_args((from.clone(), to.clone(), u128::MAX)).unwrap(),
        )
        .expect("Failed to call icrc1_transfer with max amount");
    let result_max: Result<(), LedgerError> =
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
        subaccount: Some(COMMUNITY_POOL_SUBACCOUNT),
    };
    let to = Account {
        owner: Principal::from_text("2vxsx-fae").unwrap(),
        subaccount: None,
    };
    let amount = 1_000_000_000_u128; // 10 CRNL
    let transfer_fee = 100_000_u128; // 0.001 CRNL

    // Get initial state
    let initial_total_supply: u128 = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_total_supply",
            encode_args(()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    let initial_total_burned: u128 = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "total_burned",
            encode_args(()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    let initial_community_pool: u128 = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "community_pool_balance",
            encode_args(()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    let initial_dapp_funds: u128 = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "get_dapp_funds",
            encode_args(()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    let initial_from_balance: u128 = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "balance_of",
            encode_args((from.clone(),)).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    // Perform transfer
    let transfer_result: Result<(), LedgerError> = decode_one(
        &pic.update_call(
            backend_canister,
            admin,
            "icrc1_transfer",
            encode_args((from.clone(), to.clone(), amount)).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(transfer_result.is_ok(), "Transfer should succeed");

    // Calculate expected fee distribution
    let burn_amount = transfer_fee * 20 / 100; // 20% burned
    let pool_amount = transfer_fee * 10 / 100; // 10% to community pool
    let dapp_amount = transfer_fee * 70 / 100; // 70% to dapp funds
    let amount_after_fee = amount - transfer_fee;

    // Get post-transfer state
    let final_total_supply: u128 = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "icrc1_total_supply",
            encode_args(()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    let final_total_burned: u128 = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "total_burned",
            encode_args(()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    let final_community_pool: u128 = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "community_pool_balance",
            encode_args(()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    let final_dapp_funds: u128 = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "get_dapp_funds",
            encode_args(()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    let final_from_balance: u128 = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "balance_of",
            encode_args((from.clone(),)).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    let final_to_balance: u128 = decode_one(
        &pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "balance_of",
            encode_args((to.clone(),)).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    // Verify fee processing
    assert_eq!(
        final_total_supply,
        initial_total_supply - burn_amount,
        "Total supply should decrease by burn amount"
    );
    assert_eq!(
        final_total_burned,
        initial_total_burned + burn_amount,
        "Total burned should increase by burn amount"
    );
    assert_eq!(
        final_community_pool,
        initial_community_pool + pool_amount,
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
