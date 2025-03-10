use candid::CandidType;
use candid::Principal;
use ic_cdk_macros::{init, query, update};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::storable::Bound;
use ic_cdk::api::management_canister::main::raw_rand;
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, Storable};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::cell::RefCell;

// Define memory type for stable structures
type Memory = VirtualMemory<DefaultMemoryImpl>;

// -------------------------
// Data Structures
// -------------------------

// Account struct for ICRC-1 compliance
#[derive(CandidType, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Account {
    owner: Principal,
    subaccount: Option<[u8; 32]>,
}

impl Storable for Account {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    const BOUND: Bound = Bound::Unbounded;
}

// Metadata struct for token information
#[derive(CandidType, Serialize, Deserialize, Clone)]
struct Metadata {
    name: String,
    symbol: String,
    decimals: u8,
    total_supply: u128,
    transfer_fee: u128,
    community_pool: u128,
    team_vesting_pool: u128,
    reserve: u128,
    total_burned: u128,
    vesting_start_time: u64,
    vesting_duration: u64,
}

impl Storable for Metadata {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    const BOUND: Bound = Bound::Unbounded;
}

// Log entry struct
#[derive(CandidType, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct LogEntry {
    timestamp: u64,
    event_type: String,
    details: String,
}

impl Storable for LogEntry {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    const BOUND: Bound = Bound::Bounded {
        max_size: 1024,
        is_fixed_size: false,
    };
}

#[derive(CandidType, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct AllowanceKey {
    owner: Account,
    spender: Account,
}

impl Storable for AllowanceKey {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 200, // Two Accounts, each up to 100 bytes
        is_fixed_size: false,
    };
}

// Error types for better handling
#[derive(CandidType, Deserialize, Clone)]
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

// -------------------------
// Global Stable Structures & Thread-Local Storage
// -------------------------

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );
    static METADATA: RefCell<StableBTreeMap<u8, Metadata, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|mm| mm.borrow().get(MemoryId::new(0))))
    );
    static BALANCES: RefCell<StableBTreeMap<Account, u128, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|mm| mm.borrow().get(MemoryId::new(1))))
    );
    static ALLOWANCES: RefCell<StableBTreeMap<AllowanceKey, u128, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|mm| mm.borrow().get(MemoryId::new(2))))
    );
    static LOGS: RefCell<StableBTreeMap<u64, LogEntry, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|mm| mm.borrow().get(MemoryId::new(3))))
    );
    // Stable maps for referral system:
    // Mapping from Account to referral code (String)
    static REFERRAL_BY_ACCOUNT: RefCell<StableBTreeMap<Account, String, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|mm| mm.borrow().get(MemoryId::new(4))))
    );
    // Mapping from referral code (String) to Account
    static ACCOUNT_BY_REFERRAL: RefCell<StableBTreeMap<String, Account, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|mm| mm.borrow().get(MemoryId::new(5))))
    );
    // Admin principal storage
    static ADMIN: RefCell<Principal> = RefCell::new(Principal::anonymous());
    // Mapping from referee Account to a bool indicating that the referral reward has been claimed.
    static CLAIMED_REFERRALS: RefCell<StableBTreeMap<Account, bool, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|mm| mm.borrow().get(MemoryId::new(7))))
    );
}

// Define subaccount constants for the pools
const COMMUNITY_POOL_SUBACCOUNT: [u8; 32] = [1u8; 32];
const TEAM_VESTING_POOL_SUBACCOUNT: [u8; 32] = [2u8; 32];
const RESERVE_POOL_SUBACCOUNT: [u8; 32] = [3u8; 32];

// -------------------------
// Helper Functions
// -------------------------

// Get current time in seconds
fn current_time() -> u64 {
    ic_cdk::api::time() / 1_000_000_000
}

fn caller() -> Principal {
    ic_cdk::caller()
}

// Updated admin principal getter to use the stored value
fn admin_principal() -> Principal {
    ADMIN.with(|a| a.borrow().clone())
}

