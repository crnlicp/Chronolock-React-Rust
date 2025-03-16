use candid::{CandidType, Nat, Principal};
use ic_cdk::api::management_canister::main::raw_rand;
use ic_cdk::api::time;
use ic_cdk_macros::{init, query, update};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell, Storable};
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
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

// TransferArgs for ICRC-1 compliance
#[derive(CandidType, Serialize, Deserialize)]
struct TransferArgs {
    from_subaccount: Option<[u8; 32]>,
    to: Account,
    amount: Nat,
}

// ApproveArgs for ICRC-2 compliance
#[derive(CandidType, Serialize, Deserialize)]
struct ApproveArgs {
    from_subaccount: Option<[u8; 32]>,
    spender: Account,
    amount: Nat,
}

// TransferFromArgs for ICRC-2 compliance
#[derive(CandidType, Serialize, Deserialize)]
struct TransferFromArgs {
    spender: Account,
    from: Account,
    to: Account,
    amount: Nat,
}

// Metadata struct now only tracks global token info and vesting details.
#[derive(CandidType, Serialize, Deserialize, Clone)]
struct Metadata {
    name: String,
    symbol: String,
    decimals: u8,
    total_supply: u128,
    transfer_fee: u128,
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

#[derive(CandidType, Serialize, Deserialize, Clone)]
struct TransactionEvent {
    tx_id: [u8; 32],          // Unique transaction ID
    timestamp: u64,           // Time of the event
    event_type: String,       // "Transfer", "TransferFrom", "Approval"
    from: Account,            // Source account
    to: Option<Account>,      // Destination account (None for approvals)
    spender: Option<Account>, // Spender account (for approvals/transfer_from)
    amount: Nat,              // Amount transferred or approved
    fee: Option<Nat>,         // Transaction fee, if applicable
}

impl Storable for TransactionEvent {
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

// Error types for better handling
#[derive(Debug, CandidType, Deserialize, Clone)]
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
}

// -------------------------
// Global Stable Structures & Thread-Local Storage
// -------------------------

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );
    static ADMIN_STORAGE: RefCell<StableBTreeMap<u8, Principal, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|mm| mm.borrow().get(MemoryId::new(0))))
    );
    static METADATA: RefCell<StableBTreeMap<u8, Metadata, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|mm| mm.borrow().get(MemoryId::new(1))))
    );
    static BALANCES: RefCell<StableBTreeMap<Account, u128, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|mm| mm.borrow().get(MemoryId::new(2))))
    );
    static ALLOWANCES: RefCell<StableBTreeMap<AllowanceKey, u128, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|mm| mm.borrow().get(MemoryId::new(3))))
    );
    static REFERRAL_BY_ACCOUNT: RefCell<StableBTreeMap<Account, String, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|mm| mm.borrow().get(MemoryId::new(4))))
    );
    static ACCOUNT_BY_REFERRAL: RefCell<StableBTreeMap<String, Account, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|mm| mm.borrow().get(MemoryId::new(5))))
    );
    static CLAIMED_REFERRALS: RefCell<StableBTreeMap<Account, bool, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|mm| mm.borrow().get(MemoryId::new(6))))
    );
    static LOGS: RefCell<StableBTreeMap<u64, LogEntry, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|mm| mm.borrow().get(MemoryId::new(8))))
    );
    static TRANSACTIONS: RefCell<StableBTreeMap<[u8; 32], TransactionEvent, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|mm| mm.borrow().get(MemoryId::new(10))))
    );
    // Use a 64-bit counter for log keys to avoid overflow.
    static LOG_COUNTER: RefCell<StableCell<u64, Memory>> = RefCell::new(
        StableCell::init(MEMORY_MANAGER.with(|mm| mm.borrow().get(MemoryId::new(11))), 0)
            .expect("Failed to initialize LOG_COUNTER")
    );
}

// Define subaccount constants for the pools and dapp funds
const COMMUNITY_POOL_SUBACCOUNT: [u8; 32] = [1u8; 32];
const TEAM_VESTING_POOL_SUBACCOUNT: [u8; 32] = [2u8; 32];
const RESERVE_POOL_SUBACCOUNT: [u8; 32] = [3u8; 32];
const DAPP_FUNDS_SUBACCOUNT: [u8; 32] = [4u8; 32];

// -------------------------
// Helper Functions
// -------------------------

fn current_time() -> u64 {
    // Returns seconds
    ic_cdk::api::time() / 1_000_000_000
}

fn caller() -> Principal {
    ic_cdk::caller()
}

fn admin_principal() -> Principal {
    ADMIN_STORAGE.with(|a| a.borrow().get(&0).unwrap_or(Principal::anonymous()))
}

