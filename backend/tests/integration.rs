//! PocketIC integration tests: the backend with real ICRC-1/ICRC-2 ledgers
//! and vetKD. Run with `scripts/test-backend.sh` (builds the wasm, fetches the
//! ledger wasm and points POCKET_IC_BIN at icp-cli's PocketIC server).

use backend::icrc7::{TransferArg, TransferError};
use backend::payments::{PaymentError, PaymentInfo, PaymentMethod, Prices};
use backend::types::*;
use candid::utils::ArgumentEncoder;
use candid::{decode_one, encode_args, CandidType, Nat, Principal};
use icrc_ledger_types::icrc::generic_metadata_value::MetadataValue;
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::{TransferArg as LedgerTransferArg, TransferError as LedgerTransferError};
use icrc_ledger_types::icrc2::approve::{ApproveArgs, ApproveError};
use pocket_ic::{PocketIc, PocketIcBuilder};
use serde::Deserialize;
use serde_bytes::ByteBuf;
use std::time::Duration;

const ICP_LEDGER: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";
const E8: u64 = 100_000_000;
const FEE: u64 = 10_000;
const SCHEME: u8 = 2;
/// A valid 48-byte transport public key (the BLS12-381 G1 generator).
const TRANSPORT_KEY: [u8; 48] = [
    0x97, 0xf1, 0xd3, 0xa7, 0x31, 0x97, 0xd7, 0x94, 0x26, 0x95, 0x63, 0x8c, 0x4f, 0xa9, 0xac, 0x0f,
    0xc3, 0x68, 0x8c, 0x4f, 0x97, 0x74, 0xb9, 0x05, 0xa1, 0x4e, 0x3a, 0x3f, 0x17, 0x1b, 0xac, 0x58,
    0x6c, 0x55, 0xe8, 0x3f, 0xf9, 0x7a, 0x1a, 0xef, 0xfb, 0x3a, 0xf0, 0x0a, 0xdb, 0x22, 0xc6, 0xbb,
];

// ---------------------------------------------------------------------------
// Ledger init types (the ICRC-1 reference ledger)
// ---------------------------------------------------------------------------

#[derive(CandidType)]
struct FeatureFlags {
    icrc2: bool,
}

#[derive(CandidType)]
struct ArchiveOptions {
    num_blocks_to_archive: u64,
    trigger_threshold: u64,
    controller_id: Principal,
}

#[derive(CandidType)]
struct LedgerInit {
    minting_account: Account,
    transfer_fee: Nat,
    decimals: Option<u8>,
    token_symbol: String,
    token_name: String,
    metadata: Vec<(String, MetadataValue)>,
    initial_balances: Vec<(Account, Nat)>,
    feature_flags: Option<FeatureFlags>,
    archive_options: ArchiveOptions,
}

#[derive(CandidType)]
enum LedgerArg {
    Init(LedgerInit),
}

// ---------------------------------------------------------------------------
// Test environment
// ---------------------------------------------------------------------------

struct Env {
    pic: PocketIc,
    backend: Principal,
    icp: Principal,
    chrono: Principal,
    admin: Principal,
    alice: Principal,
    bob: Principal,
    carol: Principal,
}

fn user(n: u8) -> Principal {
    Principal::self_authenticating([n; 32])
}

fn account(owner: Principal) -> Account {
    Account { owner, subaccount: None }
}

fn wasm(env_var: &str, default: &str) -> Vec<u8> {
    let path = std::env::var(env_var).unwrap_or_else(|_| default.to_string());
    std::fs::read(&path).unwrap_or_else(|e| {
        panic!("cannot read {path} ({e}). Run scripts/test-backend.sh, which builds and fetches it.")
    })
}

fn backend_wasm() -> Vec<u8> {
    wasm("BACKEND_WASM", "../target/wasm32-unknown-unknown/release/backend.wasm")
}

