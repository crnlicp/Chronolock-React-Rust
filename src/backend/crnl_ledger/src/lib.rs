use candid::{CandidType, Nat, Principal};
use ic_cdk::api::management_canister::main::raw_rand;
use ic_cdk_macros::{init, query, update};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, Storable};
use num_traits::ToPrimitive;
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
    static DAPP_FUNDS: RefCell<StableBTreeMap<u8, u128, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|mm| mm.borrow().get(MemoryId::new(8))))
    );
    static ALLOWANCES: RefCell<StableBTreeMap<AllowanceKey, u128, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|mm| mm.borrow().get(MemoryId::new(2))))
    );
    static LOGS: RefCell<StableBTreeMap<u64, LogEntry, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|mm| mm.borrow().get(MemoryId::new(3))))
    );
    static REFERRAL_BY_ACCOUNT: RefCell<StableBTreeMap<Account, String, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|mm| mm.borrow().get(MemoryId::new(4))))
    );
    static ACCOUNT_BY_REFERRAL: RefCell<StableBTreeMap<String, Account, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|mm| mm.borrow().get(MemoryId::new(5))))
    );
    static ADMIN_STORAGE: RefCell<StableBTreeMap<u8, Principal, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|mm| mm.borrow().get(MemoryId::new(9))))
    );
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

