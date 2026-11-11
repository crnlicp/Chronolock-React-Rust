#!/usr/bin/env bash
# Runs the backend unit tests and the PocketIC integration tests.
# Builds the backend wasm, fetches the pinned ICRC ledger wasm, and uses the
# PocketIC server that ships with icp-cli (or downloads one if none is found).
#
#   scripts/test-backend.sh [extra cargo test args]
set -euo pipefail
cd "$(dirname "$0")/.."

RELEASE=ledger-suite-icrc-2026-03-09
SHA256=354dd6ecfdc72b5409805b31dea22c9db11df6e14095a5a68924eb63535e6d8a
DIR=.icp/cache/local-token
WASM=$DIR/ic-icrc1-ledger.wasm.gz

mkdir -p "$DIR"
if [ ! -f "$WASM" ]; then
  curl -sfL -o "$WASM" "https://github.com/dfinity/ic/releases/download/$RELEASE/ic-icrc1-ledger.wasm.gz"
fi
echo "$SHA256  $WASM" | sha256sum -c --quiet

cargo build --target wasm32-unknown-unknown --release -p backend

if [ -z "${POCKET_IC_BIN:-}" ]; then
  # Newest PocketIC server bundled with icp-cli's network launcher, if any.
  found=$(ls -d "${ICP_HOME:-$HOME/.local/share/icp-cli}"/pkg/network-launcher/*/pocket-ic 2>/dev/null | sort -V | tail -1 || true)
  if [ -n "$found" ]; then export POCKET_IC_BIN="$found"; fi
fi

export BACKEND_WASM="$PWD/target/wasm32-unknown-unknown/release/backend.wasm"
export LEDGER_WASM="$PWD/$WASM"
cargo test -p backend "$@"