fn is_admin(caller: Principal) -> bool {
    caller == admin_principal()
}

// Convert Nat to u128 safely
fn nat_to_u128(n: Nat) -> Result<u128, LedgerError> {
    n.0.to_u128().ok_or(LedgerError::ArithmeticError)
}

// Generates a longer, more random referral code.
async fn generate_random_referral_code(random_bytes: Option<Vec<u8>>) -> String {
    let charset = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let code_length = 10;
    let random_bytes = if let Some(bytes) = random_bytes {
        bytes
    } else {
        let (bytes,) = raw_rand().await.unwrap(); // Directly await in async context
        bytes
    };
    random_bytes
        .iter()
        .take(code_length)
        .map(|b| {
            let index = (*b as usize) % charset.len();
            charset[index] as char
        })
        .collect()
}

// Asynchronously generate a transaction ID using time and randomness.
async fn generate_tx_id(random_bytes: Option<Vec<u8>>) -> [u8; 32] {
    let random_bytes = if let Some(bytes) = random_bytes {
        bytes
    } else {
        let (bytes,) = raw_rand().await.unwrap(); // Directly await in async context
        bytes
    };
    let mut hasher = Sha256::new();
    hasher.update(ic_cdk::api::time().to_le_bytes());
    hasher.update(&random_bytes);
    hasher.finalize().into()
}

// Checks vesting conditions for the team pool.
// If the account is the team vesting pool, ensure that vesting period has passed.
fn check_team_vesting(account: &Account) -> Result<(), LedgerError> {
    if let Some(sub) = account.subaccount {
        if sub == TEAM_VESTING_POOL_SUBACCOUNT {
            let metadata = METADATA.with(|m| m.borrow().get(&0).unwrap().clone());
            let unlock_time = metadata.vesting_start_time + metadata.vesting_duration;
            if current_time() < unlock_time {
                return Err(LedgerError::VestingLocked);
            }
        }
    }
    Ok(())
}

// -------------------------
// Initialization
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
    ADMIN_STORAGE.with(|a| a.borrow_mut().insert(0, admin.clone()));

    let decimals = 8;
    // Define pool amounts as percentages of total_supply.
    let community_pool_amount = total_supply * 50 / 100;
    let team_vesting_pool_amount = total_supply * 20 / 100;
    let reserve_amount = total_supply * 30 / 100;

    // Initialize Metadata without redundant pool fields.
    METADATA.with(|metadata| {
        let mut m = metadata.borrow_mut();
        m.insert(
            0,
            Metadata {
                name,
                symbol: symbol.clone(),
                decimals,
                total_supply,
                transfer_fee,
                total_burned: 0,
                vesting_start_time: current_time(),
                vesting_duration,
            },
        );
    });

    // Create dedicated pool accounts.
    let community_account = Account {
        owner: admin.clone(),
        subaccount: Some(COMMUNITY_POOL_SUBACCOUNT),
    };
    let team_account = Account {
        owner: admin.clone(),
        subaccount: Some(TEAM_VESTING_POOL_SUBACCOUNT),
    };
    let reserve_account = Account {
        owner: admin.clone(),
        subaccount: Some(RESERVE_POOL_SUBACCOUNT),
    };
    let dapp_account = Account {
        owner: admin.clone(),
        subaccount: Some(DAPP_FUNDS_SUBACCOUNT),
    };

    BALANCES.with(|balances| {
        let mut b = balances.borrow_mut();
        b.insert(community_account, community_pool_amount);
        b.insert(team_account, team_vesting_pool_amount);
        b.insert(reserve_account, reserve_amount);
        b.insert(dapp_account, 0);
    });

    log_event(
        "Init",
        format!(
            "Canister initialized with total supply: {}, transfer fee: {}, admin: {}",
            total_supply, transfer_fee, admin
        ),
    );
}

// -------------------------
// Update Functions
// -------------------------

