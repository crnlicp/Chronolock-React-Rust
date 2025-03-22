#!/bin/bash

dfx stop && dfx start --clean --background &&
bash manage_identities.sh &&
bash generate_init_args.sh &&
cargo install candid-extractor &&
dfx canister create --all &&
dfx build --all &&
candid-extractor target/wasm32-unknown-unknown/release/hello_world.wasm > src/backend/hello_world/hello_world.did &&
echo "Extracted hello_world.did" &&
candid-extractor target/wasm32-unknown-unknown/release/crnl_ledger.wasm > src/backend/crnl_ledger/crnl_ledger.did &&
echo "Extracted crnl_ledger.did" &&
candid-extractor target/wasm32-unknown-unknown/release/chronolock.wasm > src/backend/chronolock/chronolock.did &&
echo "Extracted chronolock.did" &&
dfx deploy