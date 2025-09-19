#!/bin/bash

dfx stop && dfx start --clean --background &&
bash manage_identities.sh &&
cargo install candid-extractor &&
cargo clean && 
dfx canister create --all &&
rustup target add wasm32-unknown-unknown &&
bash generate_init_args.sh &&
cargo build --target wasm32-unknown-unknown --release &&
candid-extractor target/wasm32-unknown-unknown/release/crnl_ledger_canister.wasm > src/backend/crnl_ledger_canister/crnl_ledger_canister.did &&
echo "Extracted crnl_ledger_canister.did" &&
candid-extractor target/wasm32-unknown-unknown/release/chronolock_canister.wasm > src/backend/chronolock_canister/chronolock_canister.did &&
echo "Extracted chronolock_canister.did" &&
candid-extractor target/wasm32-unknown-unknown/release/chainkey_testing_canister.wasm > src/backend/chainkey_testing_canister/chainkey_testing_canister.did &&
echo "Extracted chainkey_testing_canister.did" &&
dfx generate chainkey_testing_canister && dfx generate crnl_ledger_canister && dfx generate chronolock_canister &&
dfx deploy