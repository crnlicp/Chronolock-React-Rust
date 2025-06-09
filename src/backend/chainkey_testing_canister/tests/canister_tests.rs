// src/backend/chainkey_testing_canister/tests/chainkey_testing_canister.rs

use candid::{Decode, Encode, Principal};
use ic_cdk::api::management_canister::main::CanisterId;
use pocket_ic::PocketIc;

use chainkey_testing_canister::vetkd::VetKDCurve;
use chainkey_testing_canister::vetkd::VetKDDeriveKeyReply;
use chainkey_testing_canister::vetkd::VetKDDeriveKeyRequest;
use chainkey_testing_canister::vetkd::VetKDKeyId;
use chainkey_testing_canister::vetkd::VetKDPublicKeyReply;
use chainkey_testing_canister::vetkd::VetKDPublicKeyRequest;
use ic_vetkd_utils::{DerivedPublicKey, EncryptedVetKey, TransportSecretKey};

pub const CANISTER_WASM: &str =
    "../../../target/wasm32-unknown-unknown/release/chainkey_testing_canister.wasm";

#[test]
fn test_public_key_retrieval() {
    let ctx = TestContext::default();

    let public_key_bytes = ctx
        .canister
        .vetkd_public_key(VetKDPublicKeyRequest {
            canister_id: None,
            context: ctx.context.clone(),
            key_id: ctx.key_id.clone(),
        })
        .public_key;

    assert!(
        !public_key_bytes.is_empty(),
        "Public key should not be empty"
    );
}

#[test]
fn test_key_derivation() {
    let ctx = TestContext::default();

    // Initialize RNG for the test
    let seed = [101u8; 32];
    let tsk = TransportSecretKey::from_seed(seed.to_vec())
        .expect("failed to create transport secret key");

    let encrypted_key = ctx
        .canister
        .vetkd_derive_key(VetKDDeriveKeyRequest {
            context: ctx.context.clone(),
            input: ctx.input.clone(),
            transport_public_key: tsk.public_key(),
            key_id: ctx.key_id.clone(),
        })
        .encrypted_key;

    assert!(
        !encrypted_key.is_empty(),
        "Encrypted key should not be empty"
    );
}

#[test]
fn test_key_decryption_and_verification() {
    let ctx = TestContext::default();
    let tsk = TransportSecretKey::from_seed([101; 32].to_vec())
        .expect("failed to create transport secret key");

    // Get public key
    let public_key_bytes = ctx
        .canister
        .vetkd_public_key(VetKDPublicKeyRequest {
            canister_id: None,
            context: ctx.context.clone(),
            key_id: ctx.key_id.clone(),
        })
        .public_key;

    // Get encrypted key
    let encrypted_key = ctx
        .canister
        .vetkd_derive_key(VetKDDeriveKeyRequest {
            context: ctx.context.clone(),
            input: ctx.input.clone(),
            transport_public_key: tsk.public_key(),
            key_id: ctx.key_id.clone(),
        })
        .encrypted_key;

    // Test decryption and verification
    let encrypted_vetkey = EncryptedVetKey::deserialize(&encrypted_key)
        .expect("failed to deserialize EncryptedVetKey");
    let derived_public_key = DerivedPublicKey::deserialize(&public_key_bytes)
        .expect("failed to deserialize DerivedPublicKey");
    let vetkey = encrypted_vetkey
        .decrypt_and_verify(&tsk, &derived_public_key, &ctx.input)
        .expect("failed to decrypt and verify vetKey");

    assert!(
        !vetkey.signature_bytes().is_empty(),
        "Decrypted key should not be empty"
    );
}

