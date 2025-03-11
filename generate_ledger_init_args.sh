#!/bin/bash

# Generating initial parameter for icrc1_ledger canister

PRINCIPAL=$(dfx identity get-principal)
cat <<EOF > src/backend/crnl_ledger/ledger_init_args.did
(
  "Chronolock",
  "CRNL",
  100000000000000000000 : nat,
  31536000 : nat64,
  100000 : nat,
  principal "$PRINCIPAL"
)
EOF