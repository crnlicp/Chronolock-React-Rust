//! Chronolock backend: time-locked, encrypted messages and media.

mod auth;
mod icrc7;
mod locks;
mod media;
mod payments;
mod state;
mod types;
mod vetkd;

use auth::{admin_caller, authenticated_caller, is_admin, CallerGuard};
use candid::{Nat, Principal};
use ic_cdk::{init, post_upgrade, query, update};
use icrc7::{StandardRecord, TransferArg, TransferResult};
use icrc_ledger_types::icrc::generic_value::ICRC3Value as Value;
use icrc_ledger_types::icrc1::account::Account;
use state::{config, update_config, ADMINS};
use std::time::Duration;
use types::*;

const CLEANUP_INTERVAL: Duration = Duration::from_secs(10 * 60);

// ---------------------------------------------------------------------------
// Lifecycle
// ---------------------------------------------------------------------------

#[init]
fn init(args: Option<InitArgs>) {
    if let Some(args) = args {
        if let Some(name) = args.vetkd_key_name {
            update_config(|c| c.vetkd_key_name = name);
        }
        add_admins(args.admins);
    }
    start_timers();
}

#[post_upgrade]
fn post_upgrade(args: Option<InitArgs>) {
    // The vetKD key name is fixed at install: changing it would make every
    // existing lock undecryptable. Only admins can be added on upgrade.
    if let Some(args) = args {
        add_admins(args.admins);
    }
    start_timers();
}

fn add_admins(admins: Option<Vec<Principal>>) {
    for p in admins.unwrap_or_default() {
        if p != Principal::anonymous() {
            ADMINS.with(|a| a.borrow_mut().insert(p, ()));
        }
    }
}

fn start_timers() {
    ic_cdk_timers::set_timer_interval(CLEANUP_INTERVAL, || async { media::cleanup_expired() });
}

// ---------------------------------------------------------------------------
// Chronolocks
// ---------------------------------------------------------------------------

/// Creates a chronolock, paid with ICP or $CHRONO (ICRC-2). Approve the
/// price + one ledger fee for this canister first (see `get_payment_info`).
#[update]
async fn create_chronolock(args: CreateChronolockArgs) -> ChronoResult<Chronolock> {
    let caller = authenticated_caller()?;
    let _guard = CallerGuard::new(caller)?;
    let method = args.payment;
    // Everything that can fail is checked before money moves.
    let validated = locks::validate(caller, args)?;
    payments::ledger_for(method)?;
    let receipt = payments::collect(caller, method, validated.media.is_some()).await?;
    Ok(locks::insert(caller, validated, Some(receipt)))
}

#[update]
fn burn_chronolock(id: u64) -> ChronoResult<()> {
    let caller = authenticated_caller()?;
    locks::burn(caller, id)
}

#[query]
fn get_chronolock(id: u64) -> Option<Chronolock> {
    locks::get(id)
}

#[query]
fn list_chronolocks(page: Page) -> LockPage {
    locks::list_all(page)
}

#[query]
fn list_owned_chronolocks(owner: Principal, page: Page) -> LockPage {
    locks::list_owned(owner, page)
}

/// Locks addressed to `user` plus public locks.
#[query]
fn list_received_chronolocks(user: Principal, unlocked_only: bool, page: Page) -> LockPage {
    locks::list_for_recipient(user, unlocked_only, page)
}

#[query]
fn get_stats() -> Stats {
    locks::stats()
}

// ---------------------------------------------------------------------------
// Media
// ---------------------------------------------------------------------------

#[update]
fn start_media_upload(args: StartMediaUploadArgs) -> ChronoResult<u64> {
    media::start(args)
}

#[update]
fn upload_media_chunk(media_id: u64, index: u32, chunk: serde_bytes::ByteBuf) -> ChronoResult<()> {
    media::upload_chunk(media_id, index, chunk.into_vec())
}

