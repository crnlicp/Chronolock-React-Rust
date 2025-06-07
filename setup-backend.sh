#!/bin/bash

dfx stop && dfx start --clean --background &&
bash manage_identities.sh &&
cargo install candid-extractor &&
cargo clean && 
dfx canister create --all &&
bash generate_init_args.sh &&
cargo build --target wasm32-unknown-unknown --release &&
candid-extractor target/wasm32-unknown-unknown/release/crnl_ledger.wasm > src/backend/crnl_ledger/crnl_ledger.did &&
echo "Extracted crnl_ledger.did" &&
candid-extractor target/wasm32-unknown-unknown/release/chronolock.wasm > src/backend/chronolock/chronolock.did &&
echo "Extracted chronolock.did" &&
candid-extractor target/wasm32-unknown-unknown/release/chainkey_testing_canister.wasm > src/backend/chainkey_testing_canister/chainkey_testing_canister.did &&
echo "Extracted chainkey_testing_canister.did" &&
dfx generate chainkey_testing_canister && dfx generate crnl_ledger && dfx generate chronolock &&
dfx deploy