fn current_time() -> u64 {
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

async fn generate_random_referral_code() -> String {
    let charset = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    let (random_bytes,) = raw_rand().await.unwrap();
    random_bytes
        .iter()
        .take(7)
        .map(|b| {
            let index = (*b as usize) % charset.len();
            charset[index] as char
        })
        .collect()
}

// Convert Nat to u128 safely
fn nat_to_u128(n: Nat) -> Result<u128, LedgerError> {
    n.0.to_u128().ok_or(LedgerError::ArithmeticError)
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
    let community_pool_amount = total_supply * 50 / 100;
    let team_vesting_pool_amount = total_supply * 20 / 100;
    let reserve_amount = total_supply * 30 / 100;

    METADATA.with(|metadata| {
        let mut m = metadata.borrow_mut();
        m.insert(
            0,
            Metadata {
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
            },
        );
    });

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

    BALANCES.with(|balances| {
        let mut b = balances.borrow_mut();
        b.insert(community_account, community_pool_amount);
        b.insert(team_account, team_vesting_pool_amount);
        b.insert(reserve_account, reserve_amount);
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
        let new_community_pool = m
            .community_pool
            .checked_sub(welcome_amount)
            .ok_or(LedgerError::ArithmeticError)?;
        BALANCES.with(|b| b.borrow_mut().insert(user.clone(), welcome_amount));
        let mut updated_metadata = m.clone();
        updated_metadata.community_pool = new_community_pool;
        metadata_ref.insert(0, updated_metadata);

        let subaccount_str = match user.subaccount {
            Some(sub) => {
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
            }
            None => "None".to_string(),
        };

        log_event(
            "UserRegistered",
            format!(
                "Account: {}, Subaccount: {}, Amount: {}",
                user.owner, subaccount_str, welcome_amount
            ),
        );
        Ok(m)
    })?;

    if !REFERRAL_BY_ACCOUNT.with(|rba| rba.borrow().contains_key(&user)) {
        let mut referral_code = generate_random_referral_code().await;
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
        let subaccount_str = match user.subaccount {
            Some(sub) => {
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
            }
            None => "None".to_string(),
        };
        log_event(
            "ReferralRegistered",
            format!(
                "User: {}, Subaccount: {}, Referral_Code: {}",
                user.owner, subaccount_str, referral_code
            ),
        );
        return Ok(format!(
            "User registered with 200 {}. Your referral code is: {}",
            symbol, referral_code
        ));
    }
    let symbol = METADATA.with(|metadata| metadata.borrow().get(&0).unwrap().symbol.clone());
    Ok(format!("User registered with 200 {}", symbol))
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

    METADATA.with(|metadata| {
        let mut metadata_ref = metadata.borrow_mut();
        let mut m = metadata_ref.get(&0).unwrap().clone();
        let reward = 20 * 10u128.pow(m.decimals as u32);
        if m.community_pool < reward {
            return Err(LedgerError::InsufficientPoolFunds);
        }
        m.community_pool = m
            .community_pool
            .checked_sub(reward)
            .ok_or(LedgerError::ArithmeticError)?;
        let referrer_balance = BALANCES.with(|b| b.borrow().get(&referrer).unwrap_or(0));
        let new_balance = referrer_balance
            .checked_add(reward)
            .ok_or(LedgerError::ArithmeticError)?;
        BALANCES.with(|b| b.borrow_mut().insert(referrer.clone(), new_balance));
        metadata_ref.insert(0, m.clone());
        log_event(
            "ReferralClaimed",
            format!(
                "Referrer: {}, Referee: {}, Reward: {}",
                referrer.owner, referee.owner, reward
            ),
        );
        CLAIMED_REFERRALS.with(|cr| {
            cr.borrow_mut().insert(referee.clone(), true);
        });
        Ok(format!("Referral reward of 20 {} credited", m.symbol))
    })
}

#[update]
fn icrc1_transfer(args: TransferArgs) -> Result<Nat, LedgerError> {
    let caller = caller();
    let from = Account {
        owner: caller,
        subaccount: args.from_subaccount,
    };
    let amount = nat_to_u128(args.amount.clone())?;

    METADATA.with(|metadata| {
        let m = metadata.borrow().get(&0).unwrap();
        let transfer_fee = m.transfer_fee;

        if amount == u128::MAX {
            return Err(LedgerError::ArithmeticError);
        }

        if amount < transfer_fee {
            return Err(LedgerError::InsufficientFee);
        }

        let amount_after_fee = amount
            .checked_sub(transfer_fee)
            .ok_or(LedgerError::ArithmeticError)?;

        let from_balance = BALANCES.with(|b| b.borrow().get(&from).unwrap_or(0));
        if from_balance < amount {
            return Err(LedgerError::InsufficientBalance);
        }

        BALANCES.with(|b| {
            let mut b = b.borrow_mut();
            b.insert(from.clone(), from_balance - amount);
            let to_balance = b.get(&args.to).unwrap_or(0);
            let new_to_balance = to_balance
                .checked_add(amount_after_fee)
                .ok_or(LedgerError::ArithmeticError)?;
            b.insert(args.to.clone(), new_to_balance);
            Ok(())
        })?;

        process_fee(transfer_fee)?;
        log_event(
            "Transfer",
            format!(
                "From: {}, To: {}, Amount: {}, Fee: {}, Received: {}",
                from.owner, args.to.owner, amount, transfer_fee, amount_after_fee
            ),
        );
        Ok(args.amount)
    })
}

#[update]
fn icrc1_approve(args: ApproveArgs) -> Result<Nat, LedgerError> {
    let caller = caller();
    let owner = Account {
        owner: caller,
        subaccount: args.from_subaccount,
    };
    let amount = nat_to_u128(args.amount.clone())?;

    // Since caller is the owner principal, this check may be redundant but kept for safety
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
    log_event(
        "Approval",
        format!(
            "Owner: {}, Subaccount: {:?}, Spender: {}, Amount: {}",
            owner.owner, owner.subaccount, args.spender.owner, amount
        ),
    );
    Ok(args.amount)
}

#[update]
fn icrc1_transfer_from(args: TransferFromArgs) -> Result<Nat, LedgerError> {
    let amount = nat_to_u128(args.amount.clone())?;

    METADATA.with(|metadata| {
        let m = metadata.borrow().get(&0).unwrap();
        let transfer_fee = m.transfer_fee;

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
        log_event(
            "TransferFrom",
            format!(
                "Spender: {}, From: {}, To: {}, Amount: {}, Fee: {}, Received: {}",
                args.spender.owner,
                args.from.owner,
                args.to.owner,
                amount,
                transfer_fee,
                amount_after_fee
            ),
        );
        Ok(args.amount)
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
        BALANCES.with(|b| {
            b.borrow_mut()
                .insert(caller.clone(), balance - creation_fee)
        });
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

    let dapp_amount = DAPP_FUNDS.with(|df| df.borrow().get(&0).unwrap_or(0));
    if dapp_amount == 0 {
        return Ok(());
    }

    // TODO: Implement conversion to cycles after token launch
    log_event(
        "DappFundsConverted",
        format!("Converted {} tokens to cycles", dapp_amount),
    );
    Ok(())
}

#[update]
async fn add_dummy_data(
    num_users: u32,
    transfer_amount: u128,
    approve_amount: u128,
) -> Result<String, LedgerError> {
    let caller = caller();
    if !is_admin(caller) {
        return Err(LedgerError::Unauthorized);
    }

    let mut users = Vec::new();
    let mut previous_user: Option<Account> = None;
    let run_timestamp = current_time();

    for i in 0..num_users {
        let mut subaccount = [0u8; 32];
        subaccount[0..8].copy_from_slice(&run_timestamp.to_le_bytes());
        subaccount[8..12].copy_from_slice(&i.to_le_bytes());

        let dummy_account = Account {
            owner: caller,
            subaccount: Some(subaccount),
        };

        let register_result = register_user(dummy_account.clone()).await;
        match register_result {
            Ok(message) => {
                users.push(dummy_account.clone());
                log_event(
                    "DummyUserRegistered",
                    format!("User: {}, Message: {}", dummy_account.owner, message),
                );

                if let Some(prev) = previous_user.clone() {
                    let referral_code = get_referral_code(prev.clone()).unwrap_or_default();
                    if !referral_code.is_empty() {
                        let claim_result =
                            claim_referral(referral_code.clone(), dummy_account.clone());
                        match claim_result {
                            Ok(claim_message) => {
                                log_event(
                                    "DummyReferralClaimed",
                                    format!(
                                        "Referrer: {}, Referee: {}, Message: {}",
                                        prev.owner, dummy_account.owner, claim_message
                                    ),
                                );
                            }
                            Err(e) => {
                                log_event(
                                    "DummyReferralClaimFailed",
                                    format!(
                                        "Referrer: {}, Referee: {}, Error: {:?}",
                                        prev.owner, dummy_account.owner, e
                                    ),
                                );
                            }
                        }
                    }
                }
                previous_user = Some(dummy_account.clone());
            }
            Err(e) => {
                log_event(
                    "DummyUserRegistrationFailed",
                    format!("User: {}, Error: {:?}", dummy_account.owner, e),
                );
                return Err(e);
            }
        }
    }

    for i in 0..users.len() {
        for j in 0..users.len() {
            if i != j {
                let owner = users[i].clone();
                let spender = users[j].clone();

                let approve_result = icrc1_approve(ApproveArgs {
                    from_subaccount: owner.subaccount,
                    spender: spender.clone(),
                    amount: Nat::from(approve_amount),
                });
                match approve_result {
                    Ok(_) => {
                        log_event(
                            "DummyApproval",
                            format!(
                                "Owner: {}, Spender: {}, Amount: {}",
                                owner.owner, spender.owner, approve_amount
                            ),
                        );
                    }
                    Err(e) => {
                        log_event(
                            "DummyApprovalFailed",
                            format!(
                                "Owner: {}, Spender: {}, Amount: {}, Error: {:?}",
                                owner.owner, spender.owner, approve_amount, e
                            ),
                        );
                    }
                }
            }
        }
    }

    for i in 0..users.len() {
        for j in 0..users.len() {
            if i != j {
                let spender = users[i].clone();
                let from = users[j].clone();
                let to = users[(i + j) % users.len()].clone();

                let transfer_from_result = icrc1_transfer_from(TransferFromArgs {
                    spender: spender.clone(),
                    from: from.clone(),
                    to: to.clone(),
                    amount: Nat::from(transfer_amount),
                });
                match transfer_from_result {
                    Ok(_) => {
                        log_event(
                            "DummyTransferFrom",
                            format!(
                                "Spender: {}, From: {}, To: {}, Amount: {}",
                                spender.owner, from.owner, to.owner, transfer_amount
                            ),
                        );
                    }
                    Err(e) => {
                        log_event(
                            "DummyTransferFromFailed",
                            format!(
                                "Spender: {}, From: {}, To: {}, Amount: {}, Error: {:?}",
                                spender.owner, from.owner, to.owner, transfer_amount, e
                            ),
                        );
                    }
                }
            }
        }
    }

    for i in 0..users.len() {
        for j in 0..users.len() {
            if i != j {
                let from = users[i].clone();
                let to = users[j].clone();

                let transfer_result = icrc1_transfer(TransferArgs {
                    from_subaccount: from.subaccount,
                    to: to.clone(),
                    amount: Nat::from(transfer_amount),
                });
                match transfer_result {
                    Ok(_) => {
                        log_event(
                            "DummyTransfer",
                            format!(
                                "From: {}, To: {}, Amount: {}",
                                from.owner, to.owner, transfer_amount
                            ),
                        );
                    }
                    Err(e) => {
                        log_event(
                            "DummyTransferFailed",
                            format!(
                                "From: {}, To: {}, Amount: {}, Error: {:?}",
                                from.owner, to.owner, transfer_amount, e
                            ),
                        );
                    }
                }
            }
        }
    }

    Ok(format!(
        "Added {} dummy users, performed approvals of {} tokens, transfer-from operations of {} tokens, and direct transfers of {} tokens each.",
        num_users, approve_amount, transfer_amount, transfer_amount
    ))
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
    METADATA.with(|m| {
        let meta = m.borrow().get(&0).unwrap();
        vec![
            ("icrc1:name".to_string(), meta.name.clone()),
            ("icrc1:symbol".to_string(), meta.symbol.clone()),
            ("icrc1:decimals".to_string(), meta.decimals.to_string()),
            (
                "icrc1:logo".to_string(),
                "https://your-logo-url.com/token.png".to_string(),
            ), // Update this URL
        ]
    })
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
    Nat::from(METADATA.with(|m| m.borrow().get(&0).unwrap().community_pool))
}

#[query]
fn get_team_pool_balance() -> Nat {
    Nat::from(METADATA.with(|m| m.borrow().get(&0).unwrap().team_vesting_pool))
}

#[query]
fn get_reserve_pool_balance() -> Nat {
    Nat::from(METADATA.with(|m| m.borrow().get(&0).unwrap().reserve))
}

#[query]
fn get_total_burned() -> Nat {
    Nat::from(METADATA.with(|m| m.borrow().get(&0).unwrap().total_burned))
}

#[query]
fn get_dapp_funds() -> Nat {
    Nat::from(DAPP_FUNDS.with(|df| df.borrow().get(&0).unwrap_or(0)))
}

#[query]
fn get_fee_distribution() -> (Nat, Nat, Nat) {
    // (burn, pool, dapp)
    METADATA.with(|m| {
        let meta = m.borrow().get(&0).unwrap();
        let fee = meta.transfer_fee;
        (
            Nat::from(fee * 20 / 100),
            Nat::from(fee * 10 / 100),
            Nat::from(fee * 70 / 100),
        )
    })
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
// Centralized Fee Processing & Logging
// -------------------------

fn process_fee(fee: u128) -> Result<(), LedgerError> {
    METADATA.with(|metadata| {
        let mut m = metadata.borrow_mut().get(&0).unwrap().clone();
        let burn_amount = fee * 20 / 100;
        let pool_amount = fee * 10 / 100;
        let dapp_amount = fee * 70 / 100;

        m.total_supply = m
            .total_supply
            .checked_sub(burn_amount)
            .ok_or(LedgerError::ArithmeticError)?;
        m.total_burned = m
            .total_burned
            .checked_add(burn_amount)
            .ok_or(LedgerError::ArithmeticError)?;
        m.community_pool = m
            .community_pool
            .checked_add(pool_amount)
            .ok_or(LedgerError::ArithmeticError)?;
        metadata.borrow_mut().insert(0, m);

        DAPP_FUNDS.with(|df| {
            let current_funds = df.borrow().get(&0).unwrap_or(0);
            let new_funds = current_funds
                .checked_add(dapp_amount)
                .ok_or(LedgerError::ArithmeticError)?;
            df.borrow_mut().insert(0, new_funds);
            Ok(())
        })?;

        log_event(
            "FeeProcessed",
            format!(
                "Fee: {}, Burned: {}, Pool: {}, DApp: {}",
                fee, burn_amount, pool_amount, dapp_amount
            ),
        );
        Ok(())
    })
}

fn log_event(event_type: &str, details: String) {
    LOGS.with(|logs| {
        let mut logs = logs.borrow_mut();
        let timestamp = current_time();
        logs.insert(
            timestamp,
            LogEntry {
                timestamp,
                event_type: event_type.to_string(),
                details,
            },
        );
        if logs.len() > 10_000 {
            if let Some((first_key, _)) = logs.first_key_value() {
                logs.remove(&first_key);
            }
        }
    });
}

// Export Candid interface
ic_cdk::export_candid!();
