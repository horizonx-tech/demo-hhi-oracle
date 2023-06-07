use ic_web3_rs::{
    ethabi::Address,
    ic::{get_public_key, pubkey_to_address},
};
use network::nw::{EcdsaKeyEnvs, SupportedNetwork};

async fn public_key() -> Result<Vec<u8>, String> {
    get_public_key(
        None,
        vec![default_derivation_key()],
        EcdsaKeyEnvs {
            network: SupportedNetwork::Mainnet,
        }
        .to_key_name(),
    )
    .await
}

fn to_ethereum_address(pub_key: Vec<u8>) -> Result<Address, String> {
    pubkey_to_address(&pub_key)
}

pub async fn ethereum_address() -> Result<Address, String> {
    let pub_key = public_key().await?;
    to_ethereum_address(pub_key)
}

pub fn default_derivation_key() -> Vec<u8> {
    ic_cdk::id().as_slice().to_vec()
}
