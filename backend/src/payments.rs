//! Paying for chronolocks with ICP or $CHRONO (ICRC-2), and the treasury.
//!
//! Flow (same as ICPBonsai):
//! 1. The browser calls `icrc2_approve` on the chosen ledger for price + one
//!    ledger fee, naming this canister as spender.
//! 2. `create_chronolock` validates everything that can fail, then pulls the
//!    price with `icrc2_transfer_from` into this canister's default account
//!    (the treasury), and only then stores the lock.
//!
//! Each pull uses a fixed memo and `created_at_time`, so a retry after an
//! unknown outcome is deduplicated by the ledger and never charges twice.

use crate::state::{config, next_tx_id, update_config};
use crate::types::{ChronoError, ChronoResult};
use candid::{CandidType, Nat, Principal};
use ic_cdk::call::{Call, CallErrorExt};
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::{Memo, TransferArg, TransferError};
use icrc_ledger_types::icrc2::transfer_from::{TransferFromArgs, TransferFromError};
use serde::Deserialize;
use serde_bytes::ByteBuf;

const MAINNET_ICP_LEDGER: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";
const ICP_LEDGER_ENV: &str = "ICP_LEDGER_ID";
const MAX_ATTEMPTS: usize = 3;
const MEMO_PREFIX: &[u8] = b"chronolock:";

#[derive(CandidType, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaymentMethod {
    ICP,
    CHRONO,
}

/// Prices in the ledger's smallest unit (e8s for ICP and Soon tokens).
#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub struct Prices {
    pub icp_text: u64,
    pub icp_media: u64,
    pub chrono_text: u64,
    pub chrono_media: u64,
}

impl Default for Prices {
    fn default() -> Self {
        const E8: u64 = 100_000_000;
        Self {
            icp_text: E8 / 5,           // 0.2 ICP
            icp_media: E8,              // 1 ICP
            chrono_text: 1_000 * E8,    // 1,000 $CHRONO
            chrono_media: 5_000 * E8,   // 5,000 $CHRONO
        }
    }
}

