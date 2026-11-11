//! ICRC-7 NFT interface over chronolocks. Each lock is one token, owned by a
//! principal's default subaccount.

use crate::locks;
use crate::state::{next_tx_id, LOCKS, OWNER_INDEX};
use crate::types::Chronolock;
use candid::{CandidType, Nat, Principal};
use icrc_ledger_types::icrc::generic_value::ICRC3Value as Value;
use icrc_ledger_types::icrc1::account::Account;
use serde::Deserialize;

pub const NAME: &str = "Chronolock";
pub const SYMBOL: &str = "CHRONOLOCK";
pub const DESCRIPTION: &str = "Time-locked, encrypted messages and media on the Internet Computer";
pub const MAX_QUERY_BATCH: u64 = 100;
pub const MAX_UPDATE_BATCH: u64 = 20;
pub const DEFAULT_TAKE: u64 = 20;
pub const MAX_TAKE: u64 = 100;
pub const MAX_MEMO: u64 = 32;
const TX_WINDOW_NS: u64 = 24 * 60 * 60 * 1_000_000_000;
const PERMITTED_DRIFT_NS: u64 = 2 * 60 * 1_000_000_000;

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TransferArg {
    pub from_subaccount: Option<serde_bytes::ByteBuf>,
    pub to: Account,
    pub token_id: Nat,
    pub memo: Option<serde_bytes::ByteBuf>,
    pub created_at_time: Option<u64>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum TransferError {
    NonExistingTokenId,
    InvalidRecipient,
    Unauthorized,
    TooOld,
    CreatedInFuture { ledger_time: u64 },
    Duplicate { duplicate_of: Nat },
    GenericError { error_code: Nat, message: String },
    GenericBatchError { error_code: Nat, message: String },
}

pub type TransferResult = Result<Nat, TransferError>;

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct StandardRecord {
    pub name: String,
    pub url: String,
}

pub fn supported_standards() -> Vec<StandardRecord> {
    vec![
        StandardRecord {
            name: "ICRC-7".to_string(),
            url: "https://github.com/dfinity/ICRC/ICRCs/ICRC-7".to_string(),
        },
        StandardRecord {
            name: "ICRC-10".to_string(),
            url: "https://github.com/dfinity/ICRC/ICRCs/ICRC-10".to_string(),
        },
    ]
}

fn is_default(sub: &Option<[u8; 32]>) -> bool {
    sub.is_none_or(|s| s == [0u8; 32])
}

fn token_id(n: &Nat) -> Option<u64> {
    u64::try_from(n.0.clone()).ok()
}

pub fn collection_metadata() -> Vec<(String, Value)> {
    vec![
        ("icrc7:name".into(), Value::Text(NAME.into())),
        ("icrc7:symbol".into(), Value::Text(SYMBOL.into())),
        ("icrc7:description".into(), Value::Text(DESCRIPTION.into())),
        (
            "icrc7:total_supply".into(),
            Value::Nat(Nat::from(total_supply())),
        ),
    ]
}

pub fn total_supply() -> u64 {
    LOCKS.with(|l| l.borrow().len())
}

fn lock_metadata(l: &Chronolock) -> Vec<(String, Value)> {
    let mut m = vec![
        (
            "icrc7:name".to_string(),
            Value::Text(format!("Chronolock #{}", l.id)),
        ),
        (
            "chronolock:unlock_time".to_string(),
            Value::Nat(Nat::from(l.unlock_time)),
        ),
        (
            "chronolock:created_at".to_string(),
            Value::Nat(Nat::from(l.created_at)),
        ),
        (
            "chronolock:creator".to_string(),
            Value::Text(l.creator.to_text()),
        ),
        (
            "chronolock:recipients".to_string(),
            Value::Nat(Nat::from(l.user_keys.len() as u64)),
        ),
    ];
    if let Some(media) = &l.media {
        m.push((
            "chronolock:media_size".to_string(),
            Value::Nat(Nat::from(media.size)),
        ));
    }
    m
}

pub fn token_metadata(ids: Vec<Nat>) -> Vec<Option<Vec<(String, Value)>>> {
    ids.iter()
        .take(MAX_QUERY_BATCH as usize)
        .map(|n| token_id(n).and_then(locks::get).map(|l| lock_metadata(&l)))
        .collect()
}

pub fn owner_of(ids: Vec<Nat>) -> Vec<Option<Account>> {
    ids.iter()
        .take(MAX_QUERY_BATCH as usize)
        .map(|n| {
            token_id(n).and_then(locks::get).map(|l| Account {
                owner: l.owner,
                subaccount: None,
            })
        })
        .collect()
}

pub fn balance_of(accounts: Vec<Account>) -> Vec<Nat> {
    accounts
        .iter()
        .take(MAX_QUERY_BATCH as usize)
        .map(|a| {
            if is_default(&a.subaccount) {
                Nat::from(locks::count_owned(a.owner))
            } else {
                Nat::from(0u64)
            }
        })
        .collect()
}

fn take_of(take: Option<Nat>) -> usize {
    take.and_then(|t| token_id(&t))
        .unwrap_or(DEFAULT_TAKE)
        .min(MAX_TAKE) as usize
}

pub fn tokens(prev: Option<Nat>, take: Option<Nat>) -> Vec<Nat> {
    let start = prev.and_then(|p| token_id(&p)).map(|p| p + 1).unwrap_or(0);
    LOCKS.with(|l| {
        l.borrow()
            .range(start..)
            .take(take_of(take))
            .map(|e| Nat::from(*e.key()))
            .collect()
    })
}

pub fn tokens_of(account: Account, prev: Option<Nat>, take: Option<Nat>) -> Vec<Nat> {
    if !is_default(&account.subaccount) {
        return vec![];
    }
    let start = prev.and_then(|p| token_id(&p)).map(|p| p + 1).unwrap_or(0);
    OWNER_INDEX.with(|o| {
        o.borrow()
            .range((account.owner, start)..=(account.owner, u64::MAX))
            .take(take_of(take))
            .map(|e| Nat::from(e.key().1))
            .collect()
    })
}

pub fn transfer(caller: Principal, args: Vec<TransferArg>) -> Vec<Option<TransferResult>> {
    if args.len() as u64 > MAX_UPDATE_BATCH {
        return vec![Some(Err(TransferError::GenericBatchError {
            error_code: Nat::from(1u64),
            message: format!("at most {MAX_UPDATE_BATCH} transfers per call"),
        }))];
    }
    let now = ic_cdk::api::time();
    args.into_iter()
        .map(|a| Some(transfer_one(caller, a, now)))
        .collect()
}

fn transfer_one(caller: Principal, a: TransferArg, now: u64) -> TransferResult {
    if let Some(t) = a.created_at_time {
        if t + TX_WINDOW_NS + PERMITTED_DRIFT_NS < now {
            return Err(TransferError::TooOld);
        }
        if t > now + PERMITTED_DRIFT_NS {
            return Err(TransferError::CreatedInFuture { ledger_time: now });
        }
    }
    if a.memo.as_ref().is_some_and(|m| m.len() as u64 > MAX_MEMO) {
        return Err(TransferError::GenericError {
            error_code: Nat::from(2u64),
            message: "memo too long".into(),
        });
    }
    let from_sub = a.from_subaccount.as_ref().map(|b| b.as_slice());
    if from_sub.is_some_and(|s| s.iter().any(|b| *b != 0)) {
        return Err(TransferError::Unauthorized);
    }
    if !is_default(&a.to.subaccount) || a.to.owner == Principal::anonymous() || a.to.owner == caller
    {
        return Err(TransferError::InvalidRecipient);
    }
    let id = token_id(&a.token_id).ok_or(TransferError::NonExistingTokenId)?;
    match locks::transfer_owner(id, caller, a.to.owner) {
        Ok(()) => {
            ic_cdk::println!("lock {id} transferred {caller} -> {}", a.to.owner);
            Ok(Nat::from(next_tx_id()))
        }
        Err(crate::types::ChronoError::NotFound) => Err(TransferError::NonExistingTokenId),
        Err(_) => Err(TransferError::Unauthorized),
    }
}
