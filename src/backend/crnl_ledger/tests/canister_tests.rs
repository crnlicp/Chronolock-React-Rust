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
    )).expect("Failed to encode init arguments");

    pic.install_canister(backend_canister, wasm, init_args, None);
    (pic, backend_canister, admin)
}


// Query Tests
#[test]
fn test_icrc1_name() {
    let (pic, backend_canister, _) = setup();
    let response = pic.query_call(backend_canister, Principal::anonymous(), "icrc1_name", encode_args(()).unwrap()).expect("Failed to query icrc1_name");
    let result: String = decode_one(&response).unwrap();
    assert_eq!(result, "Chronolock");
}

#[test]
fn test_icrc1_symbol() {
    let (pic, backend_canister, _) = setup();
    let response = pic.query_call(backend_canister, Principal::anonymous(), "icrc1_symbol", encode_args(()).unwrap()).expect("Failed to query icrc1_symbol");
    let result: String = decode_one(&response).unwrap();
    assert_eq!(result, "CRNL");
}

#[test]
fn test_icrc1_decimals() {
    let (pic, backend_canister, _) = setup();
    let response = pic.query_call(backend_canister, Principal::anonymous(), "icrc1_decimals", encode_args(()).unwrap()).expect("Failed to query icrc1_decimals");
    let result: u8 = decode_one(&response).unwrap();
    assert_eq!(result, 8);
}

#[test]
fn test_icrc1_total_supply() {
    let (pic, backend_canister, _) = setup();
    let response = pic.query_call(backend_canister, Principal::anonymous(), "icrc1_total_supply", encode_args(()).unwrap()).expect("Failed to query icrc1_total_supply");
    let result: u128 = decode_one(&response).unwrap();
    assert_eq!(result, 100_000_000_000_000_000_000_u128);
}

#[test]
fn test_icrc1_fee() {
    let (pic, backend_canister, _) = setup();
    let response = pic.query_call(backend_canister, Principal::anonymous(), "icrc1_fee", encode_args(()).unwrap()).expect("Failed to query icrc1_fee");
    let result: u128 = decode_one(&response).unwrap();
    assert_eq!(result, 100_000_u128);
}

#[test]
fn test_balance_of() {
    let (pic, backend_canister, admin) = setup();
    let admin_account = Account { owner: admin, subaccount: None };
    let community_account = Account { owner: admin, subaccount: Some(COMMUNITY_POOL_SUBACCOUNT) };
    let team_account = Account { owner: admin, subaccount: Some(TEAM_VESTING_POOL_SUBACCOUNT) };
    let reserve_account = Account { owner: admin, subaccount: Some(RESERVE_POOL_SUBACCOUNT) };

    let admin_balance: u128 = decode_one(&pic.query_call(backend_canister, Principal::anonymous(), "balance_of", encode_args((admin_account,)).unwrap()).unwrap()).unwrap();
    let community_balance: u128 = decode_one(&pic.query_call(backend_canister, Principal::anonymous(), "balance_of", encode_args((community_account,)).unwrap()).unwrap()).unwrap();
    let team_balance: u128 = decode_one(&pic.query_call(backend_canister, Principal::anonymous(), "balance_of", encode_args((team_account,)).unwrap()).unwrap()).unwrap();
    let reserve_balance: u128 = decode_one(&pic.query_call(backend_canister, Principal::anonymous(), "balance_of", encode_args((reserve_account,)).unwrap()).unwrap()).unwrap();

    println!("Admin balance: {}", admin_balance);
    println!("Community balance: {}", community_balance);
    println!("Team vesting balance: {}", team_balance);
    println!("Reserve balance: {}", reserve_balance);

    assert_eq!(admin_balance + community_balance + team_balance + reserve_balance, 100_000_000_000_000_000_000_u128);
}

#[test]
fn test_get_logs() {
    let (pic, backend_canister, _) = setup();
    let response = pic.query_call(backend_canister, Principal::anonymous(), "get_logs", encode_args(()).unwrap()).expect("Failed to query get_logs");
    let result: Vec<LogEntry> = decode_one(&response).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].event_type, "Init");
}