fn install_ledger(pic: &PocketIc, id: Principal, symbol: &str, funded: &[Principal]) {
    pic.add_cycles(id, 100_000_000_000_000);
    let arg = LedgerArg::Init(LedgerInit {
        minting_account: account(user(250)),
        transfer_fee: Nat::from(FEE),
        decimals: Some(8),
        token_symbol: symbol.to_string(),
        token_name: symbol.to_string(),
        metadata: vec![],
        initial_balances: funded
            .iter()
            .map(|p| (account(*p), Nat::from(100_000 * E8)))
            .collect(),
        feature_flags: Some(FeatureFlags { icrc2: true }),
        archive_options: ArchiveOptions {
            num_blocks_to_archive: 1000,
            trigger_threshold: 2000,
            controller_id: user(250),
        },
    });
    pic.install_canister(
        id,
        wasm("LEDGER_WASM", "../.icp/cache/local-token/ic-icrc1-ledger.wasm.gz"),
        candid::encode_one(arg).unwrap(),
        None,
    );
}

fn setup() -> Env {
    let pic = PocketIcBuilder::new()
        .with_nns_subnet()
        .with_ii_subnet()
        .with_fiduciary_subnet() // threshold keys (vetKD) live on the II and fiduciary subnets
        .with_application_subnet()
        .build();
    let (admin, alice, bob, carol) = (user(1), user(2), user(3), user(4));

    // ICP ledger at its mainnet id (the backend's default), $CHRONO anywhere.
    let icp = pic
        .create_canister_with_id(None, None, Principal::from_text(ICP_LEDGER).unwrap())
        .expect("create ICP ledger");
    install_ledger(&pic, icp, "ICP", &[alice, bob]);
    let chrono = pic.create_canister();
    install_ledger(&pic, chrono, "CHRONO", &[alice, bob]);

    let backend = pic.create_canister_with_settings(Some(admin), None);
    pic.add_cycles(backend, 1_000_000_000_000_000);
    let init = Some(InitArgs { vetkd_key_name: Some("test_key_1".to_string()), admins: None });
    pic.install_canister(backend, backend_wasm(), candid::encode_one(init).unwrap(), Some(admin));

    Env { pic, backend, icp, chrono, admin, alice, bob, carol }
}

impl Env {
    fn update<A: ArgumentEncoder, T: CandidType + for<'de> Deserialize<'de>>(
        &self,
        canister: Principal,
        sender: Principal,
        method: &str,
        args: A,
    ) -> T {
        let bytes = self
            .pic
            .update_call(canister, sender, method, encode_args(args).unwrap())
            .unwrap_or_else(|e| panic!("{method} rejected: {e}"));
        decode_one(&bytes).unwrap_or_else(|e| panic!("{method} decode: {e}"))
    }

    fn query<A: ArgumentEncoder, T: CandidType + for<'de> Deserialize<'de>>(
        &self,
        canister: Principal,
        sender: Principal,
        method: &str,
        args: A,
    ) -> T {
        let bytes = self
            .pic
            .query_call(canister, sender, method, encode_args(args).unwrap())
            .unwrap_or_else(|e| panic!("{method} rejected: {e}"));
        decode_one(&bytes).unwrap_or_else(|e| panic!("{method} decode: {e}"))
    }

    fn now_s(&self) -> u64 {
        self.pic.get_time().as_nanos_since_unix_epoch() / 1_000_000_000
    }

    fn balance(&self, ledger: Principal, owner: Principal) -> u64 {
        let n: Nat = self.query(ledger, Principal::anonymous(), "icrc1_balance_of", (account(owner),));
        u64::try_from(n.0).unwrap()
    }

    fn approve(&self, ledger: Principal, from: Principal, amount: u64) {
        let args = ApproveArgs {
            from_subaccount: None,
            spender: account(self.backend),
            amount: Nat::from(amount),
            expected_allowance: None,
            expires_at: None,
            fee: None,
            memo: None,
            created_at_time: None,
        };
        let res: Result<Nat, ApproveError> = self.update(ledger, from, "icrc2_approve", (args,));
        res.expect("approve");
    }

    fn info(&self) -> PaymentInfo {
        self.query(self.backend, Principal::anonymous(), "get_payment_info", ())
    }

    fn stats(&self) -> Stats {
        self.query(self.backend, Principal::anonymous(), "get_stats", ())
    }

