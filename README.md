# Chronolock

Time-locked, encrypted messages and media on the Internet Computer. Content is
encrypted in the browser and can only be opened by its recipients (or anyone,
for public locks) after the unlock time, enforced with vetKeys.

## Project layout

```
icp.yaml                  # canisters, environments (icp-cli)
Cargo.toml                # Rust workspace
backend/                  # Rust canister: locks, media, vetKD keys, ICRC-7
  canister.yaml           # icp-cli canister config
  backend.did             # committed Candid interface (used by bindgen)
frontend/                 # React 19 + MUI 9 + Vite 8 app, served by the static-site canister
  canister.yaml           # icp-cli canister config
  candid/                 # interfaces of external canisters (ICRC ledgers)
  public/_redirects       # SPA routing
  public/_headers         # security + CORS headers
  public/.well-known/     # ic-domains, Internet Identity metadata and origins
  src/app/                # App shell, routes, theme
  src/pages/              # Home, Create, Collection, Lock, Wallet, legal pages
  src/features/           # create wizard, collection, wallet (send, OISY)
  src/components/         # layout, clock-lock widget, shared UI
  src/api/                # typed canister hooks (TanStack Query)
  src/auth/               # Internet Identity sign-in (@icp-sdk/auth)
  src/lib/                # env (ic_env cookie), crypto, errors, formatting
  tests/                  # unit tests + e2e test against a local network
scripts/                  # local-token.sh, clean.sh
.icp/data/mappings/       # mainnet canister ids (committed)
```

## Prerequisites

- Node.js 22+, pnpm 10
- Rust with `wasm32-unknown-unknown` (`rustup target add wasm32-unknown-unknown`)
- icp-cli and ic-wasm: `npm i -g @icp-sdk/icp-cli @icp-sdk/ic-wasm`

## Run locally

```bash
icp network start -d                 # local network on http://localhost:8000
icp deploy --identity anonymous      # backend + frontend
```

Open http://frontend.local.localhost:8000. Sign-in uses mainnet Internet
Identity (id.ai), which local networks trust.

For hot reload, keep the network running and start Vite (it sets the `ic_env`
cookie and proxies `/api` to the local network):

```bash
pnpm --dir frontend dev              # http://localhost:3000
```

Stop the network with `icp network stop`. `scripts/clean.sh` removes every
build output (target, dist, bindings, node_modules, local network state).

## $CHRONO token

