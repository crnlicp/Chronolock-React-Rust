# Mainnet deployment checklist

Mainnet ids (`.icp/data/mappings/ic.ids.json`) point at the existing canisters:
`backend` = `l3ofy-uqaaa-aaaal-qssxa-cai`, `frontend` = `l4pdm-ziaaa-aaaal-qssxq-cai`.
Both currently run the old dfx-era code, so the first deploy replaces them.
Existing data is intentionally discarded.

Use a named identity, never `anonymous`. With a password-protected identity pass
`--identity-password-file <file>`. Deploying to mainnet spends real cycles; ask
before running anything against `-e ic`.

## 0. Before you deploy

- [ ] `scripts/test-backend.sh`, `pnpm --dir frontend test` and the local e2e suite pass.
- [ ] A clean build works: `scripts/clean.sh && icp network start -d && icp deploy --identity anonymous`.
- [ ] Deploying identity has cycles: `icp cycles balance -n ic --identity <id>`
      (each existing canister is upgraded/reinstalled, no new ones are created).
- [ ] Add a backup controller to both canisters so a lost key cannot brick them:
      `icp canister settings update backend --add-controller <backup> -e ic`.

## 1. Deploy

The backend's Candid interface and the frontend canister type both changed
incompatibly, so both need a reinstall (state is discarded; the vetKD key is
chosen at this moment and fixed from then on):

```bash
icp deploy backend  -e ic --mode reinstall --identity <id>   # installs with key_1
icp deploy frontend -e ic --mode reinstall --identity <id>   # legacy asset canister -> certified assets
```

- [ ] `icp canister call backend get_config '()' -e ic` shows `vetkd_key_name = "key_1"` and `scheme = 2`.
- [ ] `icp canister call backend get_payment_info '()' -e ic` shows the ICP ledger `ryjl3-tyaaa-aaaaa-aaaba-cai` and default prices.
- [ ] The backend holds cycles for vetKD: each key derivation costs about 26.2B cycles
      (`key_1`), paid by the backend. At the default limit of 60 keys per user per
      hour a busy day can add up; top up and watch `icp canister status backend -e ic`.

## 2. Connect $CHRONO (after the token launches on Soon)

- [ ] Get the token's Soon **ledger** and **market** canister ids from its Soon page.
- [ ] Confirm the ledger supports approvals:
      `icp canister call <ledger> icrc1_supported_standards '()' -e ic` lists `ICRC-2`.
- [ ] Connect it:
      `icp canister call backend set_token '(opt record { ledger = principal "<ledger>"; market = opt principal "<market>"; symbol = "CHRONO" })' -e ic --identity <admin>`
- [ ] Check 8 decimals and the fee (the app reads both from the ledger, so other values also work, but
      the default prices of 1,000 / 5,000 assume 8 decimals; adjust with `set_prices` otherwise).
- [ ] Create one text and one media chronolock paying with $CHRONO and confirm the treasury balance.

## 3. Domain (chronolock.app)

Files for the domain are already served by the frontend (`/.well-known/ic-domains`,
`ii-alternative-origins`, `ii-derivation-origin`). One-time DNS and registration:

- [ ] Disable the DNS provider's own SSL/TLS (it breaks certificate provisioning).
- [ ] Add the three records for `chronolock.app` and again for `www.chronolock.app`
      (see the README, "Domain").
- [ ] Validate, register, then poll until `registered`:
      `curl -sL "https://icp.net/custom-domains/v1/chronolock.app/validate" | jq`
      `curl -sL -X POST "https://icp.net/custom-domains/v1/chronolock.app" | jq`
      `curl -sL "https://icp.net/custom-domains/v1/chronolock.app" | jq`
- [ ] Sign in on both `https://l4pdm-ziaaa-aaaal-qssxq-cai.icp0.io` and
      `https://chronolock.app` and confirm the **same principal** (the Wallet page shows it).
- [ ] Never point the agent `host` at the custom domain (the app already relies on the default).

## 4. Smoke test on mainnet

- [ ] Sign in with Internet Identity; the ICP balance shows in the header.
- [ ] Top up from OISY (Wallet page) and confirm the balance updates.
- [ ] Create a text chronolock unlocking in a few minutes, open it after the time passes.
- [ ] Create a media chronolock, open it, download the file.
- [ ] Burn one and confirm it disappears from the lists.
- [ ] Withdraw a small amount from the treasury:
      `icp canister call backend withdraw '(variant { ICP }, record { owner = principal "<to>"; subaccount = null }, 1_000_000 : nat)' -e ic --identity <admin>`

## 5. Later upgrades

Upgrades keep all state (everything is in stable memory):

```bash
icp deploy -e ic --identity <id>        # default mode upgrades
```

Never change `vetkd_key_name`, the vetKD context (`chronolock/v2`) or the IBE
identity layout in place: that would make every existing chronolock unreadable.
A new scheme needs a new `scheme` number and backwards-compatible decrypting.
`icp deploy --mode reinstall` wipes all chronolocks and must not be used again.

## Known limits

- A payment is taken in one ledger call before the lock is stored. The lock insert
  cannot fail once validation has passed, and retries are deduplicated by the
  ledger, but if the canister ever trapped between the two the payment would have
  to be refunded manually with `withdraw`; the treasury log (`icp canister logs`)
  records every payment with payer and block.
- Recipient principals and unlock times are public on-chain (needed for lists).
  Titles, text and media are encrypted.
- Opening a public chronolock requires sign-in so key requests can be rate limited.