#[update]
async fn register_user(
    user: Account,
    random_bytes: Option<Vec<u8>>,
) -> Result<String, LedgerError> {
    // Retrieve Metadata for token decimals.
    let metadata = METADATA.with(|m| m.borrow().get(&0).unwrap().clone());
    let welcome_amount = 200 * 10u128.pow(metadata.decimals as u32);

    // Check if user is already registered.
    if BALANCES.with(|b| b.borrow().contains_key(&user)) {
        return Err(LedgerError::AlreadyRegistered);
    }

    // Deduct welcome tokens from the community pool.
    let community_account = Account {
        owner: admin_principal(),
        subaccount: Some(COMMUNITY_POOL_SUBACCOUNT),
    };
    BALANCES.with(|balances| {
        let mut b = balances.borrow_mut();
        let pool_balance = b.get(&community_account).unwrap_or(0);
        if pool_balance < welcome_amount {
            return Err(LedgerError::InsufficientPoolFunds);
        }
        b.insert(community_account, pool_balance - welcome_amount);
        // Credit the new user.
        b.insert(user.clone(), welcome_amount);
        Ok(())
    })?;

    // Log user registration.
    let subaccount_str = user.subaccount.map_or("None".to_string(), |sub| {
        let trimmed: Vec<u8> = sub
            .iter()
            .copied()
            .rev()
            .skip_while(|&x| x == 0)
            .collect::<Vec<u8>>()
            .into_iter()
            .rev()
            .collect();
        if trimmed.is_empty() {
            "[0]".to_string()
        } else {
            format!("len: {}, value: {:?}", sub.len(), trimmed)
        }
    });
    log_event(
        "UserRegistered",
        format!(
            "Account: {}, Subaccount: {}, Amount: {}",
            user.owner, subaccount_str, welcome_amount
        ),
    );

    // Register referral if not already present.
    if !REFERRAL_BY_ACCOUNT.with(|rba| rba.borrow().contains_key(&user)) {
        let mut referral_code = generate_random_referral_code(random_bytes.clone()).await;
        while ACCOUNT_BY_REFERRAL.with(|abr| abr.borrow().contains_key(&referral_code)) {
            referral_code = generate_random_referral_code(random_bytes.clone()).await;
        }
        REFERRAL_BY_ACCOUNT.with(|rba| {
            rba.borrow_mut().insert(user.clone(), referral_code.clone());
        });
        ACCOUNT_BY_REFERRAL.with(|abr| {
            abr.borrow_mut().insert(referral_code.clone(), user.clone());
        });
        log_event(
            "ReferralRegistered",
            format!(
                "User: {}, Subaccount: {}, Referral_Code: {}",
                user.owner, subaccount_str, referral_code
            ),
        );
        return Ok(format!(
            "User registered with 200 {}. Your referral code is: {}",
            metadata.symbol, referral_code
        ));
    }
    Ok(format!("User registered with 200 {}", metadata.symbol))
}

#[update]
fn claim_referral(referral_code: String, referee: Account) -> Result<String, LedgerError> {
    let referrer_opt = ACCOUNT_BY_REFERRAL.with(|abr| abr.borrow().get(&referral_code).clone());
    let referrer = match referrer_opt {
        None => return Err(LedgerError::InvalidReferral),
        Some(acc) => acc,
    };

    if CLAIMED_REFERRALS.with(|cr| cr.borrow().contains_key(&referee)) {
        return Err(LedgerError::InvalidReferral);
    }

    // For referral rewards, credit the referrer from the community pool.
    let metadata = METADATA.with(|m| m.borrow().get(&0).unwrap().clone());
    let reward = 20 * 10u128.pow(metadata.decimals as u32);

    let community_account = Account {
        owner: admin_principal(),
        subaccount: Some(COMMUNITY_POOL_SUBACCOUNT),
    };

    BALANCES.with(|balances| {
        let mut b = balances.borrow_mut();
        let pool_balance = b.get(&community_account).unwrap_or(0);
        if pool_balance < reward {
            return Err(LedgerError::InsufficientPoolFunds);
        }
        b.insert(community_account, pool_balance - reward);
        let referrer_balance = b.get(&referrer).unwrap_or(0);
        let new_balance = referrer_balance
            .checked_add(reward)
            .ok_or(LedgerError::ArithmeticError)?;
        b.insert(referrer.clone(), new_balance);
        Ok(())
    })?;

    CLAIMED_REFERRALS.with(|cr| {
        cr.borrow_mut().insert(referee.clone(), true);
    });
    log_event(
        "ReferralClaimed",
        format!(
            "Referrer: {}, Referee: {}, Reward: {}",
            referrer.owner, referee.owner, reward
        ),
    );
    Ok(format!(
        "Referral reward of 20 {} credited",
        metadata.symbol
    ))
}