// Generates a random 7-character referral code using uppercase and lowercase letters.
// This function calls the management canister’s raw randomness API asynchronously.
// Note: raw_rand returns a tuple; we destructure it to obtain a Vec<u8>.
async fn generate_random_referral_code() -> String {
    let charset = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    let (random_bytes, ) = raw_rand().await.unwrap();
    random_bytes
        .iter()
        .take(7)
        .map(|b| {
            let index = (*b as usize) % charset.len();
            charset[index] as char
        })
        .collect()
}

// -------------------------
// Updated Initialization
// -------------------------

#[init]
fn init(
    name: String,
    symbol: String,
    total_supply: u128,
    vesting_duration: u64,
    transfer_fee: u128,
    admin: Principal,
) {
    // Store the provided admin principal
    ADMIN.with(|a| *a.borrow_mut() = admin.clone());

    let decimals = 8;
    // Calculate pool amounts (50% community, 20% team, 30% reserve)
    let community_pool_amount = total_supply * 50 / 100;
    let team_vesting_pool_amount = total_supply * 20 / 100;
    let reserve_amount = total_supply * 30 / 100;

    // Insert token metadata
    METADATA.with(|metadata| {
        let mut m = metadata.borrow_mut();
        m.insert(0, Metadata {
            name,
            symbol,
            decimals,
            total_supply,
            transfer_fee,
            community_pool: community_pool_amount,
            team_vesting_pool: team_vesting_pool_amount,
            reserve: reserve_amount,
            total_burned: 0,
            vesting_start_time: current_time(),
            vesting_duration,
        });
    });

    // Create pool accounts as subaccounts under the admin account
    let community_account = Account { owner: admin.clone(), subaccount: Some(COMMUNITY_POOL_SUBACCOUNT) };
    let team_account = Account { owner: admin.clone(), subaccount: Some(TEAM_VESTING_POOL_SUBACCOUNT) };
    let reserve_account = Account { owner: admin.clone(), subaccount: Some(RESERVE_POOL_SUBACCOUNT) };

    // Initialize the balances for each pool account
    BALANCES.with(|balances| {
       let mut b = balances.borrow_mut();
       b.insert(community_account, community_pool_amount);
       b.insert(team_account, team_vesting_pool_amount);
       b.insert(reserve_account, reserve_amount);
    });

    log_event("Init", format!(
        "Canister initialized with total supply: {}, transfer fee: {}, admin: {}",
        total_supply, transfer_fee, admin
    ));
}

// -------------------------
// Updated Functions with Referral System
// -------------------------

#[update]
async fn register_user(user: Account) -> Result<String, LedgerError> {
    METADATA.with(|metadata| {
        let mut metadata_ref = metadata.borrow_mut();
        let m = metadata_ref.get(&0).unwrap().clone();
        if BALANCES.with(|b| b.borrow().contains_key(&user)) {
            return Err(LedgerError::AlreadyRegistered);
        }
        let welcome_amount = 200 * 10u128.pow(m.decimals as u32);
        if m.community_pool < welcome_amount {
            return Err(LedgerError::InsufficientPoolFunds);
        }
        // Deduct from community pool
        let new_community_pool = m.community_pool.checked_sub(welcome_amount)
            .ok_or(LedgerError::ArithmeticError)?;
        // Update balance for the new user
        BALANCES.with(|b| b.borrow_mut().insert(user.clone(), welcome_amount));
        // Update metadata with new community pool value
        let mut updated_metadata = m.clone();
        updated_metadata.community_pool = new_community_pool;
        metadata_ref.insert(0, updated_metadata);
        log_event(
            "UserRegistered",
            format!("User: {}, Amount: {}", user.owner, welcome_amount),
        );
        Ok(m)
    })?;
    // Generate a referral code for this new user if not already assigned.
    if !REFERRAL_BY_ACCOUNT.with(|rba| rba.borrow().contains_key(&user)) {
        let mut referral_code = generate_random_referral_code().await;
        // Ensure uniqueness in case of a rare collision.
        while ACCOUNT_BY_REFERRAL.with(|abr| abr.borrow().contains_key(&referral_code)) {
            referral_code = generate_random_referral_code().await;
        }
        REFERRAL_BY_ACCOUNT.with(|rba| {
            rba.borrow_mut().insert(user.clone(), referral_code.clone());
        });
        ACCOUNT_BY_REFERRAL.with(|abr| {
            abr.borrow_mut().insert(referral_code.clone(), user.clone());
        });
        let symbol = METADATA.with(|metadata| metadata.borrow().get(&0).unwrap().symbol.clone());
        log_event(
            "ReferralRegistered",
            format!("User: {}, Referral_Code: {}", user.owner, referral_code),
        );
        return Ok(format!("User registered with 200 {}. Your referral code is: {}", symbol, referral_code));
    }
    let symbol = METADATA.with(|metadata| metadata.borrow().get(&0).unwrap().symbol.clone());
    Ok(format!("User registered with 200 {}", symbol))
}

