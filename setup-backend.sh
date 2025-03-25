#!/bin/bash

dfx stop && dfx start --clean --background &&
bash manage_identities.sh &&
cargo install candid-extractor &&
dfx canister create --all &&
bash generate_init_args.sh &&
candid-extractor target/wasm32-unknown-unknown/release/hello_world.wasm > src/backend/hello_world/hello_world.did &&
echo "Extracted hello_world.did" &&
candid-extractor target/wasm32-unknown-unknown/release/crnl_ledger.wasm > src/backend/crnl_ledger/crnl_ledger.did &&
echo "Extracted crnl_ledger.did" &&
candid-extractor target/wasm32-unknown-unknown/release/chronolock.wasm > src/backend/chronolock/chronolock.did &&
echo "Extracted chronolock.did" &&
candid-extractor target/wasm32-unknown-unknown/release/vetkd_mock.wasm > src/backend/vetkd_mock/vetkd_mock.did &&
echo "Extracted vetkd_mock.did" &&
dfx deploy