#[update]
fn finish_media_upload(media_id: u64) -> ChronoResult<MediaInfo> {
    media::finish(media_id)
}

#[update]
fn cancel_media_upload(media_id: u64) -> ChronoResult<()> {
    media::cancel(media_id)
}

#[query]
fn get_media_info(media_id: u64) -> ChronoResult<MediaInfo> {
    media::info(media_id)
}

#[query]
fn get_media_chunk(media_id: u64, index: u32) -> ChronoResult<serde_bytes::ByteBuf> {
    media::get_chunk(media_id, index).map(serde_bytes::ByteBuf::from)
}

// ---------------------------------------------------------------------------
// vetKD time-lock keys
// ---------------------------------------------------------------------------

#[update]
async fn ibe_public_key() -> ChronoResult<serde_bytes::ByteBuf> {
    vetkd::public_key().await.map(serde_bytes::ByteBuf::from)
}

#[update]
async fn public_time_key(unlock_time: u64, transport_public_key: serde_bytes::ByteBuf) -> ChronoResult<serde_bytes::ByteBuf> {
    vetkd::public_time_key(unlock_time, transport_public_key.into_vec())
        .await
        .map(serde_bytes::ByteBuf::from)
}

#[update]
async fn user_time_key(unlock_time: u64, transport_public_key: serde_bytes::ByteBuf) -> ChronoResult<serde_bytes::ByteBuf> {
    vetkd::user_time_key(unlock_time, transport_public_key.into_vec())
        .await
        .map(serde_bytes::ByteBuf::from)
}

// ---------------------------------------------------------------------------
// Config & admin
// ---------------------------------------------------------------------------

#[query]
fn get_config() -> PublicConfig {
    let c = config();
    PublicConfig {
        vetkd_key_name: c.vetkd_key_name,
        scheme: vetkd::SCHEME_VERSION,
        limits: c.limits,
        token: c.token,
    }
}

#[query]
fn get_payment_info() -> payments::PaymentInfo {
    payments::info()
}

/// Sets prices in smallest units (e8s). Admin only.
#[update]
fn set_prices(prices: payments::Prices) -> ChronoResult<()> {
    let admin = admin_caller()?;
    payments::set_prices(prices.clone())?;
    ic_cdk::println!("prices set to {prices:?} by {admin}");
    Ok(())
}

/// Sends collected funds from the treasury. The ledger fee is paid on top.
#[update]
async fn withdraw(method: payments::PaymentMethod, to: Account, amount: Nat) -> ChronoResult<Nat> {
    let admin = admin_caller()?;
    let _guard = CallerGuard::new(admin)?;
    let block = payments::withdraw(method, to, amount.clone()).await?;
    ic_cdk::println!("withdrew {amount} {method:?} to {to} by {admin} (block {block})");
    Ok(block)
}

/// The $CHRONO token launched on Soon, if configured.
#[query]
fn get_token() -> Option<TokenConfig> {
    config().token
}

/// Connects (or with `None`, disconnects) the $CHRONO token. Admin only.
#[update]
fn set_token(token: Option<TokenConfig>) -> ChronoResult<()> {
    let admin = admin_caller()?;
    if let Some(t) = &token {
        if t.ledger == Principal::anonymous() || t.symbol.trim().is_empty() || t.symbol.len() > 16 {
            return Err(ChronoError::InvalidInput("invalid token config".to_string()));
        }
    }
    update_config(|c| c.token = token.clone());
    ic_cdk::println!("token set to {token:?} by {admin}");
    Ok(())
}

#[query]
fn whoami() -> (Principal, bool) {
    let caller = ic_cdk::api::msg_caller();
    (caller, is_admin(&caller))
}

#[update]
fn set_limits(limits: Limits) -> ChronoResult<()> {
    let admin = admin_caller()?;
    if limits.max_chunk_bytes == 0 || limits.max_chunk_bytes > 1_950_000 {
        return Err(ChronoError::InvalidInput("max_chunk_bytes must be 1..=1950000".to_string()));
    }
    update_config(|c| c.limits = limits);
    ic_cdk::println!("limits updated by {admin}");
    Ok(())
}