#[update]
fn claim_referral(referral_code: String, referee: Account) -> Result<String, LedgerError> {
    // Look up the referrer by referral code
    let referrer_opt = ACCOUNT_BY_REFERRAL.with(|abr| abr.borrow().get(&referral_code).clone());
    let referrer = match referrer_opt {
        None => return Err(LedgerError::InvalidReferral),
        Some(acc) => acc,
    };

    // Check if the referee has already claimed a referral reward.
    if CLAIMED_REFERRALS.with(|cr| cr.borrow().contains_key(&referee)) {
        return Err(LedgerError::InvalidReferral);
    }

    METADATA.with(|metadata| {
        let mut metadata_ref = metadata.borrow_mut();
        let mut m = metadata_ref.get(&0).unwrap().clone();
        let reward = 20 * 10u128.pow(m.decimals as u32);
        if m.community_pool < reward {
            return Err(LedgerError::InsufficientPoolFunds);
        }
        m.community_pool = m.community_pool.checked_sub(reward).ok_or(LedgerError::ArithmeticError)?;
        let referrer_balance = BALANCES.with(|b| b.borrow().get(&referrer).unwrap_or(0));
        let new_balance = referrer_balance.checked_add(reward).ok_or(LedgerError::ArithmeticError)?;
        BALANCES.with(|b| b.borrow_mut().insert(referrer.clone(), new_balance));
        metadata_ref.insert(0, m.clone());
        log_event(
            "ReferralClaimed",
            format!("Referrer: {}, Referee: {}, Reward: {}", referrer.owner, referee.owner, reward),
        );
        // Mark that this referee has claimed their referral reward.
        CLAIMED_REFERRALS.with(|cr| {
            cr.borrow_mut().insert(referee.clone(), true);
        });
        Ok(format!("Referral reward of 20 {} credited", m.symbol))
    })
}



// -------------------------
// Existing Ledger Functions
// -------------------------

#[update]
fn icrc1_transfer(from: Account, to: Account, amount: u128) -> Result<(), LedgerError> {
    METADATA.with(|metadata| {
        let m = metadata.borrow().get(&0).unwrap();
        let transfer_fee = m.transfer_fee;

        // Check for max amount or overflow early
        if amount == u128::MAX {
            return Err(LedgerError::ArithmeticError);
        }
        let total_deduction = amount.checked_add(transfer_fee).ok_or(LedgerError::ArithmeticError)?;

        let from_balance = BALANCES.with(|b| b.borrow().get(&from).unwrap_or(0));
        if from_balance < total_deduction {
            return Err(LedgerError::InsufficientBalance);
        }

        BALANCES.with(|b| {
            let mut b = b.borrow_mut();
            b.insert(from.clone(), from_balance - total_deduction);
            let to_balance = b.get(&to).unwrap_or(0);
            let new_to_balance = to_balance.checked_add(amount).ok_or(LedgerError::ArithmeticError)?;
            b.insert(to.clone(), new_to_balance);
            Ok(())
        })?;

        process_fee(transfer_fee)?;
        log_event(
            "Transfer",
            format!("From: {}, To: {}, Amount: {}, Fee: {}", from.owner, to.owner, amount, transfer_fee),
        );
        Ok(())
    })
}

