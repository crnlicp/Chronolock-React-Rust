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
fn should_consistently_derive_vetkey() {
    let canister = CanisterSetup::default();

    let context = b"test-context".to_vec();
    let key_id = VetKDKeyId {
        curve: VetKDCurve::Bls12_381_G2,
        name: "insecure_test_key_1".to_string(),
    };
    let input = b"test-input".to_vec();

    let public_key_bytes = canister
        .vetkd_public_key(VetKDPublicKeyRequest {
            canister_id: None,
            context: context.clone(),
            key_id: key_id.clone(),
        })
        .public_key;

    let tsk_1 = TransportSecretKey::from_seed([101; 32].to_vec())
        .expect("failed to create transport secret key");
    let encrypted_key_1 = canister
        .vetkd_derive_key(VetKDDeriveKeyRequest {
            context: context.clone(),
            input: input.clone(),
            transport_public_key: tsk_1.public_key(),
            key_id: key_id.clone(),
        })
        .encrypted_key;
    let encrypted_vetkey_1 = EncryptedVetKey::deserialize(&encrypted_key_1)
        .expect("failed to deserialize EncryptedVetKey");
    let derived_public_key_1 = DerivedPublicKey::deserialize(&public_key_bytes)
        .expect("failed to deserialize DerivedPublicKey");
    let vetkey_1 = encrypted_vetkey_1
        .decrypt_and_verify(&tsk_1, &derived_public_key_1, &input)
        .expect("failed to decrypt and verify vetKey");
    let decrypted_key_1 = vetkey_1.signature_bytes().to_vec();

    let tsk_2 = TransportSecretKey::from_seed([102; 32].to_vec())
        .expect("failed to create transport secret key");
    let encrypted_key_2 = canister
        .vetkd_derive_key(VetKDDeriveKeyRequest {
            context,
            input: input.clone(),
            transport_public_key: tsk_2.public_key(),
            key_id,
        })
        .encrypted_key;
    let encrypted_vetkey_2 = EncryptedVetKey::deserialize(&encrypted_key_2)
        .expect("failed to deserialize EncryptedVetKey");
    let derived_public_key_2 = DerivedPublicKey::deserialize(&public_key_bytes)
        .expect("failed to deserialize DerivedPublicKey");
    let vetkey_2 = encrypted_vetkey_2
        .decrypt_and_verify(&tsk_2, &derived_public_key_2, &input)
        .expect("failed to decrypt and verify vetKey");
    let decrypted_key_2 = vetkey_2.signature_bytes().to_vec();

    assert_eq!(decrypted_key_1, decrypted_key_2);
}

pub struct CanisterSetup {
    env: PocketIc,
    canister_id: CanisterId,
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
