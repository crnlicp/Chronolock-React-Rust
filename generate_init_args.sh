#!/bin/bash

# Generating initial parameter for canisters

PRINCIPAL=$(dfx identity get-principal)
cat <<EOF > src/backend/crnl_ledger/init_args.did
(
  "Chronolock",
  "CRNL",
  100000000000000000000 : nat,
  31536000 : nat64,
  100000 : nat,
  principal "$PRINCIPAL"
)
EOF

cat <<EOF > src/backend/chronolock/init_args.did
(
  principal "$PRINCIPAL"
)
EOF