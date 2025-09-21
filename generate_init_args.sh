#!/bin/bash

# Generating initial parameters for canisters

# Get the admin principal from the current DFX identity
ADMIN_PRINCIPAL=$(dfx identity get-principal)

# Detect network from command parameters
NETWORK=""
if [ "$1" = "--ic" ]; then
  NETWORK="ic"
  echo "🌐 Auto-detected network: IC Mainnet"
elif [ "$1" = "--local" ]; then
  NETWORK="local"
  echo "🏠 Auto-detected network: Local Development"
elif [ "$1" = "--network" ] && [ -n "$2" ]; then
  NETWORK="$2"
  echo "🔧 Auto-detected network: $NETWORK"
else
  # No network specified, ask user interactively
  echo "🔧 Chronolock Canister Configuration"
  echo "=================================="
  echo "1) Local development (uses management canister vetKD)"
  echo "2) IC Mainnet (uses management canister vetKD)"
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

# Configure based on network
if [ "$NETWORK" = "ic" ]; then
  echo ""
  echo "🌐 IC Mainnet Configuration"
  echo "=========================="
  echo "✅ Using production vetKD via management canister directly"
  echo "📖 No separate vetKD canister required - all calls go to management canister"
  
  NETWORK_NAME="ic"
  
  # Switch to IC-specific dfx.json (clean configuration)
  echo "🔄 Configuring dfx.json for IC mainnet..."
  cp dfx.ic.json dfx.json
  
  echo "✅ Configured dfx.json for IC mainnet deployment"
else
  # Local development - use management canister directly
  
  # Switch to local dfx.json (clean configuration)
  echo "🔄 Configuring dfx.json for local development..."
  cp dfx.local.json dfx.json
  
  NETWORK_NAME="local"
  echo "✅ Configured dfx.json for local development"
fi

echo ""
echo "📝 Generating initialization arguments..."

# Generate init_args.did for crnl_ledger
cat <<EOF > src/backend/crnl_ledger_canister/ledger_init_args.did
(
  "Chronolock",
  "CRNL",
  10000000000000000 : nat,
  31536000 : nat64,
  10000 : nat,
  principal "$ADMIN_PRINCIPAL"
)
EOF

# Generate init_args.did for chronolock (uses management canister directly)
cat <<EOF > src/backend/chronolock_canister/chronolock_init_args.did
(
  principal "$ADMIN_PRINCIPAL",
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
echo "   🔐 VetKD System:    Management canister (direct)"
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
  echo "✅ VetKD calls go directly to management canister"
else
  echo ""
  echo "🚀 Next steps for local development:"
  echo "   1. dfx start --clean --background  (if not running)"
  echo "   2. dfx canister create --all"
  echo "   3. dfx deploy"
  echo ""
  echo "✅ VetKD calls go directly to management canister (local testing)"
fi

echo ""
echo "💡 To switch networks later, run this script again and select a different option."