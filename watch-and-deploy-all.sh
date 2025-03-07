#!/bin/bash

# Get all Rust canisters from dfx.json
RUST_CANISTERS=$(jq -r '.canisters | to_entries[] | select(.value.type == "rust") | .key' dfx.json)

# Function to build and deploy a canister
build_and_deploy() {
    canister=$1
    echo "Building and deploying $canister"
    cargo build --target wasm32-unknown-unknown --package $canister && dfx deploy $canister --build-output --no-wallet
}

# Watch for changes in all Rust canisters
cargo watch -w src -x "check" -s "$(for canister in $RUST_CANISTERS; do echo build_and_deploy $canister; done)"