#[test]
fn test_get_logs_by_range() {
    let (pic, backend_canister, _) = setup();
    let args = encode_args((0_u64, u64::MAX)).expect("Failed to encode args");
    let response = pic.query_call(backend_canister, Principal::anonymous(), "get_logs_by_range", args).expect("Failed to query get_logs_by_range");
    let result: Vec<LogEntry> = decode_one(&response).unwrap();
    assert_eq!(result.len(), 1);
}

#[test]
fn test_icrc1_allowance() {
    let (pic, backend_canister, admin) = setup();
    let owner = Account { owner: admin, subaccount: None };
    let spender = Account { owner: Principal::anonymous(), subaccount: None };
    let args = encode_args((owner, spender)).expect("Failed to encode args");
    let response = pic.query_call(backend_canister, Principal::anonymous(), "icrc1_allowance", args).expect("Failed to query icrc1_allowance");
    let result: u128 = decode_one(&response).unwrap();
    assert_eq!(result, 0_u128);
}

// Update Tests
#[test]
fn test_register_user() {
    let (pic, backend_canister, _) = setup();
    let user = Account { owner: Principal::from_text("2vxsx-fae").unwrap(), subaccount: None };
    let args = encode_args((user.clone(),)).expect("Failed to encode args");
    let response = pic.update_call(backend_canister, Principal::anonymous(), "register_user", args).expect("Failed to call register_user");
    let result: Result<String, LedgerError> = decode_one(&response).unwrap();
    assert!(result.unwrap().starts_with("User registered with 200 CRNL"));
}

#[test]
fn test_claim_referral() {
    let (pic, backend_canister, _) = setup();
    let referrer = Account { owner: Principal::from_text("2vxsx-fae").unwrap(), subaccount: None };
    let referee = Account { owner: Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap(), subaccount: None };

    let reg_args = encode_args((referrer.clone(),)).expect("Failed to encode args");
    let reg_response = pic.update_call(backend_canister, Principal::anonymous(), "register_user", reg_args).expect("Failed to register referrer");
    let reg_result: Result<String, LedgerError> = decode_one(&reg_response).unwrap();
    let referral_code = reg_result.unwrap().split("Your referral code is: ").nth(1).unwrap().to_string();

    let args = encode_args((referral_code, referee)).expect("Failed to encode args");
    let response = pic.update_call(backend_canister, Principal::anonymous(), "claim_referral", args).expect("Failed to call claim_referral");
    let result: Result<String, LedgerError> = decode_one(&response).unwrap();
    assert_eq!(result.unwrap(), "Referral reward of 20 CRNL credited");
}

#[test]
fn test_icrc1_transfer() {
    let (pic, backend_canister, admin) = setup();
    let from = Account { owner: admin, subaccount: Some(COMMUNITY_POOL_SUBACCOUNT) };
    let to = Account { owner: Principal::from_text("2vxsx-fae").unwrap(), subaccount: None };
    let amount = 1_000_000_000_u128; // 10 CRNL

    let from_balance: u128 = decode_one(&pic.query_call(backend_canister, Principal::anonymous(), "balance_of", encode_args((from.clone(),)).unwrap()).unwrap()).unwrap();
    println!("Transfer from balance before: {}", from_balance);

    let args = encode_args((from.clone(), to.clone(), amount)).expect("Failed to encode args");
    let response = pic.update_call(backend_canister, admin, "icrc1_transfer", args).expect("Failed to call icrc1_transfer");
    let result: Result<(), LedgerError> = decode_one(&response).unwrap();
    if let Err(e) = &result {
        println!("Transfer failed with: {:?}", e);
    }
    assert!(result.is_ok());

    let to_balance: u128 = decode_one(&pic.query_call(backend_canister, Principal::anonymous(), "balance_of", encode_args((to,)).unwrap()).unwrap()).unwrap();
    assert_eq!(to_balance, amount);
}