    fn lock_args(&self, recipients: &[Principal], payment: PaymentMethod) -> CreateChronolockArgs {
        let user_keys = if recipients.is_empty() {
            vec![UserKey { user: "public".to_string(), key: "wrapped".to_string() }]
        } else {
            recipients
                .iter()
                .map(|p| UserKey { user: p.to_text(), key: "wrapped".to_string() })
                .collect()
        };
        CreateChronolockArgs {
            scheme: SCHEME,
            unlock_time: self.now_s() + 3600,
            user_keys,
            encrypted_metadata: "ciphertext".to_string(),
            media_id: None,
            payment,
        }
    }

    fn create(&self, sender: Principal, args: CreateChronolockArgs) -> ChronoResult<Chronolock> {
        self.update(self.backend, sender, "create_chronolock", (args,))
    }

    /// Approves the ICP text price and creates a lock for `recipients`.
    fn paid_lock(&self, sender: Principal, recipients: &[Principal]) -> Chronolock {
        self.approve(self.icp, sender, E8 / 5 + FEE);
        self.create(sender, self.lock_args(recipients, PaymentMethod::ICP)).expect("create")
    }

    fn connect_chrono(&self) {
        let token = Some(TokenConfig { ledger: self.chrono, market: None, symbol: "CHRONO".into() });
        let res: ChronoResult<()> = self.update(self.backend, self.admin, "set_token", (token,));
        res.expect("set_token");
    }

    fn upload(&self, owner: Principal, chunks: &[&[u8]]) -> u64 {
        let size: usize = chunks.iter().map(|c| c.len()).sum();
        let args = StartMediaUploadArgs { size: size as u64, chunk_count: chunks.len() as u32 };
        let id: ChronoResult<u64> = self.update(self.backend, owner, "start_media_upload", (args,));
        let id = id.expect("start upload");
        for (i, c) in chunks.iter().enumerate() {
            let r: ChronoResult<()> = self.update(
                self.backend,
                owner,
                "upload_media_chunk",
                (id, i as u32, ByteBuf::from(c.to_vec())),
            );
            r.expect("upload chunk");
        }
        let r: ChronoResult<MediaInfo> = self.update(self.backend, owner, "finish_media_upload", (id,));
        r.expect("finish upload");
        id
    }

    fn page() -> Page {
        Page { offset: 0, limit: 50 }
    }
}

// ---------------------------------------------------------------------------
// Payments
// ---------------------------------------------------------------------------

#[test]
fn anonymous_cannot_create() {
    let env = setup();
    let res = env.create(Principal::anonymous(), env.lock_args(&[], PaymentMethod::ICP));
    assert_eq!(res.unwrap_err(), ChronoError::AnonymousCaller);
}

#[test]
fn create_without_approval_charges_nothing_and_stores_nothing() {
    let env = setup();
    let before = env.balance(env.icp, env.alice);
    let res = env.create(env.alice, env.lock_args(&[], PaymentMethod::ICP));
    assert!(matches!(
        res.unwrap_err(),
        ChronoError::Payment(PaymentError::InsufficientAllowance { .. })
    ));
    assert_eq!(env.balance(env.icp, env.alice), before);
    assert_eq!(env.stats().total_locks, 0);
    assert_eq!(env.info().collected.payments, 0);
}

#[test]
fn icp_text_lock_costs_exactly_the_price_plus_fees() {
    let env = setup();
    let price = E8 / 5; // 0.2 ICP
    assert_eq!(env.info().prices, Prices::default());
    let before = env.balance(env.icp, env.alice);

    let lock = env.paid_lock(env.alice, &[env.bob]);

    assert_eq!(env.balance(env.icp, env.backend), price, "treasury receives the price");
    assert_eq!(env.balance(env.icp, env.alice), before - price - 2 * FEE, "approve fee + transfer fee");
    let receipt = lock.payment.expect("receipt");
    assert_eq!(receipt.method, PaymentMethod::ICP);
    assert_eq!(receipt.amount, Nat::from(price));
    assert_eq!(receipt.ledger, env.icp);
    let info = env.info();
    assert_eq!(info.collected.icp, Nat::from(price));
    assert_eq!(info.collected.payments, 1);
    assert_eq!(info.treasury.owner, env.backend);
    assert_eq!(lock.owner, env.alice);
    assert_eq!(lock.scheme, SCHEME);
}

