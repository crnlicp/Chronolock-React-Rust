#!/bin/bash

# Parse command line arguments
NETWORK="local"
if [ "$1" = "--ic" ]; then
  NETWORK="ic"
fi

echo "🚀 Chronolock Backend Setup"
echo "=========================="
echo "📋 Network: $NETWORK"
echo ""

# dfx stop && dfx start --clean --background &&
# bash manage_identities.sh &&
cargo install candid-extractor &&
cargo clean && 
export DFX_WARNING=-mainnet_plaintext_identity &&
dfx canister create --all &&
rustup target add wasm32-unknown-unknown &&

# Pass network flag to generate_init_args.sh
if [ "$NETWORK" = "ic" ]; then
  bash generate_init_args.sh --ic
else
  bash generate_init_args.sh --local
fi &&

cargo build --target wasm32-unknown-unknown --release &&
candid-extractor target/wasm32-unknown-unknown/release/crnl_ledger_canister.wasm > src/backend/crnl_ledger_canister/crnl_ledger_canister.did &&
echo "Extracted crnl_ledger_canister.did" &&
candid-extractor target/wasm32-unknown-unknown/release/chronolock_canister.wasm > src/backend/chronolock_canister/chronolock_canister.did &&
echo "Extracted chronolock_canister.did" &&

# Generate declarations for remaining canisters

dfx generate crnl_ledger_canister && dfx generate chronolock_canister &&

# Deploy based on network
if [ "$NETWORK" = "ic" ]; then
  echo ""
  echo "🌐 Deploying to IC mainnet with cycle allocation..."
  echo "📊 Checking wallet balance before deployment..."
  dfx wallet --network ic balance
  
  echo ""
  echo "🚀 Deploying canisters to IC mainnet..."
  echo "📦 Deploying crnl_ledger_canister..."
  dfx deploy --network ic --with-cycles 2000000000000 crnl_ledger_canister &&
  echo "📦 Deploying chronolock_canister..."
  dfx deploy --network ic --with-cycles 3000000000000 chronolock_canister &&
  echo "📦 Deploying frontend..."
  dfx deploy --network ic --with-cycles 2000000000000 frontend &&
  
  echo ""
  echo "📊 Checking wallet balance after deployment..."
  dfx wallet --network ic balance
  echo ""
  echo "✅ IC mainnet deployment complete!"
  echo "🔗 Canisters deployed:"
  echo "   • crnl_ledger_canister"
  echo "   • chronolock_canister"
  echo "   • frontend"
  echo "✅ All canisters use management canister for vetKD"
else
  echo ""
  echo "🏠 Deploying to local network (free cycles)..."
  dfx deploy &&
  echo ""
  echo "✅ Local deployment complete!"
  echo "🔗 Canisters deployed:"
  echo "   • crnl_ledger_canister"
  echo "   • chronolock_canister"
  echo "   • frontend"
  echo "   • internet_identity"
fi

echo ""
echo "🎉 Backend setup completed successfully!"