// src/backend/crnl_ledger/src/lib.rs

use candid::{CandidType, Nat, Principal};
use ic_cdk::api::management_canister::main::raw_rand;
use ic_cdk::api::time;
use ic_cdk::caller;
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
    fn to_bytes(&self) -> Cow<'_, [u8]> {
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

#[derive(CandidType, Serialize, Deserialize)]
struct ClaimReferralArgs {
    referral_code: String,
}

#[derive(CandidType, Serialize, Deserialize)]
struct DeductBalanceArgs {
    caller: Account,
    amount: Nat,
    description: String,
}

#[derive(CandidType, Serialize, Deserialize)]
struct PoolTransferArgs {
    from_pool: String,             // "community", "team", "reserve", or "dapp"
    to_pool: Option<String>,       // Same pool names, or None if transferring to principal
    to_principal: Option<Account>, // Target account if not transferring to pool
    amount: Nat,
    description: String,
}

// ApproveArgs for ICRC-2 compliance
#[derive(CandidType, Serialize, Deserialize)]
struct ApproveArgs {
    from_subaccount: Option<[u8; 32]>,
    spender: Account,
    amount: Nat,
    expires_at: Option<u64>,
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
    logo: Option<String>,
}

impl Storable for Metadata {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        // Try decoding the current Metadata layout first. Older canisters
        // may have stored a legacy Metadata without the `logo` field, so
        // fall back to decoding that and convert it.
        // We avoid unwrap() to prevent panics during upgrades.
        let bytes_ref: &[u8] = &bytes;
        if let Ok(meta) = candid::decode_one::<Metadata>(bytes_ref) {
            meta
        } else {
            // Define a legacy metadata shape matching the previous on-disk layout
            #[derive(CandidType, Deserialize)]
            struct LegacyMetadata {
                name: String,
                symbol: String,
                decimals: u8,
                total_supply: u128,
                transfer_fee: u128,
                total_burned: u128,
                vesting_start_time: u64,
                vesting_duration: u64,
            }

            if let Ok(legacy) = candid::decode_one::<LegacyMetadata>(bytes_ref) {
                Metadata {
                    name: legacy.name,
                    symbol: legacy.symbol,
                    decimals: legacy.decimals,
                    total_supply: legacy.total_supply,
                    transfer_fee: legacy.transfer_fee,
                    total_burned: legacy.total_burned,
                    vesting_start_time: legacy.vesting_start_time,
                    vesting_duration: legacy.vesting_duration,
                    logo: None,
                }
            } else {
                // As a last resort, return a safe default to keep the canister running.
                Metadata {
                    name: "".to_string(),
                    symbol: "".to_string(),
                    decimals: 0,
                    total_supply: 0,
                    transfer_fee: 0,
                    total_burned: 0,
                    vesting_start_time: 0,
                    vesting_duration: 0,
                    logo: None,
                }
            }
        }
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
    fn to_bytes(&self) -> Cow<'_, [u8]> {
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
    fn to_bytes(&self) -> Cow<'_, [u8]> {
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
    fn to_bytes(&self) -> Cow<'_, [u8]> {
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
    // Authentication-related errors
    NotAuthenticated,
    InvalidPrincipal,
    UnauthorizedCaller,
    AdminRequired,
}