#[test]
fn chrono_payments_need_the_token_connected_by_an_admin() {
    let env = setup();
    let media_price = 5_000 * E8;
    let res = env.create(env.alice, env.lock_args(&[], PaymentMethod::CHRONO));
    assert_eq!(res.unwrap_err(), ChronoError::Payment(PaymentError::NotEnabled));

    let token = Some(TokenConfig { ledger: env.chrono, market: None, symbol: "CHRONO".into() });
    let denied: ChronoResult<()> = env.update(env.backend, env.alice, "set_token", (token,));
    assert_eq!(denied.unwrap_err(), ChronoError::Unauthorized);
    env.connect_chrono();
    assert_eq!(env.info().chrono_ledger, Some(env.chrono));

    // Media lock paid with $CHRONO.
    let media_id = env.upload(env.alice, &[b"chunk-one", b"chunk-two"]);
    env.approve(env.chrono, env.alice, media_price + FEE);
    let mut args = env.lock_args(&[], PaymentMethod::CHRONO);
    args.media_id = Some(media_id);
    let lock = env.create(env.alice, args).expect("create");

    assert_eq!(env.balance(env.chrono, env.backend), media_price);
    assert_eq!(lock.media.as_ref().unwrap().chunk_count, 2);
    assert_eq!(lock.payment.unwrap().method, PaymentMethod::CHRONO);
    assert_eq!(env.info().collected.chrono, Nat::from(media_price));
    assert_eq!(env.stats().media_locks, 1);
    assert_eq!(env.balance(env.icp, env.backend), 0, "ICP untouched");
}

#[test]
fn insufficient_funds_fail_cleanly() {
    let env = setup();
    // Carol has no ICP at all; give her just enough to pay the approve fee.
    let arg = LedgerTransferArg {
        from_subaccount: None,
        to: account(env.carol),
        fee: None,
        created_at_time: None,
        memo: None,
        amount: Nat::from(2 * FEE),
    };
    let sent: Result<Nat, LedgerTransferError> = env.update(env.icp, env.alice, "icrc1_transfer", (arg,));
    sent.unwrap();
    env.approve(env.icp, env.carol, E8);

    let res = env.create(env.carol, env.lock_args(&[], PaymentMethod::ICP));
    assert!(matches!(
        res.unwrap_err(),
        ChronoError::Payment(PaymentError::InsufficientFunds { .. })
    ));
    assert_eq!(env.stats().total_locks, 0);
    assert_eq!(env.balance(env.icp, env.backend), 0);
}

#[test]
fn invalid_requests_are_rejected_before_any_payment() {
    let env = setup();
    env.approve(env.icp, env.alice, E8);
    let after_approve = env.balance(env.icp, env.alice);
    let cases: Vec<(&str, Box<dyn Fn(&mut CreateChronolockArgs)>)> = vec![
        ("past unlock time", Box::new(|a| a.unlock_time = 1)),
        ("wrong scheme", Box::new(|a| a.scheme = 1)),
        ("no keys", Box::new(|a| a.user_keys.clear())),
        ("empty metadata", Box::new(|a| a.encrypted_metadata.clear())),
        ("bad recipient", Box::new(|a| a.user_keys[0].user = "not-a-principal".into())),
        ("unknown media", Box::new(|a| a.media_id = Some(999))),
        ("oversized metadata", Box::new(|a| a.encrypted_metadata = "x".repeat(70_000))),
    ];
    for (name, mutate) in cases {
        let mut args = env.lock_args(&[env.bob], PaymentMethod::ICP);
        mutate(&mut args);
        assert!(env.create(env.alice, args).is_err(), "{name} should be rejected");
        assert_eq!(env.balance(env.icp, env.alice), after_approve, "{name} must not charge");
    }
    assert_eq!(env.stats().total_locks, 0);
}

#[test]
fn a_second_create_while_one_is_in_flight_is_refused() {
    let env = setup();
    env.approve(env.icp, env.alice, 2 * (E8 / 5 + FEE));
    let args = || encode_args((env.lock_args(&[], PaymentMethod::ICP),)).unwrap();
    let first = env.pic.submit_call(env.backend, env.alice, "create_chronolock", args()).unwrap();
    let second = env.pic.submit_call(env.backend, env.alice, "create_chronolock", args()).unwrap();
    let results: Vec<ChronoResult<Chronolock>> = [first, second]
        .into_iter()
        .map(|id| decode_one(&env.pic.await_call(id).unwrap()).unwrap())
        .collect();

    let ok = results.iter().filter(|r| r.is_ok()).count();
    let busy = results
        .iter()
        .filter(|r| matches!(r, Err(ChronoError::AlreadyProcessing)))
        .count();
    assert_eq!((ok, busy), (1, 1), "exactly one succeeds, the other is refused: {results:?}");
    assert_eq!(env.balance(env.icp, env.backend), E8 / 5, "charged once");
    // The guard is released afterwards.
    assert!(env.create(env.alice, env.lock_args(&[], PaymentMethod::ICP)).is_ok());
}