#[update]
fn icrc1_approve(owner: Account, spender: Account, amount: u128) -> Result<(), LedgerError> {
    let caller = caller();
    if caller != owner.owner {
        return Err(LedgerError::Unauthorized);
    }
    ALLOWANCES.with(|allowances| {
        allowances
            .borrow_mut()
            .insert(AllowanceKey { owner: owner.clone(), spender: spender.clone() }, amount);
    });
    log_event(
        "Approval",
        format!("Owner: {}, Spender: {}, Amount: {}", owner.owner, spender.owner, amount),
    );
    Ok(())
}

#[update]
fn icrc1_transfer_from(spender: Account, from: Account, to: Account, amount: u128) -> Result<(), LedgerError> {
    METADATA.with(|metadata| {
        let m = metadata.borrow().get(&0).unwrap();
        let transfer_fee = m.transfer_fee;
        let total_deduction = amount.checked_add(transfer_fee).ok_or(LedgerError::ArithmeticError)?;

        let allowance_key = AllowanceKey { owner: from.clone(), spender: spender.clone() };
        let allowance = ALLOWANCES.with(|a| a.borrow().get(&allowance_key).unwrap_or(0));
        if allowance < amount {
            return Err(LedgerError::InsufficientAllowance);
        }

        let from_balance = BALANCES.with(|b| b.borrow().get(&from).unwrap_or(0));
        if from_balance < total_deduction {
            return Err(LedgerError::InsufficientBalance);
        }

        ALLOWANCES.with(|a| a.borrow_mut().insert(allowance_key, allowance - amount));
        BALANCES.with(|b| {
            let mut b = b.borrow_mut();
            b.insert(from.clone(), from_balance - total_deduction);
            let to_balance = b.get(&to).unwrap_or(0);
            let new_to_balance = to_balance.checked_add(amount).ok_or(LedgerError::ArithmeticError)?;
            b.insert(to.clone(), new_to_balance);
            Ok(())
        })?;
        process_fee(transfer_fee)?;
        log_event(
            "TransferFrom",
            format!(
                "Spender: {}, From: {}, To: {}, Amount: {}, Fee: {}",
                spender.owner, from.owner, to.owner, amount, transfer_fee
            ),
        );
        Ok(())
    })
}

#[update]
fn create_media_chronolock(caller: Account) -> Result<String, LedgerError> {
    METADATA.with(|metadata| {
        let m = metadata.borrow().get(&0).unwrap();
        let creation_fee = 2_000_000_000; // 20 $CRNL
        let balance = BALANCES.with(|b| b.borrow().get(&caller).unwrap_or(0));
        if balance < creation_fee {
            return Err(LedgerError::InsufficientBalance);
        }
        BALANCES.with(|b| b.borrow_mut().insert(caller.clone(), balance - creation_fee));
        process_fee(creation_fee)?;
        log_event(
            "MediaChronoLockCreated",
            format!("Caller: {}, Fee: {}", caller.owner, creation_fee),
        );
        Ok(format!("Media ChronoLock created for 20 {}", m.symbol))
    })
}

#[update]
fn create_text_chronolock(caller: Account) -> Result<String, LedgerError> {
    log_event(
        "TextChronoLockCreated",
        format!("Caller: {}", caller.owner),
    );
    Ok("Text ChronoLock created for free".to_string())
}

#[update]
fn set_transfer_fee(new_fee: u128) -> Result<(), LedgerError> {
    let caller = caller();
    if !is_admin(caller) {
        return Err(LedgerError::Unauthorized);
    }
    METADATA.with(|metadata| {
        let mut m = metadata.borrow_mut().get(&0).unwrap().clone();
        m.transfer_fee = new_fee;
        metadata.borrow_mut().insert(0, m);
    });
    log_event("SetTransferFee", format!("New fee: {}", new_fee));
    Ok(())
}