#[update]
fn add_admin(p: Principal) -> ChronoResult<()> {
    admin_caller()?;
    if p == Principal::anonymous() {
        return Err(ChronoError::InvalidInput("anonymous cannot be admin".to_string()));
    }
    ADMINS.with(|a| a.borrow_mut().insert(p, ()));
    Ok(())
}

#[update]
fn remove_admin(p: Principal) -> ChronoResult<()> {
    admin_caller()?;
    ADMINS.with(|a| a.borrow_mut().remove(&p));
    Ok(())
}

#[query]
fn list_admins() -> ChronoResult<Vec<Principal>> {
    admin_caller()?;
    Ok(ADMINS.with(|a| a.borrow().iter().map(|e| *e.key()).collect()))
}

// ---------------------------------------------------------------------------
// ICRC-7 / ICRC-10
// ---------------------------------------------------------------------------

#[query]
fn icrc10_supported_standards() -> Vec<StandardRecord> {
    icrc7::supported_standards()
}

#[query]
fn icrc7_collection_metadata() -> Vec<(String, Value)> {
    icrc7::collection_metadata()
}

#[query]
fn icrc7_name() -> String {
    icrc7::NAME.to_string()
}

#[query]
fn icrc7_symbol() -> String {
    icrc7::SYMBOL.to_string()
}

#[query]
fn icrc7_description() -> Option<String> {
    Some(icrc7::DESCRIPTION.to_string())
}

#[query]
fn icrc7_logo() -> Option<String> {
    None
}

#[query]
fn icrc7_total_supply() -> Nat {
    Nat::from(icrc7::total_supply())
}

#[query]
fn icrc7_supply_cap() -> Option<Nat> {
    None
}

#[query]
fn icrc7_max_query_batch_size() -> Option<Nat> {
    Some(Nat::from(icrc7::MAX_QUERY_BATCH))
}

#[query]
fn icrc7_max_update_batch_size() -> Option<Nat> {
    Some(Nat::from(icrc7::MAX_UPDATE_BATCH))
}

#[query]
fn icrc7_default_take_value() -> Option<Nat> {
    Some(Nat::from(icrc7::DEFAULT_TAKE))
}

#[query]
fn icrc7_max_take_value() -> Option<Nat> {
    Some(Nat::from(icrc7::MAX_TAKE))
}

#[query]
fn icrc7_max_memo_size() -> Option<Nat> {
    Some(Nat::from(icrc7::MAX_MEMO))
}

#[query]
fn icrc7_atomic_batch_transfers() -> Option<bool> {
    Some(false)
}

#[query]
fn icrc7_token_metadata(ids: Vec<Nat>) -> Vec<Option<Vec<(String, Value)>>> {
    icrc7::token_metadata(ids)
}

#[query]
fn icrc7_owner_of(ids: Vec<Nat>) -> Vec<Option<Account>> {
    icrc7::owner_of(ids)
}

#[query]
fn icrc7_balance_of(accounts: Vec<Account>) -> Vec<Nat> {
    icrc7::balance_of(accounts)
}

#[query]
fn icrc7_tokens(prev: Option<Nat>, take: Option<Nat>) -> Vec<Nat> {
    icrc7::tokens(prev, take)
}

#[query]
fn icrc7_tokens_of(account: Account, prev: Option<Nat>, take: Option<Nat>) -> Vec<Nat> {
    icrc7::tokens_of(account, prev, take)
}

#[update]
fn icrc7_transfer(args: Vec<TransferArg>) -> Vec<Option<TransferResult>> {
    match authenticated_caller() {
        Ok(caller) => icrc7::transfer(caller, args),
        Err(_) => args.iter().map(|_| Some(Err(icrc7::TransferError::Unauthorized))).collect(),
    }
}

ic_cdk::export_candid!();
