// src/backend/chainkey_testing_canister/src/vetkd.rs

use super::with_rng;
use crate::ensure_call_is_paid;
use crate::inc_call_count;
use candid::CandidType;
use candid::Deserialize;
use candid::Principal;
use ic_cdk::update;
use ic_crypto_internal_bls12_381_type::{G2Affine, Scalar};
use ic_crypto_internal_bls12_381_vetkd::{
    DerivationContext, DerivedPublicKey, EncryptedKey, EncryptedKeyShare, TransportPublicKey,
    TransportPublicKeyDeserializationError,
};

pub type CanisterId = Principal;

#[derive(CandidType, Clone, Deserialize, Eq, PartialEq)]
pub enum VetKDCurve {
    #[serde(rename = "bls12_381_g2")]
    #[allow(non_camel_case_types)]
    Bls12_381_G2,
}

#[derive(CandidType, Clone, Deserialize, Eq, PartialEq)]
pub struct VetKDKeyId {
    pub curve: VetKDCurve,
    pub name: String,
}

#[derive(CandidType, Deserialize)]
pub struct VetKDPublicKeyRequest {
    pub canister_id: Option<CanisterId>,
    pub context: Vec<u8>,
    pub key_id: VetKDKeyId,
}

#[derive(CandidType, Deserialize)]
pub struct VetKDPublicKeyReply {
    pub public_key: Vec<u8>,
}

#[derive(CandidType, Deserialize)]
pub struct VetKDDeriveKeyRequest {
    pub input: Vec<u8>,
    pub context: Vec<u8>,
    pub transport_public_key: Vec<u8>,
    pub key_id: VetKDKeyId,
}

#[derive(CandidType, Deserialize)]
pub struct VetKDDeriveKeyReply {
    pub encrypted_key: Vec<u8>,
}

/// DISCLAIMER: This canister here provides an *unsafe* example implementation
/// of a [proposed](https://github.com/dfinity/interface-spec/pull/158) vetKD
/// system API for demonstration purposes. Because of this, in the following
/// we hard-code a randomly generated master secret key. In case vetKD will be
/// integrated into the Internet Computer protocol, then such a key would be
/// created in a secure manner with distributed key generation so that the key
/// never exists in combined form anywyere and nodes can use it only collectively.
const MASTER_SK_HEX: &str = "718c36cd1dcf3501fd04bbe24c3bb9eedfd066d2420e794dd9342cf71d04176f";

lazy_static::lazy_static! {
    static ref MASTER_SK: Scalar = Scalar::deserialize(
        &hex::decode(MASTER_SK_HEX).expect("failed to hex-decode")
    ).expect("failed to deserialize Scalar");
    static ref MASTER_PK: G2Affine = G2Affine::from(G2Affine::generator() * &*MASTER_SK);
}

#[update]
async fn vetkd_public_key(request: VetKDPublicKeyRequest) -> VetKDPublicKeyReply {
    inc_call_count("vetkd_public_key".to_string());
    ensure_bls12_381_g2_insecure_test_key_1(&request.key_id);

    let is_production: bool = option_env!("IS_PRODUCTION").is_some();

    if is_production {
        // Call management canister to get production key
        let mgmt_canister = Principal::management_canister();
        let (pk_result,): (Result<Vec<u8>, String>,) =
            match ic_cdk::call(mgmt_canister, "vetkd_public_key", (request,)).await {
                Ok(res) => res,
                Err(_e) => {
                    return VetKDPublicKeyReply {
                        public_key: vec![], // Return empty key on error
                    };
                }
            };

        VetKDPublicKeyReply {
            public_key: pk_result.unwrap_or_default(),
        }
    } else {
        // Use test key for development
        let context = {
            let canister_id = request.canister_id.unwrap_or_else(ic_cdk::caller);
            DerivationContext::new(canister_id.as_slice(), &request.context)
        };

        let derived_public_key = DerivedPublicKey::derive_sub_key(&MASTER_PK, &context);
        VetKDPublicKeyReply {
            public_key: derived_public_key.serialize().to_vec(),
        }
    }
}

#[update]
async fn vetkd_derive_key(request: VetKDDeriveKeyRequest) -> VetKDDeriveKeyReply {
    inc_call_count("vetkd_derive_key".to_string());
    ensure_call_is_paid(0);
    ensure_transport_public_key_is_48_bytes(&request.transport_public_key);
    ensure_bls12_381_g2_insecure_test_key_1(&request.key_id);

    let is_production: bool = option_env!("IS_PRODUCTION").is_some();

    if is_production {
        // Call management canister to derive key in production
        let mgmt_canister = Principal::management_canister();
        let (key_result,): (Result<Vec<u8>, String>,) =
            match ic_cdk::call(mgmt_canister, "vetkd_derive_key", (request,)).await {
                Ok(res) => res,
                Err(_e) => {
                    return VetKDDeriveKeyReply {
                        encrypted_key: vec![], // Return empty key on error
                    };
                }
            };

        VetKDDeriveKeyReply {
            encrypted_key: key_result.unwrap_or_default(),
        }
    } else {
        // Use test key derivation for development
        let context = DerivationContext::new(ic_cdk::caller().as_slice(), &request.context);
        let tpk =
            TransportPublicKey::deserialize(&request.transport_public_key).unwrap_or_else(|e| {
                match e {
                    TransportPublicKeyDeserializationError::InvalidPublicKey => {
                        ic_cdk::trap("invalid transport public key")
                    }
                }
            });

        let eks = with_rng(|rng| {
            EncryptedKeyShare::create(rng, &MASTER_PK, &MASTER_SK, &tpk, &context, &request.input)
        })
        .await;

        let ek = EncryptedKey::combine_all(
            &vec![(0, eks)],
            1,
            &MASTER_PK,
            &tpk,
            &context,
            &request.input,
        )
        .unwrap_or_else(|_e| ic_cdk::trap("bad key share"));

        VetKDDeriveKeyReply {
            encrypted_key: ek.serialize().to_vec(),
        }
    }
}

fn ensure_transport_public_key_is_48_bytes(transport_public_key: &[u8]) {
    if transport_public_key.len() != 48 {
        ic_cdk::trap("Transport public key must be 48 bytes");
    }
}

fn ensure_bls12_381_g2_insecure_test_key_1(key_id: &VetKDKeyId) {
    if key_id.curve != VetKDCurve::Bls12_381_G2 {
        ic_cdk::trap("unsupported key ID curve");
    }
    if key_id.name.as_str() != "insecure_test_key_1" {
        ic_cdk::trap(&format!(
            "unsupported key ID name: expected 'insecure_test_key_1' but got '{}'",
            key_id.name
        ));
    }
}

ic_cdk::export_candid!();