#[query]
fn get_referral_code(user: Account) -> Option<String> {
    REFERRAL_BY_ACCOUNT.with(|rba| rba.borrow().get(&user).clone())
}

#[query]
fn icrc1_allowance(owner: Account, spender: Account) -> u128 {
    let key = AllowanceKey { owner, spender };
    ALLOWANCES.with(|a| a.borrow().get(&key).unwrap_or(0))
}

#[query]
fn icrc1_name() -> String {
    METADATA.with(|m| m.borrow().get(&0).unwrap().name.clone())
}

#[query]
fn icrc1_symbol() -> String {
    METADATA.with(|m| m.borrow().get(&0).unwrap().symbol.clone())
}

#[query]
fn icrc1_decimals() -> u8 {
    METADATA.with(|m| m.borrow().get(&0).unwrap().decimals)
}

#[query]
fn icrc1_total_supply() -> u128 {
    METADATA.with(|m| m.borrow().get(&0).unwrap().total_supply)
}

#[query]
fn icrc1_fee() -> u128 {
    METADATA.with(|m| m.borrow().get(&0).unwrap().transfer_fee)
}

#[query]
fn balance_of(account: Account) -> u128 {
    BALANCES.with(|b| b.borrow().get(&account).unwrap_or(0))
}

#[query]
fn community_pool_balance() -> u128 {
    METADATA.with(|m| m.borrow().get(&0).unwrap().community_pool)
}

#[query]
fn total_burned() -> u128 {
    METADATA.with(|m| m.borrow().get(&0).unwrap().total_burned)
}

#[query]
fn get_logs() -> Vec<LogEntry> {
    LOGS.with(|logs| {
        logs.borrow()
            .iter()
            .map(|(_, entry)| entry.clone())
            .collect()
    })
}

#[query]
fn get_logs_by_range(start_time: u64, end_time: u64) -> Vec<LogEntry> {
    LOGS.with(|logs| {
        logs.borrow()
            .range(start_time..=end_time)
            .map(|(_, entry)| entry.clone())
            .collect()
    })
}

// -------------------------
// Helper: Check Admin
// -------------------------

fn is_admin(caller: Principal) -> bool {
    caller == admin_principal()
}

// -------------------------
// Centralized Fee Processing & Logging
// -------------------------

fn process_fee(fee: u128) -> Result<(), LedgerError> {
    METADATA.with(|metadata| {
        let mut m = metadata.borrow_mut().get(&0).unwrap().clone();
        let burn_amount = fee * 20 / 100;
        let pool_amount = fee * 10 / 100;
        let dapp_amount = fee * 70 / 100;

        m.total_supply = m.total_supply.checked_sub(burn_amount).ok_or(LedgerError::ArithmeticError)?;
        m.total_burned = m.total_burned.checked_add(burn_amount).ok_or(LedgerError::ArithmeticError)?;
        m.community_pool = m.community_pool.checked_add(pool_amount).ok_or(LedgerError::ArithmeticError)?;
        metadata.borrow_mut().insert(0, m);

        log_event(
            "FeeProcessed",
            format!("Fee: {}, Burned: {}, Pool: {}, DApp: {}", fee, burn_amount, pool_amount, dapp_amount),
        );
        Ok(())
    })
}

fn log_event(event_type: &str, details: String) {
    LOGS.with(|logs| {
        let mut logs = logs.borrow_mut();
        let timestamp = current_time();
        logs.insert(timestamp, LogEntry {
            timestamp,
            event_type: event_type.to_string(),
            details,
        });
        if logs.len() > 10_000 {
            if let Some((first_key, _)) = logs.first_key_value() {
                logs.remove(&first_key);
            }
        }
    });
}

// Export Candid interface
ic_cdk::export_candid!();