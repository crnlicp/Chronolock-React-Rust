#!/usr/bin/env bash
# Local stand-in for the $CHRONO token launched on Soon (mainnet only).
#
#   scripts/local-token.sh setup            # install a test ICRC ledger and connect it
#   scripts/local-token.sh mint <principal> [amount]   # credit test $CHRONO (default 100000)
#
# Uses the official ICRC-1/ICRC-2 ledger wasm with Soon's parameters
# (8 decimals, 0.0001 fee). The ledger is detached from the project, like
# ICPBonsai's local token, so nothing token-specific lives in icp.yaml.
set -euo pipefail
cd "$(dirname "$0")/.."

RELEASE=ledger-suite-icrc-2026-03-09
SHA256=354dd6ecfdc72b5409805b31dea22c9db11df6e14095a5a68924eb63535e6d8a
DIR=.icp/cache/local-token
WASM=$DIR/ic-icrc1-ledger.wasm.gz
ID_FILE=$DIR/ledger.id
DEV_ID=chronolock-local-dev
icpx() { icp "$@" --identity "${ICP_IDENTITY:-anonymous}"; }

# Admin calls need a non-anonymous identity. Unless ICP_IDENTITY is set, use a
# throwaway local-only identity and make it a controller (= admin) of the
# local backend. Never use it on mainnet.
admin_identity() {
  if [ -n "${ICP_IDENTITY:-}" ] && [ "$ICP_IDENTITY" != anonymous ]; then echo "$ICP_IDENTITY"; return; fi
  if ! icp identity list | grep -qw "$DEV_ID"; then
    icp identity new "$DEV_ID" --storage plaintext >/dev/null
  fi
  icpx canister settings update backend -e local \
    --add-controller "$(icp identity principal --identity "$DEV_ID")" >/dev/null
  echo "$DEV_ID"
}

ledger_id() { cat "$ID_FILE" 2>/dev/null || { echo "Run '$0 setup' first." >&2; exit 1; }; }

setup() {
  mkdir -p "$DIR"
  if [ ! -f "$WASM" ]; then
    curl -sfL -o "$WASM" "https://github.com/dfinity/ic/releases/download/$RELEASE/ic-icrc1-ledger.wasm.gz"
  fi
  echo "$SHA256  $WASM" | sha256sum -c --quiet
  # The caller is the minting account, so `mint` is a plain transfer from it.
  local minter; minter=$(icpx identity principal)
  local ledger; ledger=$(icpx canister create --detached -e local -q)
  icpx canister install "$ledger" -e local --wasm "$WASM" --args "(variant { Init = record {
    token_symbol = \"CHRONO\"; token_name = \"Chrono (local test)\";
    decimals = opt (8 : nat8); transfer_fee = 10_000 : nat;
    minting_account = record { owner = principal \"$minter\" };
    fee_collector_account = null; max_memo_length = opt (32 : nat16);
    metadata = vec {}; initial_balances = vec {};
    feature_flags = opt record { icrc2 = true };
    archive_options = record {
      num_blocks_to_archive = 1000 : nat64; trigger_threshold = 2000 : nat64;
      controller_id = principal \"$minter\";
      max_transactions_per_response = null; max_message_size_bytes = null;
      cycles_for_archive_creation = null; node_max_memory_size_bytes = null;
      more_controller_ids = null;
    };
    index_principal = null;
  }})"
  echo "$ledger" > "$ID_FILE"
  icp canister call backend set_token --identity "$(admin_identity)" \
    "(opt record { ledger = principal \"$ledger\"; market = null; symbol = \"CHRONO\" })" -e local
  echo "Local \$CHRONO ledger: $ledger"
}

mint() {
  local to=${1:?principal required} amount=${2:-100000}
  icpx token "$(ledger_id)" transfer "$amount" "$to" -e local
}

case "${1:-}" in
  setup) setup ;;
  mint) shift; mint "$@" ;;
  *) sed -n '2,9p' "$0"; exit 1 ;;
esac