#[update]
async fn icrc1_transfer(
    args: TransferArgs,
    random_bytes: Option<Vec<u8>>,
) -> Result<Nat, LedgerError> {
    let caller = caller();
    let from = Account {
        owner: caller,
        subaccount: args.from_subaccount,
    };

    // Enforce vesting for team pool if applicable.
    check_team_vesting(&from)?;

    let amount = nat_to_u128(args.amount.clone())?;
    let metadata = METADATA.with(|m| m.borrow().get(&0).unwrap().clone());
    let transfer_fee = metadata.transfer_fee;

    if amount == u128::MAX {
        return Err(LedgerError::ArithmeticError);
    }
    if amount < transfer_fee {
        return Err(LedgerError::InsufficientFee);
    }
    let amount_after_fee = amount
        .checked_sub(transfer_fee)
        .ok_or(LedgerError::ArithmeticError)?;

    // Update sender and receiver balances.
    BALANCES.with(|b| {
        let mut b = b.borrow_mut();
        let from_balance = b.get(&from).unwrap_or(0);
        if from_balance < amount {
            return Err(LedgerError::InsufficientBalance);
        }
        b.insert(from.clone(), from_balance - amount);
        let to_balance = b.get(&args.to).unwrap_or(0);
        let new_to_balance = to_balance
            .checked_add(amount_after_fee)
            .ok_or(LedgerError::ArithmeticError)?;
        b.insert(args.to.clone(), new_to_balance);
        Ok(())
    })?;

    process_fee(transfer_fee)?;

    let tx_id = generate_tx_id(random_bytes).await;
    TRANSACTIONS.with(|txs| {
        txs.borrow_mut().insert(
            tx_id,
            TransactionEvent {
                tx_id,
                timestamp: current_time(),
                event_type: "Transfer".to_string(),
                from: from.clone(),
                to: Some(args.to.clone()),
                spender: None,
                amount: args.amount.clone(),
                fee: Some(Nat::from(transfer_fee)),
            },
        );
    });

    Ok(args.amount)
}

#[update]
fn icrc1_approve(args: ApproveArgs) -> Result<Nat, LedgerError> {
    let caller = caller();
    let owner = Account {
        owner: caller,
        subaccount: args.from_subaccount,
    };
    let amount = nat_to_u128(args.amount.clone())?;

    // Check that caller is the owner.
    if caller != owner.owner {
        return Err(LedgerError::Unauthorized);
    }

    ALLOWANCES.with(|allowances| {
        allowances.borrow_mut().insert(
            AllowanceKey {
                owner: owner.clone(),
                spender: args.spender.clone(),
            },
            amount,
        );
    });

    // Note: generate_tx_id is async so we use a minimal workaround here.
    // We omit the transaction logging in this synchronous context.
    TRANSACTIONS.with(|txs| {
        // Using a placeholder tx_id of zeros; in production, consider refactoring to async.
        let tx_id = [0u8; 32];
        txs.borrow_mut().insert(
            tx_id,
            TransactionEvent {
                tx_id,
                timestamp: current_time(),
                event_type: "Approval".to_string(),
                from: owner.clone(),
                to: None,
                spender: Some(args.spender.clone()),
                amount: args.amount.clone(),
                fee: None,
            },
        );
    });

    Ok(args.amount)
}

#[update]
async fn icrc1_transfer_from(
    args: TransferFromArgs,
    random_bytes: Option<Vec<u8>>,
) -> Result<Nat, LedgerError> {
    // Enforce that only the approved spender can perform a transfer_from.
    if caller() != args.spender.owner {
        return Err(LedgerError::Unauthorized);
    }

    // If transferring from the team vesting pool, check vesting.
    check_team_vesting(&args.from)?;

    let amount = nat_to_u128(args.amount.clone())?;
    let metadata = METADATA.with(|m| m.borrow().get(&0).unwrap().clone());
    let transfer_fee = metadata.transfer_fee;

    if amount == u128::MAX {
        return Err(LedgerError::ArithmeticError);
    }
    if amount < transfer_fee {
        return Err(LedgerError::InsufficientFee);
    }
    let amount_after_fee = amount
        .checked_sub(transfer_fee)
        .ok_or(LedgerError::ArithmeticError)?;

    let allowance_key = AllowanceKey {
        owner: args.from.clone(),
        spender: args.spender.clone(),
    };
    let allowance = ALLOWANCES.with(|a| a.borrow().get(&allowance_key).unwrap_or(0));
    if allowance < amount {
        return Err(LedgerError::InsufficientAllowance);
    }

    let from_balance = BALANCES.with(|b| b.borrow().get(&args.from).unwrap_or(0));
    if from_balance < amount {
        return Err(LedgerError::InsufficientBalance);
    }

    ALLOWANCES.with(|a| a.borrow_mut().insert(allowance_key, allowance - amount));
    BALANCES.with(|b| {
        let mut b = b.borrow_mut();
        b.insert(args.from.clone(), from_balance - amount);
        let to_balance = b.get(&args.to).unwrap_or(0);
        let new_to_balance = to_balance
            .checked_add(amount_after_fee)
            .ok_or(LedgerError::ArithmeticError)?;
        b.insert(args.to.clone(), new_to_balance);
        Ok(())
    })?;

    process_fee(transfer_fee)?;

    let tx_id = generate_tx_id(random_bytes).await;
    TRANSACTIONS.with(|txs| {
        txs.borrow_mut().insert(
            tx_id,
            TransactionEvent {
                tx_id,
                timestamp: current_time(),
                event_type: "TransferFrom".to_string(),
                from: args.from.clone(),
                to: Some(args.to.clone()),
                spender: Some(args.spender.clone()),
                amount: args.amount.clone(),
                fee: Some(Nat::from(transfer_fee)),
            },
        );
    });

    Ok(args.amount)
}