// ---------------------------------------------------------------------------
// Time-lock keys (vetKD)
// ---------------------------------------------------------------------------

#[test]
fn keys_are_released_only_after_the_unlock_time_and_only_to_the_caller() {
    let env = setup();
    let lock = env.paid_lock(env.alice, &[env.bob]);
    let tpk = || ByteBuf::from(TRANSPORT_KEY.to_vec());
    let key = |who: Principal, method: &str| -> ChronoResult<ByteBuf> {
        env.update(env.backend, who, method, (lock.unlock_time, tpk()))
    };

    assert_eq!(key(env.bob, "user_time_key").unwrap_err(), ChronoError::TimeLocked);
    assert_eq!(key(env.bob, "public_time_key").unwrap_err(), ChronoError::TimeLocked);

    env.pic.advance_time(Duration::from_secs(3601));
    env.pic.tick();

    assert_eq!(
        key(Principal::anonymous(), "user_time_key").unwrap_err(),
        ChronoError::AnonymousCaller
    );
    let bob_key = key(env.bob, "user_time_key").expect("bob's key");
    let alice_key = key(env.alice, "user_time_key").expect("alice's key");
    let public_key = key(env.bob, "public_time_key").expect("public key");
    assert!(!bob_key.is_empty());
    // Each identity gets a different key: the caller is bound into the derivation.
    assert_ne!(bob_key, alice_key);
    assert_ne!(bob_key, public_key);

    let short: ChronoResult<ByteBuf> =
        env.update(env.backend, env.bob, "user_time_key", (lock.unlock_time, ByteBuf::from(vec![1u8; 10])));
    assert!(matches!(short.unwrap_err(), ChronoError::InvalidInput(_)));

    let pk: ChronoResult<ByteBuf> = env.update(env.backend, env.bob, "ibe_public_key", ());
    assert_eq!(pk.expect("ibe public key").len(), 96);
}

#[test]
fn key_requests_are_rate_limited_per_caller() {
    let env = setup();
    let mut limits = env
        .query::<_, PublicConfig>(env.backend, env.alice, "get_config", ())
        .limits;
    limits.key_derivations_per_hour = 2;
    let denied: ChronoResult<()> = env.update(env.backend, env.alice, "set_limits", (limits.clone(),));
    assert_eq!(denied.unwrap_err(), ChronoError::Unauthorized);
    let set: ChronoResult<()> = env.update(env.backend, env.admin, "set_limits", (limits,));
    set.unwrap();

    let past = env.now_s() - 10;
    let ask = |who: Principal| -> ChronoResult<ByteBuf> {
        env.update(env.backend, who, "public_time_key", (past, ByteBuf::from(TRANSPORT_KEY.to_vec())))
    };
    assert!(ask(env.alice).is_ok());
    assert!(ask(env.alice).is_ok());
    assert!(matches!(ask(env.alice).unwrap_err(), ChronoError::LimitExceeded(_)));
    assert!(ask(env.bob).is_ok(), "other callers are unaffected");

    env.pic.advance_time(Duration::from_secs(3601));
    env.pic.tick();
    assert!(ask(env.alice).is_ok(), "the window resets");
}

// ---------------------------------------------------------------------------
// Media
// ---------------------------------------------------------------------------

