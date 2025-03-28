// vetkd_mock/lib.rs

use candid::{CandidType, Principal};
use hex;
use ic_cdk_macros::{query, update};
use serde::Deserialize;

#[derive(CandidType, Deserialize)]
struct VetkdPublicKeyArgs {
    key_id: VetkdPublicKeyArgsKeyId,
    derivation_path: Vec<Vec<u8>>,
    canister_id: Option<Principal>,
}

#[derive(CandidType, Deserialize)]
struct VetkdPublicKeyArgsKeyId {
    name: String,
    curve: VetkdCurve,
}

#[derive(CandidType, Deserialize)]
enum VetkdCurve {
    Bls12381G2,
}

#[derive(CandidType, Deserialize)]
struct ByteBuf(Vec<u8>);

#[derive(CandidType, Deserialize)]
struct VetkdDeriveEncryptedKeyArgs {
    key_id: VetkdDeriveEncryptedKeyArgsKeyId,
    derivation_path: Vec<Vec<u8>>,
    derivation_id: ByteBuf,
    encryption_public_key: ByteBuf,
}

#[derive(CandidType, Deserialize)]
struct VetkdDeriveEncryptedKeyArgsKeyId {
    name: String,
    curve: VetkdCurve,
}

#[query]
fn vetkd_public_key(args: VetkdPublicKeyArgs) -> Vec<u8> {
    // Return a mock public key as bytes
    let mock_key = format!("mock_public_key_{}", args.key_id.name);
    mock_key.as_bytes().to_vec()
}

#[update]
fn vetkd_derive_encrypted_key(args: VetkdDeriveEncryptedKeyArgs) -> Vec<u8> {
    let derivation_id_hex = hex::encode(&args.derivation_id.0);
    let encryption_pubkey_hex = hex::encode(&args.encryption_public_key.0);
    let mock_key = format!(
        "mock_encrypted_key_{}_{}",
        derivation_id_hex, encryption_pubkey_hex
    );
    // println!("Mock VETKD returning: {}", mock_key);
    mock_key.as_bytes().to_vec()
}

ic_cdk::export_candid!();