#[test]
fn test_icrc1_approve_and_transfer_from() {
    let (pic, backend_canister, admin) = setup();
    let owner = Account { owner: admin, subaccount: Some(COMMUNITY_POOL_SUBACCOUNT) };
    let spender = Account { owner: Principal::from_text("2vxsx-fae").unwrap(), subaccount: None };
    let to = Account { owner: Principal::anonymous(), subaccount: None };
    let amount = 1_000_000_000_u128; // 10 CRNL

    let owner_balance: u128 = decode_one(&pic.query_call(backend_canister, Principal::anonymous(), "balance_of", encode_args((owner.clone(),)).unwrap()).unwrap()).unwrap();
    println!("Owner balance before approve: {}", owner_balance);

    let approve_args = encode_args((owner.clone(), spender.clone(), amount)).expect("Failed to encode args");
    let approve_response = pic.update_call(backend_canister, admin, "icrc1_approve", approve_args).expect("Failed to call icrc1_approve");
    let approve_result: Result<(), LedgerError> = decode_one(&approve_response).unwrap();
    if let Err(e) = &approve_result {
        println!("Approve failed with: {:?}", e);
    }
    assert!(approve_result.is_ok());

    let transfer_args = encode_args((spender.clone(), owner.clone(), to.clone(), amount)).expect("Failed to encode args");
    let transfer_response = pic.update_call(backend_canister, spender.owner, "icrc1_transfer_from", transfer_args).expect("Failed to call icrc1_transfer_from");
    let transfer_result: Result<(), LedgerError> = decode_one(&transfer_response).unwrap();
    assert!(transfer_result.is_ok());

    let to_balance: u128 = decode_one(&pic.query_call(backend_canister, Principal::anonymous(), "balance_of", encode_args((to,)).unwrap()).unwrap()).unwrap();
    assert_eq!(to_balance, amount);
}

#[test]
fn test_create_media_chronolock() {
    let (pic, backend_canister, admin) = setup();
    let caller = Account { owner: admin, subaccount: Some(COMMUNITY_POOL_SUBACCOUNT) };

    let caller_balance_args = encode_args((caller.clone(),)).expect("Failed to encode args");
    let caller_balance: u128 = decode_one(&pic.query_call(backend_canister, Principal::anonymous(), "balance_of", caller_balance_args.clone()).unwrap()).unwrap();
    println!("Caller balance before chronolock: {}", caller_balance);

    let args = encode_args((caller.clone(),)).expect("Failed to encode args");
    let response = pic.update_call(backend_canister, admin, "create_media_chronolock", args).expect("Failed to call create_media_chronolock");
    let result: Result<String, LedgerError> = decode_one(&response).unwrap();
    if let Err(e) = &result {
        println!("Create media chronolock failed with: {:?}", e);
    }
    assert_eq!(result.unwrap(), "Media ChronoLock created for 20 CRNL");

    let post_balance: u128 = decode_one(&pic.query_call(backend_canister, Principal::anonymous(), "balance_of", caller_balance_args).unwrap()).unwrap();
    println!("Caller balance after chronolock: {}", post_balance);
    assert_eq!(post_balance, caller_balance - 2_000_000_000_u128);
}

#[test]
fn test_create_text_chronolock() {
    let (pic, backend_canister, admin) = setup();
    let caller = Account { owner: admin, subaccount: None };
    let args = encode_args((caller,)).expect("Failed to encode args");
    let response = pic.update_call(backend_canister, admin, "create_text_chronolock", args).expect("Failed to call create_text_chronolock");
    let result: Result<String, LedgerError> = decode_one(&response).unwrap();
    assert_eq!(result.unwrap(), "Text ChronoLock created for free");
}

#[test]
fn test_set_transfer_fee() {
    let (pic, backend_canister, admin) = setup();
    let args = encode_args((200_000_u128,)).expect("Failed to encode args");
    let response = pic.update_call(backend_canister, admin, "set_transfer_fee", args).expect("Failed to call set_transfer_fee");
    let result: Result<(), LedgerError> = decode_one(&response).unwrap();
    assert!(result.is_ok());

    let fee_response = pic.query_call(backend_canister, Principal::anonymous(), "icrc1_fee", encode_args(()).unwrap()).expect("Failed to query icrc1_fee");
    let fee: u128 = decode_one(&fee_response).unwrap();
    assert_eq!(fee, 200_000_u128);
}