#[test]
fn media_uploads_are_owner_bound_and_limited() {
    let env = setup();
    let start = |who: Principal, size: u64, chunks: u32| -> ChronoResult<u64> {
        env.update(env.backend, who, "start_media_upload", (StartMediaUploadArgs { size, chunk_count: chunks },))
    };
    assert_eq!(start(Principal::anonymous(), 10, 1).unwrap_err(), ChronoError::AnonymousCaller);
    assert!(matches!(start(env.alice, 0, 1).unwrap_err(), ChronoError::LimitExceeded(_)));
    assert!(matches!(start(env.alice, 11 * 1024 * 1024, 6).unwrap_err(), ChronoError::LimitExceeded(_)));
    assert!(matches!(start(env.alice, 4_000_000, 1).unwrap_err(), ChronoError::InvalidInput(_)));

    let id = start(env.alice, 8, 2).unwrap();
    let chunk = |who: Principal, index: u32, data: &[u8]| -> ChronoResult<()> {
        env.update(env.backend, who, "upload_media_chunk", (id, index, ByteBuf::from(data.to_vec())))
    };
    assert_eq!(chunk(env.bob, 0, b"evil").unwrap_err(), ChronoError::Unauthorized);
    assert!(matches!(chunk(env.alice, 5, b"data").unwrap_err(), ChronoError::InvalidInput(_)));
    chunk(env.alice, 0, b"data").unwrap();

    let finish = |who: Principal| -> ChronoResult<MediaInfo> {
        env.update(env.backend, who, "finish_media_upload", (id,))
    };
    assert!(matches!(finish(env.alice).unwrap_err(), ChronoError::InvalidInput(_)), "missing chunk");
    chunk(env.alice, 1, b"toolong").unwrap();
    assert!(matches!(finish(env.alice).unwrap_err(), ChronoError::InvalidInput(_)), "size mismatch");
    chunk(env.alice, 1, b"more").unwrap();
    assert!(finish(env.alice).unwrap().finished);

    // Unattached uploads are private to their owner.
    let read = |who: Principal| -> ChronoResult<ByteBuf> {
        env.query(env.backend, who, "get_media_chunk", (id, 0u32))
    };
    assert_eq!(read(env.alice).unwrap().as_slice(), b"data");
    assert_eq!(read(env.bob).unwrap_err(), ChronoError::Unauthorized);

    // Bob cannot attach Alice's media to his own lock.
    env.approve(env.icp, env.bob, 2 * E8);
    let mut args = env.lock_args(&[], PaymentMethod::ICP);
    args.media_id = Some(id);
    assert_eq!(env.create(env.bob, args).unwrap_err(), ChronoError::Unauthorized);

    // At most two open uploads per user.
    start(env.alice, 8, 1).unwrap();
    assert!(matches!(start(env.alice, 8, 1).unwrap_err(), ChronoError::LimitExceeded(_)));
}

#[test]
fn unattached_uploads_expire_but_attached_media_stays() {
    let env = setup();
    let orphan = env.upload(env.alice, &[b"orphan"]);
    let kept = env.upload(env.alice, &[b"kept"]);
    env.approve(env.icp, env.alice, E8 + FEE);
    let mut args = env.lock_args(&[], PaymentMethod::ICP);
    args.unlock_time = env.now_s() + 3 * 3600;
    args.media_id = Some(kept);
    env.create(env.alice, args).expect("create");

    // Past the 1h TTL plus the 10-minute cleanup timer.
    env.pic.advance_time(Duration::from_secs(3600 + 601));
    for _ in 0..5 {
        env.pic.tick();
    }

    let info = |id: u64| -> ChronoResult<MediaInfo> { env.query(env.backend, env.alice, "get_media_info", (id,)) };
    assert_eq!(info(orphan).unwrap_err(), ChronoError::NotFound);
    assert!(info(kept).unwrap().lock_id.is_some());
    // Attached (encrypted) media is readable by anyone.
    let chunk: ChronoResult<ByteBuf> = env.query(env.backend, env.bob, "get_media_chunk", (kept, 0u32));
    assert_eq!(chunk.unwrap().as_slice(), b"kept");
}

// ---------------------------------------------------------------------------
// Lists, burning, NFT interface
// ---------------------------------------------------------------------------

