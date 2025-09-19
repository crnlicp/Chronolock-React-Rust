#!/bin/bash

# Generating initial parameters for canisters

# Get the admin principal from the current DFX identity
ADMIN_PRINCIPAL=$(dfx identity get-principal)

# Detect network or ask user
NETWORK=""
if [ "$1" = "--network" ] && [ -n "$2" ]; then
  NETWORK="$2"
else
  echo "🔧 Chronolock Canister Configuration"
  echo "=================================="
  echo "1) Local development (uses chainkey_testing_canister)"
  echo "2) IC Mainnet (requires production VetKD canister ID)"
  echo ""
  read -p "Select deployment target (1 or 2): " choice
  
  case $choice in
    1)
      NETWORK="local"
      ;;
    2)
      NETWORK="ic"
      ;;
    *)
      echo "❌ Invalid choice. Exiting."
      exit 1
      ;;
  esac
fi

# Configure VetKD canister ID and dfx.json based on network
if [ "$NETWORK" = "ic" ]; then
  echo ""
  echo "🌐 IC Mainnet Configuration"
  echo "=========================="
  echo "⚠️  You need the production VetKD canister ID from DFINITY."
  echo "📖 Check DFINITY documentation or contact support for the correct ID."
  echo ""
  read -p "Enter production VetKD canister ID: " VETKD_CANISTER_ID
  
  if [ -z "$VETKD_CANISTER_ID" ]; then
    echo "❌ VetKD canister ID is required for mainnet deployment."
    exit 1
  fi
  
  NETWORK_NAME="ic"
  
  # Switch to IC-specific dfx.json (without chainkey_testing_canister)
  echo "🔄 Configuring dfx.json for IC mainnet (removing chainkey_testing_canister)..."
  cp dfx.ic.json dfx.json
  
  echo "✅ Using production VetKD canister: $VETKD_CANISTER_ID"
  echo "✅ Configured dfx.json for IC mainnet deployment"
else
  # Local development - use chainkey_testing_canister
  
  # Switch to local dfx.json (with chainkey_testing_canister)
  echo "🔄 Configuring dfx.json for local development (including chainkey_testing_canister)..."
  cp dfx.local.json dfx.json
  
  CHAINKEY_TESTING_CANISTER_CANISTER_ID=$(dfx canister id chainkey_testing_canister 2>/dev/null)
  
  if [ -z "$CHAINKEY_TESTING_CANISTER_CANISTER_ID" ]; then
    echo "❌ Could not retrieve chainkey_testing_canister canister ID."
    echo "💡 Make sure you've run 'dfx canister create --all' first."
    exit 1
  fi
  
  VETKD_CANISTER_ID="$CHAINKEY_TESTING_CANISTER_CANISTER_ID"
  NETWORK_NAME="local"
  echo "✅ Using local chainkey_testing_canister: $VETKD_CANISTER_ID"
  echo "✅ Configured dfx.json for local development"
fi

echo ""
echo "📝 Generating initialization arguments..."

# Generate init_args.did for crnl_ledger
cat <<EOF > src/backend/crnl_ledger_canister/ledger_init_args.did
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
cat <<EOF > src/backend/chronolock_canister/chronolock_init_args.did
(
  principal "$ADMIN_PRINCIPAL",
  opt principal "$VETKD_CANISTER_ID",
  opt "$NETWORK_NAME"
)
EOF

echo ""
echo "✅ Generated init_args.did files:"
echo "   📄 src/backend/crnl_ledger_canister/ledger_init_args.did"
echo "   📄 src/backend/chronolock_canister/chronolock_init_args.did"
echo ""
echo "🎯 Configuration Summary:"
echo "   👤 Admin Principal: $ADMIN_PRINCIPAL"
echo "   🔐 VetKD Canister:  $VETKD_CANISTER_ID"
echo "   🌐 Network:         $NETWORK_NAME"
echo "   📋 dfx.json:        Configured for $NETWORK_NAME deployment"

if [ "$NETWORK" = "ic" ]; then
  echo ""
  echo "🚀 Next steps for IC mainnet deployment:"
  echo "   1. pnpm run build"
  echo "   2. dfx deploy --network ic crnl_ledger_canister"
  echo "   3. dfx deploy --network ic chronolock_canister"
  echo "   4. dfx deploy --network ic frontend"
  echo ""
  echo "✅ chainkey_testing_canister automatically excluded from IC deployment"
else
  echo ""
  echo "🚀 Next steps for local development:"
  echo "   1. dfx start --clean --background  (if not running)"
  echo "   2. dfx canister create --all"
  echo "   3. dfx deploy"
  echo ""
  echo "✅ chainkey_testing_canister included for local testing"
fi

echo ""
echo "💡 To switch networks later, run this script again and select a different option."