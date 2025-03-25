#!/bin/bash

# Generating initial parameters for canisters

# Get the admin principal from the current DFX identity
ADMIN_PRINCIPAL=$(dfx identity get-principal)

# Get the canister ID of the vetkd_mock canister
VETKD_MOCK_CANISTER_ID=$(dfx canister id vetkd_mock)

# Check if VETKD_MOCK_CANISTER_ID was retrieved successfully
if [ -z "$VETKD_MOCK_CANISTER_ID" ]; then
  echo "Error: Could not retrieve vetkd_mock canister ID. Ensure the canister is created and DFX is configured correctly."
  exit 1
fi

# Generate init_args.did for crnl_ledger
cat <<EOF > src/backend/crnl_ledger/init_args.did
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
cat <<EOF > src/backend/chronolock/init_args.did
(
  principal "$ADMIN_PRINCIPAL",
  opt principal "$VETKD_MOCK_CANISTER_ID"
)
EOF

echo "Generated init_args.did files:"
echo "- src/backend/crnl_ledger/init_args.did"
echo "- src/backend/chronolock/init_args.did"