#[test]
fn lists_follow_ownership_recipients_and_unlock_time() {
    let env = setup();
    let private = env.paid_lock(env.alice, &[env.bob]);
    let public = env.paid_lock(env.alice, &[]);
    let ids = |page: LockPage| page.items.iter().map(|l| l.id).collect::<Vec<_>>();
    let owned = |who: Principal| -> LockPage {
        env.query(env.backend, who, "list_owned_chronolocks", (who, Env::page()))
    };
    let received = |who: Principal, unlocked: bool| -> LockPage {
        env.query(env.backend, who, "list_received_chronolocks", (who, unlocked, Env::page()))
    };

    assert_eq!(ids(owned(env.alice)), vec![public.id, private.id], "newest first");
    assert!(owned(env.bob).items.is_empty());
    assert_eq!(ids(received(env.bob, false)), vec![public.id, private.id]);
    assert_eq!(ids(received(env.carol, false)), vec![public.id], "carol only sees the public one");
    assert!(received(env.bob, true).items.is_empty(), "nothing unlocked yet");

    env.pic.advance_time(Duration::from_secs(3601));
    env.pic.tick();
    assert_eq!(ids(received(env.bob, true)), vec![public.id, private.id]);

    let all: LockPage = env.query(env.backend, Principal::anonymous(), "list_chronolocks", (Env::page(),));
    assert_eq!(all.total, 2);
    let stats = env.stats();
    assert_eq!((stats.total_locks, stats.unique_creators, stats.public_locks), (2, 1, 1));
}

#[test]
fn only_the_owner_or_an_admin_can_burn() {
    let env = setup();
    let media_id = env.upload(env.alice, &[b"bytes"]);
    env.approve(env.icp, env.alice, E8 + FEE);
    let mut args = env.lock_args(&[env.bob], PaymentMethod::ICP);
    args.media_id = Some(media_id);
    let lock = env.create(env.alice, args).unwrap();
    let other = env.paid_lock(env.alice, &[]);
    let burn = |who: Principal, id: u64| -> ChronoResult<()> { env.update(env.backend, who, "burn_chronolock", (id,)) };

    assert_eq!(burn(env.bob, lock.id).unwrap_err(), ChronoError::Unauthorized);
    burn(env.alice, lock.id).unwrap();
    assert_eq!(burn(env.alice, lock.id).unwrap_err(), ChronoError::NotFound);
    let gone: Option<Chronolock> = env.query(env.backend, env.alice, "get_chronolock", (lock.id,));
    assert!(gone.is_none());
    let info: ChronoResult<MediaInfo> = env.query(env.backend, env.alice, "get_media_info", (media_id,));
    assert_eq!(info.unwrap_err(), ChronoError::NotFound, "media is deleted with the lock");
    let received: LockPage =
        env.query(env.backend, env.bob, "list_received_chronolocks", (env.bob, false, Env::page()));
    assert_eq!(received.items.len(), 1, "only the public lock remains");

    burn(env.admin, other.id).expect("admins can burn");
    assert_eq!(env.stats().total_locks, 0);
}

#[test]
fn icrc7_transfer_moves_ownership() {
    let env = setup();
    let lock = env.paid_lock(env.alice, &[env.bob]);
    let transfer = |from: Principal, to: Principal| -> Vec<Option<Result<Nat, TransferError>>> {
        let arg = TransferArg {
            from_subaccount: None,
            to: account(to),
            token_id: Nat::from(lock.id),
            memo: None,
            created_at_time: None,
        };
        env.update(env.backend, from, "icrc7_transfer", (vec![arg],))
    };
    let owner = || -> Vec<Option<Account>> {
        env.query(env.backend, Principal::anonymous(), "icrc7_owner_of", (vec![Nat::from(lock.id)],))
    };

    assert!(matches!(transfer(env.bob, env.carol)[0], Some(Err(TransferError::Unauthorized))));
    assert!(matches!(transfer(env.alice, env.alice)[0], Some(Err(TransferError::InvalidRecipient))));
    assert!(matches!(transfer(env.alice, env.carol)[0], Some(Ok(_))));
    assert_eq!(owner()[0].as_ref().unwrap().owner, env.carol);

    let balances: Vec<Nat> = env.query(
        env.backend,
        Principal::anonymous(),
        "icrc7_balance_of",
        (vec![account(env.alice), account(env.carol)],),
    );
    assert_eq!(balances, vec![Nat::from(0u64), Nat::from(1u64)]);
    let tokens: Vec<Nat> = env.query(
        env.backend,
        Principal::anonymous(),
        "icrc7_tokens_of",
        (account(env.carol), None::<Nat>, None::<Nat>),
    );
    assert_eq!(tokens, vec![Nat::from(lock.id)]);
    let supply: Nat = env.query(env.backend, Principal::anonymous(), "icrc7_total_supply", ());
    assert_eq!(supply, Nat::from(1u64));
}