#[test]
fn test_key_consistency_with_different_transport_keys() {
    let ctx = TestContext::default();

    // Get public key once for both operations
    let public_key_bytes = ctx
        .canister
        .vetkd_public_key(VetKDPublicKeyRequest {
            canister_id: None,
            context: ctx.context.clone(),
            key_id: ctx.key_id.clone(),
        })
        .public_key;

    // First transport key
    let tsk_1 = TransportSecretKey::from_seed([101; 32].to_vec())
        .expect("failed to create transport secret key");
    let key_1 = derive_and_decrypt_key(&ctx, &tsk_1, &public_key_bytes);

    // Second transport key
    let tsk_2 = TransportSecretKey::from_seed([102; 32].to_vec())
        .expect("failed to create transport secret key");
    let key_2 = derive_and_decrypt_key(&ctx, &tsk_2, &public_key_bytes);

    assert_eq!(key_1, key_2, "Derived keys should be identical");
}

fn derive_and_decrypt_key(
    ctx: &TestContext,
    tsk: &TransportSecretKey,
    public_key_bytes: &[u8],
) -> Vec<u8> {
    let encrypted_key = ctx
        .canister
        .vetkd_derive_key(VetKDDeriveKeyRequest {
            context: ctx.context.clone(),
            input: ctx.input.clone(),
            transport_public_key: tsk.public_key(),
            key_id: ctx.key_id.clone(),
        })
        .encrypted_key;

    let encrypted_vetkey = EncryptedVetKey::deserialize(&encrypted_key)
        .expect("failed to deserialize EncryptedVetKey");
    let derived_public_key = DerivedPublicKey::deserialize(public_key_bytes)
        .expect("failed to deserialize DerivedPublicKey");
    let vetkey = encrypted_vetkey
        .decrypt_and_verify(tsk, &derived_public_key, &ctx.input)
        .expect("failed to decrypt and verify vetKey");

    vetkey.signature_bytes().to_vec()
}

pub struct CanisterSetup {
    env: PocketIc,
    canister_id: CanisterId,
}

struct TestContext {
    canister: CanisterSetup,
    context: Vec<u8>,
    key_id: VetKDKeyId,
    input: Vec<u8>,
}

impl Default for TestContext {
    fn default() -> Self {
        let canister = CanisterSetup::default();
        Self {
            canister,
            context: b"test-context".to_vec(),
            key_id: VetKDKeyId {
                curve: VetKDCurve::Bls12_381_G2,
                name: "insecure_test_key_1".to_string(),
            },
            input: b"test-input".to_vec(),
        }
    }
}

impl CanisterSetup {
    pub fn new() -> Self {
        let env = PocketIc::new();
        let canister_id = env.create_canister();
        env.add_cycles(canister_id, u128::MAX);
        let wasm_bytes = std::fs::read(CANISTER_WASM).expect("Failed to read WASM file");
        env.install_canister(canister_id, wasm_bytes, vec![], None);
        Self { env, canister_id }
    }

    pub fn vetkd_public_key(&self, args: VetKDPublicKeyRequest) -> VetKDPublicKeyReply {
        let method = "vetkd_public_key";
        let result = self.env.update_call(
            self.canister_id,
            Principal::anonymous(),
            method,
            Encode!(&args).expect("failed to encode args"),
        );
        match result {
            Ok(bytes) => {
                Decode!(&bytes, VetKDPublicKeyReply).expect("failed to decode {method} result")
            }
            Err(user_error) => panic!("{method} user error: {user_error}"),
        }
    }

    pub fn vetkd_derive_key(&self, args: VetKDDeriveKeyRequest) -> VetKDDeriveKeyReply {
        let method = "vetkd_derive_key";
        let result = self.env.update_call(
            self.canister_id,
            Principal::anonymous(),
            method,
            Encode!(&args).expect("failed to encode args"),
        );
        match result {
            Ok(bytes) => {
                Decode!(&bytes, VetKDDeriveKeyReply).expect("failed to decode {method} result")
            }
            Err(user_error) => panic!("{method} user error: {user_error}"),
        }
    }
}

impl Default for CanisterSetup {
    fn default() -> Self {
        Self::new()
    }
}