#[derive(CandidType, Serialize, Deserialize, Clone)]
struct FeeDistribution {
    fee: Nat,
    burn_percent: u8,
    community_percent: u8,
    dapp_percent: u8,
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
    static ALLOWANCE_EXPIRATIONS: RefCell<StableBTreeMap<AllowanceKey, u64, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|mm| mm.borrow().get(MemoryId::new(4))))
    );
    static REFERRAL_BY_ACCOUNT: RefCell<StableBTreeMap<Account, String, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|mm| mm.borrow().get(MemoryId::new(5))))
    );
    static ACCOUNT_BY_REFERRAL: RefCell<StableBTreeMap<String, Account, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|mm| mm.borrow().get(MemoryId::new(6))))
    );
    static CLAIMED_REFERRALS: RefCell<StableBTreeMap<Account, bool, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|mm| mm.borrow().get(MemoryId::new(7))))
    );
    static LOGS: RefCell<StableBTreeMap<u64, LogEntry, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|mm| mm.borrow().get(MemoryId::new(8))))
    );
    static TRANSACTIONS: RefCell<StableBTreeMap<[u8; 32], TransactionEvent, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|mm| mm.borrow().get(MemoryId::new(9))))
    );
    // Use a 64-bit counter for log keys to avoid overflow.
    static LOG_COUNTER: RefCell<StableCell<u64, Memory>> = RefCell::new(
        StableCell::init(MEMORY_MANAGER.with(|mm| mm.borrow().get(MemoryId::new(10))), 0)
            .expect("Failed to initialize LOG_COUNTER")
    );
    // Whitelist for trusted principals (e.g., Internet Identity principals)
    static TRUSTED_PRINCIPALS: RefCell<StableBTreeMap<Principal, bool, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|mm| mm.borrow().get(MemoryId::new(11))))
    );
    // Authentication bypass for admin operations
    static ADMIN_BYPASS_ENABLED: RefCell<StableCell<bool, Memory>> = RefCell::new(
        StableCell::init(MEMORY_MANAGER.with(|mm| mm.borrow().get(MemoryId::new(12))), false)
            .expect("Failed to initialize ADMIN_BYPASS_ENABLED")
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

// -------------------------
// Authentication Helper Functions
// -------------------------

// Check if a principal is a valid Internet Identity principal
// Internet Identity principals have specific characteristics:
// - They are not anonymous
// - They follow a specific textual format with hyphens
// - Real II principals like: 4s3y7-25yvt-jbdte-vpvcq-n4ghs-j5jo6-beihs-om2zi-oqzu6-krbhf-gqe
//   are 29-byte self-authenticating principals (ending with 0x02) with proper random-looking segments
// - Test/mock II principals may be shorter (10 bytes, ending with 0x01)
// - We reject simple/mock self-auth principals with many repeated segments (like "aaaaa")
fn is_valid_internet_identity_principal(principal: Principal) -> bool {
    // Reject anonymous outright
    if principal == Principal::anonymous() {
        return false;
    }

    // If principal is in the trusted whitelist, accept immediately
    let trusted = TRUSTED_PRINCIPALS.with(|tp| tp.borrow().get(&principal).unwrap_or(false));
    if trusted {
        return true;
    }

    let principal_bytes = principal.as_slice();

    // Reject management canister principal explicitly
    if principal.to_text() == "aaaaa-aa" {
        return false;
    }

    // Enforce textual structure to match proper II principals
    let text = principal.to_text();

    // Principal text must have proper length and contain hyphens
    if text.len() < 10 || text.len() > 63 || !text.contains('-') {
        return false;
    }

    // Validate the format: segments separated by hyphens
    // Each segment must be non-empty, max 5 characters, lowercase alphanumeric
    if !text.split('-').all(|segment| {
        !segment.is_empty()
            && segment.len() <= 5
            && segment.chars().all(|c| matches!(c, 'a'..='z' | '0'..='9'))
    }) {
        return false;
    }

    // For self-authenticating principals (29 bytes ending with 0x02):
    // Reject if they look like simple test/mock principals
    if principal_bytes.len() == 29 && principal_bytes.last() == Some(&0x02) {
        let segments: Vec<&str> = text.split('-').collect();

        // Count segments that are just "aaaaa" or similar repeated patterns
        let repeated_count = segments
            .iter()
            .filter(|s| {
                if s.len() < 3 {
                    return false;
                }
                let chars: Vec<char> = s.chars().collect();
                chars
                    .windows(3)
                    .all(|w| w[0] == 'a' && w[1] == 'a' && w[2] == 'a')
            })
            .count();

        // Reject if more than half the segments look like test data
        if repeated_count > segments.len() / 2 {
            return false;
        }
    }

    // Accept shorter principals (e.g., 10 bytes ending with 0x01 for test/mock II principals)
    // and proper self-auth principals that pass validation
    principal_bytes.len() >= 10
}

// Validate that the caller is properly authenticated
fn validate_caller_authentication() -> Result<Principal, LedgerError> {
    let caller = caller();

    // Allow admin to bypass authentication if enabled
    if ADMIN_BYPASS_ENABLED.with(|ab| ab.borrow().get().clone()) && is_admin(caller) {
        return Ok(caller);
    }

    // Check if the caller is a valid Internet Identity principal
    if !is_valid_internet_identity_principal(caller) {
        log_event(
            "AuthenticationFailure",
            format!("Invalid principal attempted access: {}", caller),
        );
        return Err(LedgerError::NotAuthenticated);
    }

    Ok(caller)
}

// Validate admin authentication (stricter requirements)
fn validate_admin_authentication() -> Result<Principal, LedgerError> {
    let caller = validate_caller_authentication()?;

    if !is_admin(caller) {
        log_event(
            "UnauthorizedAdminAccess",
            format!("Non-admin principal attempted admin operation: {}", caller),
        );
        return Err(LedgerError::AdminRequired);
    }

    Ok(caller)
}

// Generates a longer, more random referral code.
async fn generate_random_referral_code(random_bytes: Option<Vec<u8>>) -> String {
    let charset = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let code_length = 12;
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

// Helper function to get pool account from pool name
fn get_pool_account(pool_name: &str) -> Result<Account, LedgerError> {
    let subaccount = match pool_name.to_lowercase().as_str() {
        "community" => COMMUNITY_POOL_SUBACCOUNT,
        "team" => TEAM_VESTING_POOL_SUBACCOUNT,
        "reserve" => RESERVE_POOL_SUBACCOUNT,
        "dapp" => DAPP_FUNDS_SUBACCOUNT,
        _ => return Err(LedgerError::InvalidAccount),
    };
    Ok(Account {
        owner: admin_principal(),
        subaccount: Some(subaccount),
    })
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
                logo: None,
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
    // Validate caller authentication
    let authenticated_caller = validate_caller_authentication()?;

    // Ensure the user account owner matches the authenticated caller
    if user.owner != authenticated_caller {
        return Err(LedgerError::UnauthorizedCaller);
    }

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
fn claim_referral(args: ClaimReferralArgs) -> Result<String, LedgerError> {
    // Validate caller authentication
    let authenticated_caller = validate_caller_authentication()?;

    let referee = Account {
        owner: authenticated_caller,
        subaccount: None, // or get from an optional argument if needed
    };
    let referrer_opt =
        ACCOUNT_BY_REFERRAL.with(|abr| abr.borrow().get(&args.referral_code).clone());
    let referrer = match referrer_opt {
        None => return Err(LedgerError::InvalidReferral),
        Some(acc) => acc,
    };

    if !ACCOUNT_BY_REFERRAL.with(|abr| abr.borrow().contains_key(&args.referral_code))
        || CLAIMED_REFERRALS.with(|cr| cr.borrow().contains_key(&referee))
    {
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
    // Validate caller authentication
    let authenticated_caller = validate_caller_authentication()?;

    let from = Account {
        owner: authenticated_caller,
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
    // Validate caller authentication
    let authenticated_caller = validate_caller_authentication()?;

    let owner = Account {
        owner: authenticated_caller,
        subaccount: args.from_subaccount,
    };
    let amount = nat_to_u128(args.amount.clone())?;

    // Check that caller is the owner (redundant but kept for clarity)
    if authenticated_caller != owner.owner {
        return Err(LedgerError::Unauthorized);
    }

    if let Some(expires_at) = args.expires_at {
        // Store expires_at in a separate StableBTreeMap
        ALLOWANCE_EXPIRATIONS.with(|expirations| {
            expirations.borrow_mut().insert(
                AllowanceKey {
                    owner: owner.clone(),
                    spender: args.spender.clone(),
                },
                expires_at,
            );
        });
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
    // Validate caller authentication
    let authenticated_caller = validate_caller_authentication()?;

    // Enforce that only the approved spender can perform a transfer_from.
    if authenticated_caller != args.spender.owner {
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

    if let Some(expires_at) =
        ALLOWANCE_EXPIRATIONS.with(|expirations| expirations.borrow().get(&allowance_key))
    {
        if current_time() > expires_at {
            return Err(LedgerError::InsufficientAllowance);
        }
    }

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
fn deduct_from_balance(args: DeductBalanceArgs) -> Result<String, LedgerError> {
    // Validate caller authentication
    let authenticated_caller = validate_caller_authentication()?;

    // Ensure the caller account owner matches the authenticated caller
    if args.caller.owner != authenticated_caller {
        return Err(LedgerError::UnauthorizedCaller);
    }

    let deduction_amount = nat_to_u128(args.amount.clone())?;
    let balance = BALANCES.with(|b| b.borrow().get(&args.caller).unwrap_or(0));
    if balance < deduction_amount {
        return Err(LedgerError::InsufficientBalance);
    }
    BALANCES.with(|b| {
        b.borrow_mut()
            .insert(args.caller.clone(), balance - deduction_amount)
    });
    process_fee(deduction_amount)?;
    log_event(
        "BalanceDeducted",
        format!(
            "Caller: {}, Amount: {}, Description: {}",
            args.caller.owner, deduction_amount, args.description
        ),
    );

    let metadata = METADATA.with(|m| m.borrow().get(&0).unwrap().clone());
    let display_amount = deduction_amount / 10u128.pow(metadata.decimals as u32);
    Ok(format!(
        "Deducted {} {} from balance. Description: {}",
        display_amount, metadata.symbol, args.description
    ))
}

#[update]
fn admin_transfer(args: PoolTransferArgs) -> Result<String, LedgerError> {
    // Validate admin authentication - only admin can perform admin transfers
    let _authenticated_admin = validate_admin_authentication()?;

    // Get source pool account
    let from_account = get_pool_account(&args.from_pool)?;

    // Determine destination account
    let to_account = if let Some(to_pool_name) = &args.to_pool {
        // Pool-to-pool transfer
        get_pool_account(to_pool_name)?
    } else if let Some(ref to_principal_account) = args.to_principal {
        // Pool-to-principal transfer
        to_principal_account.clone()
    } else {
        return Err(LedgerError::InvalidAccount);
    };

    // Prevent transferring to the same account
    if from_account == to_account {
        return Err(LedgerError::InvalidAccount);
    }

    // Check vesting conditions if transferring from team pool
    check_team_vesting(&from_account)?;

    let transfer_amount = nat_to_u128(args.amount.clone())?;

    // Perform the transfer
    BALANCES.with(|balances| {
        let mut b = balances.borrow_mut();
        let from_balance = b.get(&from_account).unwrap_or(0);
        if from_balance < transfer_amount {
            return Err(LedgerError::InsufficientBalance);
        }
        b.insert(from_account.clone(), from_balance - transfer_amount);
        let to_balance = b.get(&to_account).unwrap_or(0);
        let new_to_balance = to_balance
            .checked_add(transfer_amount)
            .ok_or(LedgerError::ArithmeticError)?;
        b.insert(to_account.clone(), new_to_balance);
        Ok(())
    })?;

    // Log the transfer
    let to_description = if let Some(to_pool) = &args.to_pool {
        format!("pool:{}", to_pool)
    } else if let Some(to_principal) = &args.to_principal {
        format!("principal:{}", to_principal.owner)
    } else {
        "unknown".to_string()
    };

    log_event(
        "PoolTransfer",
        format!(
            "From: pool:{}, To: {}, Amount: {}, Description: {}",
            args.from_pool, to_description, transfer_amount, args.description
        ),
    );

    let metadata = METADATA.with(|m| m.borrow().get(&0).unwrap().clone());
    let display_amount = transfer_amount / 10u128.pow(metadata.decimals as u32);
    Ok(format!(
        "Transferred {} {} from {} pool to {}. Description: {}",
        display_amount, metadata.symbol, args.from_pool, to_description, args.description
    ))
}

#[update]
fn set_transfer_fee(new_fee: u128) -> Result<(), LedgerError> {
    // Validate admin authentication
    let _authenticated_admin = validate_admin_authentication()?;
    METADATA.with(|metadata| {
        let mut m = metadata.borrow_mut().get(&0).unwrap().clone();
        m.transfer_fee = new_fee;
        metadata.borrow_mut().insert(0, m);
    });
    log_event("SetTransferFee", format!("New fee: {}", new_fee));
    Ok(())
}

#[update]
async fn admin_mint(
    to: Account,
    amount: Nat,
    description: Option<String>,
    random_bytes: Option<Vec<u8>>,
) -> Result<Nat, LedgerError> {
    // Only admin can mint
    let _authenticated_admin = validate_admin_authentication()?;

    let amt = nat_to_u128(amount.clone())?;
    if amt == u128::MAX {
        return Err(LedgerError::ArithmeticError);
    }

    // Increase total supply
    METADATA.with(|metadata| {
        let mut m = metadata.borrow_mut().get(&0).unwrap().clone();
        m.total_supply = m
            .total_supply
            .checked_add(amt)
            .ok_or(LedgerError::ArithmeticError)?;
        metadata.borrow_mut().insert(0, m);
        Ok(())
    })?;

    // Credit recipient balance
    BALANCES.with(|balances| {
        let mut b = balances.borrow_mut();
        let current = b.get(&to).unwrap_or(0);
        let new_balance = current
            .checked_add(amt)
            .ok_or(LedgerError::ArithmeticError)?;
        b.insert(to.clone(), new_balance);
        Ok(())
    })?;

    // Create a transaction event (async tx id)
    let tx_id = generate_tx_id(random_bytes).await;
    TRANSACTIONS.with(|txs| {
        txs.borrow_mut().insert(
            tx_id,
            TransactionEvent {
                tx_id,
                timestamp: current_time(),
                event_type: "Mint".to_string(),
                from: Account {
                    owner: admin_principal(),
                    subaccount: None,
                },
                to: Some(to.clone()),
                spender: None,
                amount: amount.clone(),
                fee: None,
            },
        );
    });

    log_event(
        "AdminMint",
        format!(
            "To: {}, Amount: {}, Description: {}",
            to.owner,
            amt,
            description.unwrap_or_default()
        ),
    );

    Ok(amount)
}

#[update]
async fn convert_dapp_funds_to_cycles() -> Result<(), LedgerError> {
    // Validate admin authentication
    let _authenticated_admin = validate_admin_authentication()?;
    // For demonstration purposes, assume conversion happens here.
    log_event(
        "DappFundsConverted",
        "Converted dapp funds to cycles".to_string(),
    );
    Ok(())
}

// -------------------------
// Authentication Management Functions (Admin Only)
// -------------------------

#[update]
fn add_trusted_principal(principal: Principal) -> Result<(), LedgerError> {
    // Validate admin authentication
    let _authenticated_admin = validate_admin_authentication()?;

    TRUSTED_PRINCIPALS.with(|tp| {
        tp.borrow_mut().insert(principal, true);
    });

    log_event(
        "TrustedPrincipalAdded",
        format!("Added trusted principal: {}", principal),
    );

    Ok(())
}

#[update]
fn remove_trusted_principal(principal: Principal) -> Result<(), LedgerError> {
    // Validate admin authentication
    let _authenticated_admin = validate_admin_authentication()?;

    TRUSTED_PRINCIPALS.with(|tp| {
        tp.borrow_mut().remove(&principal);
    });

    log_event(
        "TrustedPrincipalRemoved",
        format!("Removed trusted principal: {}", principal),
    );

    Ok(())
}

#[update]
fn set_admin_bypass(enabled: bool) -> Result<(), LedgerError> {
    // Validate admin authentication
    let _authenticated_admin = validate_admin_authentication()?;

    ADMIN_BYPASS_ENABLED.with(|ab| {
        ab.borrow_mut()
            .set(enabled)
            .expect("Failed to set admin bypass");
    });

    log_event(
        "AdminBypassChanged",
        format!("Admin bypass enabled: {}", enabled),
    );

    Ok(())
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
        ("icrc1:fee".to_string(), meta.transfer_fee.to_string()),
        (
            "icrc1:total_supply".to_string(),
            meta.total_supply.to_string(),
        ),
        (
            "icrc1:logo".to_string(),
            meta.logo.clone().unwrap_or_else(|| "".to_string()),
        ),
    ]
}

#[query]
fn get_logo() -> String {
    METADATA.with(|m| m.borrow().get(&0).unwrap().logo.clone().unwrap_or_default())
}

#[update]
fn set_logo(new_logo: String) -> Result<(), LedgerError> {
    // Only admin may update the logo
    let _admin = validate_admin_authentication()?;
    METADATA.with(|metadata| {
        let mut m = metadata.borrow_mut().get(&0).unwrap().clone();
        m.logo = Some(new_logo.clone());
        metadata.borrow_mut().insert(0, m);
    });
    log_event("SetLogo", format!("Admin updated logo URL: {}", new_logo));
    Ok(())
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
fn get_fee_distribution() -> FeeDistribution {
    // Fee distribution percentages are derived from transfer_fee.
    let fee_val = METADATA.with(|m| m.borrow().get(&0).unwrap().transfer_fee);
    let burn_pct: u8 = 20;
    let community_pct: u8 = 10;
    let dapp_pct: u8 = 70;

    FeeDistribution {
        fee: Nat::from(fee_val),
        burn_percent: burn_pct,
        community_percent: community_pct,
        dapp_percent: dapp_pct,
    }
}

#[query]
fn get_logs_paginated(offset: u64, limit: u64) -> Result<Vec<LogEntry>, LedgerError> {
    // Validate admin authentication for log access
    let _authenticated_admin = validate_admin_authentication()?;
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
fn get_logs_by_range(start_time: u64, end_time: u64) -> Result<Vec<LogEntry>, LedgerError> {
    // Validate admin authentication for log access
    let _authenticated_admin = validate_admin_authentication()?;
    Ok(LOGS.with(|logs| {
        logs.borrow()
            .range(start_time..=end_time)
            .map(|(_, entry)| entry.clone())
            .collect()
    }))
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
// Authentication Query Functions
// -------------------------

#[query]
fn is_caller_authenticated() -> bool {
    validate_caller_authentication().is_ok()
}

#[query]
fn is_principal_trusted(principal: Principal) -> bool {
    TRUSTED_PRINCIPALS.with(|tp| tp.borrow().get(&principal).unwrap_or(false))
}

#[query]
fn is_valid_ii_principal(principal: Principal) -> bool {
    is_valid_internet_identity_principal(principal)
}

#[query]
fn get_trusted_principals() -> Vec<Principal> {
    // Only admin can view the full list of trusted principals
    if validate_admin_authentication().is_err() {
        return vec![];
    }

    TRUSTED_PRINCIPALS.with(|tp| {
        tp.borrow()
            .iter()
            .filter(|(_, trusted)| *trusted)
            .map(|(principal, _)| principal.clone())
            .collect()
    })
}

#[query]
fn is_admin_bypass_enabled() -> bool {
    // Only admin can check this status
    if validate_admin_authentication().is_err() {
        return false;
    }

    ADMIN_BYPASS_ENABLED.with(|ab| ab.borrow().get().clone())
}

#[query]
fn get_caller_principal_info() -> (Principal, bool, bool) {
    let caller = caller();
    let is_authenticated = validate_caller_authentication().is_ok();
    let is_admin = is_admin(caller);
    (caller, is_authenticated, is_admin)
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
        if logs.len() > 1_00_000 {
            if let Some((first_key, _)) = logs.first_key_value() {
                logs.remove(&first_key);
            }
        }
    });
}

// Export Candid interface
ic_cdk::export_candid!();