// ---------------------------------------------------------------------------
// Admin and upgrades
// ---------------------------------------------------------------------------

#[test]
fn only_admins_set_prices_and_withdraw() {
    let env = setup();
    env.paid_lock(env.alice, &[]);
    let new_prices = Prices { icp_text: E8 / 2, icp_media: 2 * E8, chrono_text: 10 * E8, chrono_media: 20 * E8 };
    let set = |who: Principal, p: Prices| -> ChronoResult<()> { env.update(env.backend, who, "set_prices", (p,)) };

    assert_eq!(set(env.alice, new_prices.clone()).unwrap_err(), ChronoError::Unauthorized);
    let mut zero = new_prices.clone();
    zero.icp_text = 0;
    assert!(matches!(set(env.admin, zero).unwrap_err(), ChronoError::InvalidInput(_)));
    set(env.admin, new_prices.clone()).unwrap();
    assert_eq!(env.info().prices, new_prices);

    // The new price applies to the next lock.
    env.approve(env.icp, env.bob, E8 / 2 + FEE);
    env.create(env.bob, env.lock_args(&[], PaymentMethod::ICP)).unwrap();
    assert_eq!(env.balance(env.icp, env.backend), E8 / 5 + E8 / 2);

    let withdraw = |who: Principal, amount: u64| -> ChronoResult<Nat> {
        env.update(env.backend, who, "withdraw", (PaymentMethod::ICP, account(env.carol), Nat::from(amount)))
    };
    assert_eq!(withdraw(env.alice, 1).unwrap_err(), ChronoError::Unauthorized);
    withdraw(env.admin, E8 / 2).unwrap();
    assert_eq!(env.balance(env.icp, env.carol), E8 / 2);
    assert_eq!(env.balance(env.icp, env.backend), E8 / 5 - FEE, "fee paid on top from the treasury");
    assert!(matches!(
        withdraw(env.admin, 10 * E8).unwrap_err(),
        ChronoError::Payment(PaymentError::InsufficientFunds { .. })
    ));

    // Admins can be added and removed.
    let add: ChronoResult<()> = env.update(env.backend, env.admin, "add_admin", (env.bob,));
    add.unwrap();
    set(env.bob, Prices::default()).expect("bob is now an admin");
    let remove: ChronoResult<()> = env.update(env.backend, env.admin, "remove_admin", (env.bob,));
    remove.unwrap();
    assert_eq!(set(env.bob, Prices::default()).unwrap_err(), ChronoError::Unauthorized);
}

#[test]
fn state_survives_an_upgrade() {
    let env = setup();
    env.connect_chrono();
    let media_id = env.upload(env.alice, &[b"persisted"]);
    env.approve(env.icp, env.alice, E8 + FEE);
    let mut args = env.lock_args(&[env.bob], PaymentMethod::ICP);
    args.media_id = Some(media_id);
    let lock = env.create(env.alice, args).unwrap();
    let before = env.info();

    env.pic
        .upgrade_canister(env.backend, backend_wasm(), candid::encode_one(None::<InitArgs>).unwrap(), Some(env.admin))
        .expect("upgrade");

    let after: Option<Chronolock> = env.query(env.backend, env.alice, "get_chronolock", (lock.id,));
    let after = after.expect("lock survives");
    assert_eq!(after.user_keys, lock.user_keys);
    assert_eq!(after.payment, lock.payment);
    assert_eq!(env.info().collected, before.collected);
    assert_eq!(env.info().chrono_ledger, Some(env.chrono));
    let chunk: ChronoResult<ByteBuf> = env.query(env.backend, env.bob, "get_media_chunk", (media_id, 0u32));
    assert_eq!(chunk.unwrap().as_slice(), b"persisted");
    let config: PublicConfig = env.query(env.backend, env.alice, "get_config", ());
    assert_eq!(config.vetkd_key_name, "test_key_1");
    // Ids keep counting up and the cleanup timer is re-armed after upgrade.
    assert_eq!(env.paid_lock(env.alice, &[]).id, lock.id + 1);
}
