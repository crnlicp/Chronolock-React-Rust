#!/bin/bash

# Generating initial parameters for canisters

# Get the admin principal from the current DFX identity
ADMIN_PRINCIPAL=$(dfx identity get-principal)

# Get the canister ID of the chainkey_testing_canister canister
CHAINKEY_TESTING_CANISTER_CANISTER_ID=$(dfx canister id chainkey_testing_canister)

# Check if CHAINKEY_TESTING_CANISTER_CANISTER_ID was retrieved successfully
if [ -z "$CHAINKEY_TESTING_CANISTER_CANISTER_ID" ]; then
  echo "Error: Could not retrieve chainkey_testing_canister canister ID. Ensure the canister is created and DFX is configured correctly."
  exit 1
fi

# Generate init_args.did for crnl_ledger
cat <<EOF > src/backend/crnl_ledger/ledger_init_args.did
(
  "Chronolock",
  "CRNL",
  100000000000000000000 : nat,
  31536000 : nat64,
  100000 : nat,
  principal "$ADMIN_PRINCIPAL"
)
EOF

# Generate init_args.did for chronolock
cat <<EOF > src/backend/chronolock/chronolock_init_args.did
(
  principal "$ADMIN_PRINCIPAL",
  opt principal "$CHAINKEY_TESTING_CANISTER_CANISTER_ID"
)
EOF

echo "Generated init_args.did files:"
echo "- src/backend/crnl_ledger/ledger_init_args.did"
echo "- src/backend/chronolock/chronolock_init_args.did"