#[update]
fn create_media_chronolock(caller: Account) -> Result<String, LedgerError> {
    let metadata = METADATA.with(|m| m.borrow().get(&0).unwrap().clone());
    let creation_fee = 2_000_000_000; // 20 $CRNL
    let balance = BALANCES.with(|b| b.borrow().get(&caller).unwrap_or(0));
    if balance < creation_fee {
        return Err(LedgerError::InsufficientBalance);
    }
    BALANCES.with(|b| {
        b.borrow_mut()
            .insert(caller.clone(), balance - creation_fee)
    });
    process_fee(creation_fee)?;
    log_event(
        "MediaChronoLockCreated",
        format!("Caller: {}, Fee: {}", caller.owner, creation_fee),
    );
    Ok(format!(
        "Media ChronoLock created for 20 {}",
        metadata.symbol
    ))
}

#[update]
fn create_text_chronolock(caller: Account) -> Result<String, LedgerError> {
    log_event("TextChronoLockCreated", format!("Caller: {}", caller.owner));
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

#[update]
async fn convert_dapp_funds_to_cycles() -> Result<(), LedgerError> {
    let caller = caller();
    if !is_admin(caller) {
        return Err(LedgerError::Unauthorized);
    }
    // For demonstration purposes, assume conversion happens here.
    log_event(
        "DappFundsConverted",
        "Converted dapp funds to cycles".to_string(),
    );
    Ok(())
}

// -------------------------
// Dummy Data Generation (For testing only)
// Note: The nested loops here have O(n²) complexity and should be batched/limited for large numbers of users.
// -------------------------

#[update]
async fn register_dummy_users(
    num_users: u32,
    random_bytes: Option<Vec<u8>>,
) -> Result<Vec<Account>, LedgerError> {
    ic_cdk::println!("Starting register_dummy_users");
    let caller = caller();
    if !is_admin(caller) {
        return Err(LedgerError::Unauthorized);
    }
    let mut users = Vec::new();
    let run_timestamp = current_time();
    log_event(
        "RegisterUsersStarted",
        format!("Registering {} users", num_users),
    );

    for i in 0..num_users {
        let mut subaccount = [0u8; 32];
        subaccount[0..8].copy_from_slice(&run_timestamp.to_le_bytes());
        subaccount[8..12].copy_from_slice(&i.to_le_bytes());
        let dummy_account = Account {
            owner: caller,
            subaccount: Some(subaccount),
        };
        match register_user(dummy_account.clone(), random_bytes.clone()).await {
            Ok(_) => {
                users.push(dummy_account.clone());
                log_event(
                    "UserRegistered",
                    format!("User {}: {}", i, dummy_account.owner),
                );
            }
            Err(e) => {
                log_event(
                    "UserRegistrationFailed",
                    format!("User {} failed: {:?}", i, e),
                );
                return Err(e);
            }
        }
    }
    log_event(
        "RegisterUsersCompleted",
        format!("Registered {} users", users.len()),
    );
    ic_cdk::println!("Completed register_dummy_users");
    Ok(users)
}

#[update]
fn perform_dummy_claim_referral(users: Vec<Account>) -> Result<(), LedgerError> {
    let caller = caller();
    if !is_admin(caller) {
        return Err(LedgerError::Unauthorized);
    }
    ic_cdk::println!("Step 2: perform_dummy_claim_referral started");
    log_event(
        "ClaimReferralsStarted",
        "Starting dummy referral claims".to_string(),
    );

    // For simplicity, using nested loops; in production, consider batching.
    for i in 1..users.len() {
        let referrer = users[i - 1].clone();
        let referee = users[i].clone();
        let referral_code = REFERRAL_BY_ACCOUNT.with(|rba| rba.borrow().get(&referrer).clone());
        match referral_code {
            Some(code) => match claim_referral(code, referee.clone()) {
                Ok(msg) => {
                    log_event(
                        "ReferralClaimed",
                        format!(
                            "Referrer: {}, Referee: {}, Message: {}",
                            referrer.owner, referee.owner, msg
                        ),
                    );
                }
                Err(e) => {
                    log_event(
                        "ReferralClaimFailed",
                        format!(
                            "Referrer: {}, Referee: {}, Error: {:?}",
                            referrer.owner, referee.owner, e
                        ),
                    );
                    return Err(e);
                }
            },
            None => {
                log_event(
                    "ReferralCodeNotFound",
                    format!("No referral code found for referrer: {}", referrer.owner),
                );
                return Err(LedgerError::InvalidReferral);
            }
        }
    }
    log_event(
        "ClaimReferralsCompleted",
        "All referral claims completed".to_string(),
    );
    ic_cdk::println!("Completed perform_dummy_claim_referral");
    Ok(())
}

#[update]
fn perform_dummy_approvals(users: Vec<Account>, approve_amount: u128) -> Result<(), LedgerError> {
    let caller = caller();
    if !is_admin(caller) {
        return Err(LedgerError::Unauthorized);
    }
    ic_cdk::println!("Step 3: perform_dummy_approvals started");
    log_event("ApprovalsStarted", "Starting dummy approvals".to_string());
    for i in 0..users.len() {
        for j in 0..users.len() {
            if i != j {
                let owner = users[i].clone();
                let spender = users[j].clone();
                match icrc1_approve(ApproveArgs {
                    from_subaccount: owner.subaccount,
                    spender: spender.clone(),
                    amount: Nat::from(approve_amount),
                }) {
                    Ok(_) => log_event(
                        "Approval",
                        format!("Owner: {}, Spender: {}", owner.owner, spender.owner),
                    ),
                    Err(e) => {
                        log_event(
                            "ApprovalFailed",
                            format!(
                                "Owner: {}, Spender: {}, Error: {:?}",
                                owner.owner, spender.owner, e
                            ),
                        );
                        return Err(e);
                    }
                }
            }
        }
    }
    log_event("ApprovalsCompleted", "All approvals completed".to_string());
    Ok(())
}

#[update]
async fn generate_dummy_data(
    num_users: u32,
    transfer_amount: u128,
    approve_amount: u128,
) -> Result<String, LedgerError> {
    let caller = caller();
    if !is_admin(caller) {
        return Err(LedgerError::Unauthorized);
    }
    ic_cdk::println!("Starting dummy data generation:");

    // Generate sufficient random bytes once
    let num_referral_codes = num_users;
    let num_tx_ids = num_users * (num_users - 1) * 2; // Transfers + Transfer-from
    let total_bytes_needed = num_referral_codes as usize * 10 + num_tx_ids as usize * 32;
    let (mut rand_bytes,) = raw_rand().await.unwrap();
    while rand_bytes.len() < total_bytes_needed {
        let (more_bytes,) = raw_rand().await.unwrap();
        rand_bytes.extend(more_bytes);
    }
    let mut rand_index = 0;

    // Helper function to get next n bytes
    fn get_next_bytes(rand_bytes: &Vec<u8>, n: usize, index: &mut usize) -> Vec<u8> {
        let start = *index;
        let end = start + n;
        if end > rand_bytes.len() {
            panic!("Not enough random bytes");
        }
        let bytes = rand_bytes[start..end].to_vec();
        *index = end;
        bytes
    }

    // Step 1: Register users
    let users = {
        let mut users = Vec::new();
        let run_timestamp = current_time();
        ic_cdk::println!("Step 1: Starting new users registration");
        log_event(
            "RegisterUsersStarted",
            format!("Registering {} users", num_users),
        );

        for i in 0..num_users {
            let mut subaccount = [0u8; 32];
            subaccount[0..8].copy_from_slice(&run_timestamp.to_le_bytes());
            subaccount[8..12].copy_from_slice(&i.to_le_bytes());
            let dummy_account = Account {
                owner: caller,
                subaccount: Some(subaccount),
            };
            let referral_bytes = get_next_bytes(&rand_bytes, 10, &mut rand_index);
            match register_user(dummy_account.clone(), Some(referral_bytes)).await {
                Ok(_) => {
                    users.push(dummy_account.clone());
                    log_event(
                        "UserRegistered",
                        format!("User {}: {}", i, dummy_account.owner),
                    );
                }
                Err(e) => return Err(e),
            }
        }
        ic_cdk::println!("Registering new users completed");
        log_event(
            "RegisterUsersCompleted",
            format!("Registered {} users", users.len()),
        );
        users
    };

    // Step 2: Referral claims
    if let Err(e) = perform_dummy_claim_referral(users.clone()) {
        return Err(e);
    }
    ic_cdk::println!("Referral claims operations completed");

    // Step 3: Approvals
    if let Err(e) = perform_dummy_approvals(users.clone(), approve_amount) {
        return Err(e);
    }
    ic_cdk::println!("Approvals operations completed");

    // Step 4: Transfer-from
    {
        ic_cdk::println!("Step 4: perform_dummy_transfer_from started");
        for i in 0..users.len() {
            for j in 0..users.len() {
                if i != j {
                    let spender = users[i].clone();
                    let from = users[j].clone();
                    let to = users[(i + j) % users.len()].clone();
                    let tx_bytes = get_next_bytes(&rand_bytes, 32, &mut rand_index);
                    match icrc1_transfer_from(
                        TransferFromArgs {
                            spender: spender.clone(),
                            from: from.clone(),
                            to: to.clone(),
                            amount: Nat::from(transfer_amount),
                        },
                        Some(tx_bytes),
                    )
                    .await
                    {
                        Ok(_) => log_event(
                            "TransferFrom",
                            format!(
                                "Spender: {}, From: {}, To: {}",
                                spender.owner, from.owner, to.owner
                            ),
                        ),
                        Err(e) => return Err(e),
                    }
                }
            }
        }
        log_event(
            "TransferFromCompleted",
            "All transfer-from operations completed".to_string(),
        );
    }
    ic_cdk::println!("Transfer-from operations completed");

    // Step 5: Transfers
    {
        ic_cdk::println!("Step 5: perform_dummy_transfers started");
        for i in 0..users.len() {
            for j in 0..users.len() {
                if i != j {
                    let from = users[i].clone();
                    let to = users[j].clone();
                    let tx_bytes = get_next_bytes(&rand_bytes, 32, &mut rand_index);
                    match icrc1_transfer(
                        TransferArgs {
                            from_subaccount: from.subaccount,
                            to: to.clone(),
                            amount: Nat::from(transfer_amount),
                        },
                        Some(tx_bytes),
                    )
                    .await
                    {
                        Ok(_) => log_event(
                            "Transfer",
                            format!("From: {}, To: {}", from.owner, to.owner),
                        ),
                        Err(e) => return Err(e),
                    }
                }
            }
        }
        log_event("TransfersCompleted", "All transfers completed".to_string());
    }
    ic_cdk::println!("Direct transfers operations completed");

    Ok("Dummy data generation completed successfully".to_string())
}

// -------------------------
// Query Functions
// -------------------------

#[query]
fn get_admin() -> Principal {
    admin_principal()
}

#[query]
fn get_referral_code(user: Account) -> Option<String> {
    REFERRAL_BY_ACCOUNT.with(|rba| rba.borrow().get(&user).clone())
}

#[query]
fn icrc1_supported_standards() -> Vec<String> {
    vec!["ICRC-1".to_string(), "ICRC-2".to_string()]
}

#[query]
fn icrc1_metadata() -> Vec<(String, String)> {
    let meta = METADATA.with(|m| m.borrow().get(&0).unwrap().clone());
    vec![
        ("icrc1:name".to_string(), meta.name),
        ("icrc1:symbol".to_string(), meta.symbol),
        ("icrc1:decimals".to_string(), meta.decimals.to_string()),
        (
            "icrc1:logo".to_string(),
            "https://your-logo-url.com/token.png".to_string(),
        ),
    ]
}

#[query]
fn icrc1_allowance(owner: Account, spender: Account) -> Nat {
    let key = AllowanceKey { owner, spender };
    Nat::from(ALLOWANCES.with(|a| a.borrow().get(&key).unwrap_or(0)))
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
fn icrc1_total_supply() -> Nat {
    Nat::from(METADATA.with(|m| m.borrow().get(&0).unwrap().total_supply))
}

#[query]
fn icrc1_fee() -> Nat {
    Nat::from(METADATA.with(|m| m.borrow().get(&0).unwrap().transfer_fee))
}

#[query]
fn icrc1_balance_of(account: Account) -> Nat {
    Nat::from(BALANCES.with(|b| b.borrow().get(&account).unwrap_or(0)))
}

#[query]
fn get_community_pool_balance() -> Nat {
    let community_account = Account {
        owner: admin_principal(),
        subaccount: Some(COMMUNITY_POOL_SUBACCOUNT),
    };
    Nat::from(BALANCES.with(|b| b.borrow().get(&community_account).unwrap_or(0)))
}

#[query]
fn get_team_pool_balance() -> Nat {
    let team_account = Account {
        owner: admin_principal(),
        subaccount: Some(TEAM_VESTING_POOL_SUBACCOUNT),
    };
    Nat::from(BALANCES.with(|b| b.borrow().get(&team_account).unwrap_or(0)))
}

#[query]
fn get_reserve_pool_balance() -> Nat {
    let reserve_account = Account {
        owner: admin_principal(),
        subaccount: Some(RESERVE_POOL_SUBACCOUNT),
    };
    Nat::from(BALANCES.with(|b| b.borrow().get(&reserve_account).unwrap_or(0)))
}

#[query]
fn get_total_burned() -> Nat {
    Nat::from(METADATA.with(|m| m.borrow().get(&0).unwrap().total_burned))
}

#[query]
fn get_dapp_funds() -> Nat {
    let dapp_account = Account {
        owner: admin_principal(),
        subaccount: Some(DAPP_FUNDS_SUBACCOUNT),
    };
    Nat::from(BALANCES.with(|b| b.borrow().get(&dapp_account).unwrap_or(0)))
}

#[query]
fn get_fee_distribution() -> (Nat, Nat, Nat) {
    // Fee distribution percentages are derived from transfer_fee.
    let fee = METADATA.with(|m| m.borrow().get(&0).unwrap().transfer_fee);
    (
        Nat::from(fee * 20 / 100),
        Nat::from(fee * 10 / 100),
        Nat::from(fee * 70 / 100),
    )
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

#[query]
fn get_transactions_by_principal(
    principal: Principal,
    start_tx_id: [u8; 32],
    limit: u64,
) -> Vec<TransactionEvent> {
    TRANSACTIONS.with(|txs| {
        let txs = txs.borrow();
        let mut result = Vec::new();
        let mut count = 0;
        for (_, event) in txs.range(start_tx_id..) {
            if count >= limit {
                break;
            }
            let matches = event.from.owner == principal
                || event.to.as_ref().map_or(false, |a| a.owner == principal)
                || event
                    .spender
                    .as_ref()
                    .map_or(false, |a| a.owner == principal);
            if matches {
                result.push(event.clone());
                count += 1;
            }
        }
        result
    })
}

#[query]
fn get_transaction_by_id(tx_id: [u8; 32]) -> Option<TransactionEvent> {
    TRANSACTIONS.with(|txs| txs.borrow().get(&tx_id).clone())
}

#[query]
fn get_transactions(start: [u8; 32], end: [u8; 32]) -> Vec<TransactionEvent> {
    TRANSACTIONS.with(|txs| {
        txs.borrow()
            .range(start..=end)
            .map(|(_, event)| event.clone())
            .collect()
    })
}

// -------------------------
// Centralized Fee Processing & Logging
// -------------------------

// process_fee now updates the dedicated BALANCES accounts.
fn process_fee(fee: u128) -> Result<(), LedgerError> {
    let burn_amount = fee * 20 / 100;
    let pool_amount = fee * 10 / 100;
    let dapp_amount = fee * 70 / 100;

    // Update total supply and total burned.
    METADATA.with(|metadata| {
        let mut m = metadata.borrow_mut().get(&0).unwrap().clone();
        m.total_supply = m
            .total_supply
            .checked_sub(burn_amount)
            .ok_or(LedgerError::ArithmeticError)?;
        m.total_burned = m
            .total_burned
            .checked_add(burn_amount)
            .ok_or(LedgerError::ArithmeticError)?;
        metadata.borrow_mut().insert(0, m);
        Ok(())
    })?;

    // Credit pool amount to community pool account.
    let community_account = Account {
        owner: admin_principal(),
        subaccount: Some(COMMUNITY_POOL_SUBACCOUNT),
    };
    BALANCES.with(|balances| {
        let mut b = balances.borrow_mut();
        let current = b.get(&community_account).unwrap_or(0);
        let new_balance = current
            .checked_add(pool_amount)
            .ok_or(LedgerError::ArithmeticError)?;
        b.insert(community_account, new_balance);
        Ok(())
    })?;

    // Credit dapp_amount to the dapp funds account.
    let dapp_account = Account {
        owner: admin_principal(),
        subaccount: Some(DAPP_FUNDS_SUBACCOUNT),
    };
    BALANCES.with(|balances| {
        let mut b = balances.borrow_mut();
        let current = b.get(&dapp_account).unwrap_or(0);
        let new_balance = current
            .checked_add(dapp_amount)
            .ok_or(LedgerError::ArithmeticError)?;
        b.insert(dapp_account, new_balance);
        Ok(())
    })?;

    Ok(())
}

fn log_event(event_type: &str, details: String) {
    LOGS.with(|logs| {
        let mut logs = logs.borrow_mut();
        let timestamp = time();
        let counter = LOG_COUNTER.with(|c| {
            let mut counter_cell = c.borrow_mut();
            let count = *counter_cell.get();
            counter_cell
                .set(count + 1)
                .expect("Failed to update LOG_COUNTER");
            count
        });
        let unique_key = timestamp + counter as u64;

        logs.insert(
            unique_key,
            LogEntry {
                timestamp,
                event_type: event_type.to_string(),
                details,
            },
        );

        // Maintain a maximum size for logs.
        if logs.len() > 1_000_000 {
            if let Some((first_key, _)) = logs.first_key_value() {
                logs.remove(&first_key);
            }
        }
    });
}

// Export Candid interface
ic_cdk::export_candid!();