/// Running totals of what the treasury collected (smallest units).
#[derive(CandidType, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Collected {
    pub icp: Nat,
    pub chrono: Nat,
    pub payments: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub struct PaymentReceipt {
    pub method: PaymentMethod,
    pub ledger: Principal,
    pub amount: Nat,
    pub block_index: Nat,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct PaymentInfo {
    pub prices: Prices,
    pub icp_ledger: Principal,
    /// `None` until an admin connects the $CHRONO token.
    pub chrono_ledger: Option<Principal>,
    /// Where payments go; also the spender to approve.
    pub treasury: Account,
    pub collected: Collected,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum PaymentError {
    /// $CHRONO payments are not enabled yet (token not connected).
    NotEnabled,
    InsufficientFunds { balance: Nat },
    /// Approve at least price + fee for this canister first.
    InsufficientAllowance { allowance: Nat },
    /// The ledger could not be reached; nothing was charged.
    LedgerUnavailable(String),
    Rejected(String),
}

pub fn icp_ledger() -> Principal {
    if ic_cdk::api::env_var_name_exists(ICP_LEDGER_ENV) {
        if let Ok(p) = Principal::from_text(ic_cdk::api::env_var_value(ICP_LEDGER_ENV)) {
            return p;
        }
    }
    Principal::from_text(MAINNET_ICP_LEDGER).expect("valid ICP ledger id")
}

pub fn ledger_for(method: PaymentMethod) -> ChronoResult<Principal> {
    match method {
        PaymentMethod::ICP => Ok(icp_ledger()),
        PaymentMethod::CHRONO => config()
            .token
            .map(|t| t.ledger)
            .ok_or(ChronoError::Payment(PaymentError::NotEnabled)),
    }
}

pub fn treasury() -> Account {
    Account {
        owner: ic_cdk::api::canister_self(),
        subaccount: None,
    }
}

pub fn price(method: PaymentMethod, with_media: bool) -> u64 {
    let p = config().prices;
    match (method, with_media) {
        (PaymentMethod::ICP, false) => p.icp_text,
        (PaymentMethod::ICP, true) => p.icp_media,
        (PaymentMethod::CHRONO, false) => p.chrono_text,
        (PaymentMethod::CHRONO, true) => p.chrono_media,
    }
}

pub fn info() -> PaymentInfo {
    let c = config();
    PaymentInfo {
        prices: c.prices,
        icp_ledger: icp_ledger(),
        chrono_ledger: c.token.map(|t| t.ledger),
        treasury: treasury(),
        collected: c.collected,
    }
}

pub fn set_prices(p: Prices) -> ChronoResult<()> {
    if [p.icp_text, p.icp_media, p.chrono_text, p.chrono_media].contains(&0) {
        return Err(ChronoError::InvalidInput("prices must be greater than zero".into()));
    }
    update_config(|c| c.prices = p);
    Ok(())
}

fn memo() -> Memo {
    let mut m = MEMO_PREFIX.to_vec();
    m.extend_from_slice(&next_tx_id().to_be_bytes());
    Memo(ByteBuf::from(m))
}

fn record(method: PaymentMethod, amount: &Nat) {
    update_config(|c| {
        match method {
            PaymentMethod::ICP => c.collected.icp += amount.clone(),
            PaymentMethod::CHRONO => c.collected.chrono += amount.clone(),
        }
        c.collected.payments += 1;
    });
}

/// Pulls the price from `payer` into the treasury. Callers must have done all
/// validation that can fail before calling this, and must hold a CallerGuard.
pub async fn collect(
    payer: Principal,
    method: PaymentMethod,
    with_media: bool,
) -> ChronoResult<PaymentReceipt> {
    let ledger = ledger_for(method)?;
    let amount = Nat::from(price(method, with_media));
    let args = TransferFromArgs {
        spender_subaccount: None,
        from: Account { owner: payer, subaccount: None },
        to: treasury(),
        amount: amount.clone(),
        fee: None,
        memo: Some(memo()),
        created_at_time: Some(ic_cdk::api::time()),
    };

    let mut last_err = String::new();
    for _ in 0..MAX_ATTEMPTS {
        let res = Call::bounded_wait(ledger, "icrc2_transfer_from")
            .with_arg(&args)
            .await;
        let reply = match res {
            Ok(r) => r,
            Err(e) => {
                // A clean reject means nothing happened; otherwise the outcome
                // is unknown and the identical retry is deduplicated.
                last_err = e.to_string();
                if e.is_clean_reject() && !e.is_immediately_retryable() {
                    break;
                }
                continue;
            }
        };
        let result: Result<Nat, TransferFromError> = reply
            .candid()
            .map_err(|e| ChronoError::Internal(format!("bad ledger reply: {e}")))?;
        let block_index = match result {
            Ok(block) => block,
            Err(TransferFromError::Duplicate { duplicate_of }) => duplicate_of,
            Err(TransferFromError::TemporarilyUnavailable) => {
                last_err = "ledger temporarily unavailable".into();
                continue;
            }
            Err(TransferFromError::InsufficientFunds { balance }) => {
                return Err(ChronoError::Payment(PaymentError::InsufficientFunds { balance }))
            }
            Err(TransferFromError::InsufficientAllowance { allowance }) => {
                return Err(ChronoError::Payment(PaymentError::InsufficientAllowance {
                    allowance,
                }))
            }
            Err(e) => return Err(ChronoError::Payment(PaymentError::Rejected(e.to_string()))),
        };
        record(method, &amount);
        ic_cdk::println!("{method:?} payment of {amount} from {payer} in block {block_index}");
        return Ok(PaymentReceipt { method, ledger, amount, block_index });
    }
    Err(ChronoError::Payment(PaymentError::LedgerUnavailable(last_err)))
}

/// Sends treasury funds out. The ledger fee is paid from the treasury on top.
pub async fn withdraw(method: PaymentMethod, to: Account, amount: Nat) -> ChronoResult<Nat> {
    let ledger = ledger_for(method)?;
    let args = TransferArg {
        from_subaccount: None,
        to,
        fee: None,
        created_at_time: Some(ic_cdk::api::time()),
        memo: Some(memo()),
        amount,
    };
    let mut last_err = String::new();
    for _ in 0..MAX_ATTEMPTS {
        let reply = match Call::bounded_wait(ledger, "icrc1_transfer").with_arg(&args).await {
            Ok(r) => r,
            Err(e) => {
                last_err = e.to_string();
                if e.is_clean_reject() && !e.is_immediately_retryable() {
                    break;
                }
                continue;
            }
        };
        let result: Result<Nat, TransferError> = reply
            .candid()
            .map_err(|e| ChronoError::Internal(format!("bad ledger reply: {e}")))?;
        return match result {
            Ok(block) => Ok(block),
            Err(TransferError::Duplicate { duplicate_of }) => Ok(duplicate_of),
            Err(TransferError::TemporarilyUnavailable) => {
                last_err = "ledger temporarily unavailable".into();
                continue;
            }
            Err(TransferError::InsufficientFunds { balance }) => {
                Err(ChronoError::Payment(PaymentError::InsufficientFunds { balance }))
            }
            Err(e) => Err(ChronoError::Payment(PaymentError::Rejected(e.to_string()))),
        };
    }
    Err(ChronoError::Payment(PaymentError::LedgerUnavailable(last_err)))
}
