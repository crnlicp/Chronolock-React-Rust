use candid::Nat;
use evm_rpc_canister_types::{
    Block, EthMainnetService, GetBlockByNumberResult, MultiGetBlockByNumberResult, RpcServices,
    EVM_RPC,
};
use ic_cdk::api::management_canister::{
    ecdsa::{
        ecdsa_public_key, sign_with_ecdsa, EcdsaCurve, EcdsaKeyId, EcdsaPublicKeyArgument,
        SignWithEcdsaArgument,
    },
    schnorr::{
        schnorr_public_key, sign_with_schnorr, SchnorrAlgorithm, SchnorrKeyId,
        SchnorrPublicKeyArgument, SignWithSchnorrArgument,
    },
};

#[ic_cdk::update]
async fn get_evm_block(height: u128) -> Block {
    // Ethereum RPC providers
    let rpc_providers = RpcServices::EthMainnet(Some(vec![
        EthMainnetService::Cloudflare,
        EthMainnetService::Alchemy,
        EthMainnetService::Llama,
    ]));

    // Base RPC providers
    // Get chain ID from https://chainlist.org/
    // let rpc_providers = RpcServices::Custom {
    //     chainId: 8453,
    //     services: vec![
    //         evm_rpc_canister_types::RpcApi {
    //             url: "https://base.llamarpc.com".to_string(),
    //             headers: None,
    //         },
    //         evm_rpc_canister_types::RpcApi {
    //             url: "https://base-rpc.publicnode.com".to_string(),
    //             headers: None,
    //         },
    //     ],
    // };

    let cycles = 10_000_000_000; // Rest gets refunded
    let (result,) = EVM_RPC
        .eth_get_block_by_number(
            rpc_providers,
            None,
            evm_rpc_canister_types::BlockTag::Number(Nat::from(height)),
            cycles,
        )
        .await
        .expect("Failed to call RPC canister");

    match result {
        MultiGetBlockByNumberResult::Consistent(r) => match r {
            GetBlockByNumberResult::Ok(block) => block,
            GetBlockByNumberResult::Err(err) => panic!("{err:?}"),
        },
        MultiGetBlockByNumberResult::Inconsistent(v) => {
            panic!("RPC providers gave inconsistent results: {:?}", v)
        }
    }
}

fn sha256(input: &String) -> [u8; 32] {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(input.as_bytes());
    hasher.finalize().into()
}

#[ic_cdk::update]
async fn get_ecdsa_public_key() -> String {
    let (pub_key,) = ecdsa_public_key(EcdsaPublicKeyArgument {
        key_id: key_id_ecdsa(),
        ..Default::default()
    })
    .await
    .expect("Failed to get ecdsa public key");
    hex::encode(pub_key.public_key)
}

#[ic_cdk::update]
async fn get_schnorr_public_key() -> String {
    let (pub_key,) = schnorr_public_key(SchnorrPublicKeyArgument {
        key_id: key_id_schnorr(),
        ..Default::default()
    })
    .await
    .expect("Failed to get schnorr public key");
    hex::encode(pub_key.public_key)
}

#[ic_cdk::update]
async fn sign_message_with_ecdsa(message: String) -> String {
    let message_hash = sha256(&message);
    let (signature,) = sign_with_ecdsa(SignWithEcdsaArgument {
        message_hash: message_hash.to_vec(),
        key_id: key_id_ecdsa(),
        ..Default::default()
    })
    .await
    .expect("Failed to sign");
    hex::encode(signature.signature)
}

#[ic_cdk::update]
async fn sign_message_with_schnorr(message: String) -> String {
    let message = sha256(&message);
    let (signature,) = sign_with_schnorr(SignWithSchnorrArgument {
        message: message.to_vec(),
        key_id: key_id_schnorr(),
        ..Default::default()
    })
    .await
    .expect("Failed to sign");
    hex::encode(signature.signature)
}

fn key_id_ecdsa() -> EcdsaKeyId {
    EcdsaKeyId {
        curve: EcdsaCurve::Secp256k1,
        name: "test_key_1".to_string(), // Use "key_1" for production
    }
}

fn key_id_schnorr() -> SchnorrKeyId {
    SchnorrKeyId {
        algorithm: SchnorrAlgorithm::Ed25519,
        name: "test_key_1".to_string(), // Use "key_1" for production
    }
}