$CHRONO is launched on [Soon](https://soonlaunchpad.fun): a standard
ICRC-1/ICRC-2 ledger (8 decimals, 0.0001 fee). The app has no ledger of its
own. An admin connects the token at runtime, as ICPBonsai does:

```bash
icp canister call backend set_token '(opt record { ledger = principal "<ledger>"; market = opt principal "<market>"; symbol = "CHRONO" })' -e ic
icp canister call backend set_token '(null)' -e ic   # disconnect
```

Locally, install a test ledger with Soon's parameters and connect it:

```bash
scripts/local-token.sh setup                  # once per fresh local network
scripts/local-token.sh mint <principal> 50000 # credit test $CHRONO
```

Admin calls need a non-anonymous identity: the script creates a local-only
`chronolock-local-dev` identity and makes it a controller of the local backend
(set `ICP_IDENTITY` to use another one).

## Payments and treasury

Creating a chronolock costs, by default:

| | ICP | $CHRONO |
|---|---|---|
| Text | 0.2 | 1,000 |
| Media | 1 | 5,000 |

The user also pays two ledger fees (approve + payment). Flow: the browser calls
`icrc2_approve` on the chosen ledger (price + fee, 10-minute expiry, spender =
backend), then `create_chronolock` validates everything, pulls the price with
`icrc2_transfer_from` into the backend's default account (the treasury) and only
then stores the lock. Pulls are deduplicated by memo + timestamp, so retries
never double-charge. `get_payment_info` returns prices, ledgers, the treasury
account and totals collected.

Admin commands (controllers and added admins):

```bash
# Prices in e8s (both ICP and Soon tokens have 8 decimals)
icp canister call backend set_prices '(record { icp_text = 20_000_000 : nat64; icp_media = 100_000_000 : nat64; chrono_text = 100_000_000_000 : nat64; chrono_media = 500_000_000_000 : nat64 })' -e ic
# Withdraw from the treasury (amount in e8s; the ledger fee is paid on top)
icp canister call backend withdraw '(variant { ICP }, record { owner = principal "<to>"; subaccount = null }, 100_000_000 : nat)' -e ic
icp canister call backend withdraw '(variant { CHRONO }, record { owner = principal "<to>"; subaccount = null }, 100_000_000_000 : nat)' -e ic
```

Locally the network seeds the anonymous identity with ICP. Fund a test user with
`icp token transfer 5 <principal> --identity anonymous` and
`scripts/local-token.sh mint <principal> 10000`.

## Wallet

`/wallet` is the user's Chronolock wallet: their Internet Identity principal.

- Balances of ICP and $CHRONO, read straight from the ledgers (no cycles cost).
- Receive: principal (ICP and $CHRONO) and the legacy ICP account ID for
  exchanges, with QR codes.
- Send with a review step: ICRC-1 transfer to a principal or ICRC account, or
  the legacy ICP `transfer` to a 64-hex account ID.
- Top up from OISY (`@icp-sdk/signer`, ICRC-25/27/49 over the OISY popup): the
  user approves one transfer from OISY into their Chronolock wallet. Internet
  Identity stays the app account because uploads and unlocking make many calls,
  which would each need a wallet approval. OISY is mainnet-only.

## Tests

```bash
cargo test -p backend
pnpm --dir frontend test
# end-to-end against the running local deployment:
ICP_E2E=1 pnpm --dir frontend exec vitest run tests/e2e.local.test.ts
```

## Backend interface

After changing backend endpoints, regenerate the committed Candid file:

```bash
icp build backend
candid-extractor target/wasm32-unknown-unknown/release/backend.wasm > backend/backend.did
```

The frontend regenerates its TypeScript bindings from it on every build.

## Mainnet

Mainnet ids live in `.icp/data/mappings/ic.ids.json`. Deploy, then connect the
token with `set_token` (above):

```bash
icp deploy -e ic --identity <your-identity>
```

The `ic` environment installs the backend with the production vetKD key
(`key_1`). The key is fixed at install time; changing it later would make
existing locks undecryptable. The existing mainnet frontend canister is a legacy
asset canister, so its first deploy needs `--mode reinstall`.

## Domain: chronolock.app

The frontend canister serves `chronolock.app` and `www.chronolock.app`
(`frontend/public/.well-known/ic-domains`). Internet Identity accounts are
derived from the canister's own address (`https://<frontend-id>.icp0.io`),
which lists both domains in `/.well-known/ii-alternative-origins`. Users get
the same principal on the canister URL and on the domain, and the domain can
never orphan accounts. Canister calls go to `icp-api.io` (the agent's default
on custom domains); never set `host` to the domain.

One-time setup per domain (`chronolock.app`, then `www.chronolock.app`):

1. DNS (disable your DNS provider's own SSL/TLS first):

   | Type | Host | Value |
   |---|---|---|
   | ALIAS/ANAME (apex) or CNAME | `chronolock.app` | `chronolock.app.icp1.io` |
   | TXT | `_canister-id.chronolock.app` | `l4pdm-ziaaa-aaaal-qssxq-cai` |
   | CNAME | `_acme-challenge.chronolock.app` | `_acme-challenge.chronolock.app.icp2.io` |

2. Deploy the frontend so `/.well-known/ic-domains` is live, then validate and register:

   ```bash
   curl -sL "https://icp.net/custom-domains/v1/chronolock.app/validate" | jq
   curl -sL -X POST "https://icp.net/custom-domains/v1/chronolock.app" | jq
   curl -sL "https://icp.net/custom-domains/v1/chronolock.app" | jq   # until "registered"
   ```

## How encryption works

See [docs/ENCRYPTION_AND_DECRYPTION.md](docs/ENCRYPTION_AND_DECRYPTION.md).
In short: content (including titles) is AES-256-GCM encrypted in the browser;
the AES key is IBE-encrypted with vetKeys to an identity binding the unlock time
and recipient; after the unlock time the backend derives the matching vetKey for
the caller. The IBE public key is derived offline, never trusted from the canister.
