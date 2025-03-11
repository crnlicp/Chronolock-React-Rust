cargo install candid-extractor &&
dfx deploy &&
candid-extractor target/wasm32-unknown-unknown/release/hello_world.wasm > src/backend/hello_world/hello_world.did &&
echo "Extracted hello_world.did" &&
candid-extractor target/wasm32-unknown-unknown/release/crnl_ledger.wasm > src/backend/crnl_ledger/crnl_ledger.did 
echo "Extracted crnl_ledger.did"