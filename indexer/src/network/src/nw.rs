use std::collections::HashMap;

use candid::{CandidType, Deserialize};
use ic_stable_memory::derive::{CandidAsDynSizeBytes, StableType};
use lazy_static::lazy_static;

#[derive(
    CandidType,
    Debug,
    Clone,
    PartialEq,
    PartialOrd,
    Deserialize,
    StableType,
    CandidAsDynSizeBytes,
    Default,
    Eq,
    Hash,
    Copy,
    Ord,
)]
pub enum SupportedNetwork {
    #[default]
    Mainnet,
    Optimism,
    PolygonMumbai,
}
#[derive(
    CandidType,
    Debug,
    Clone,
    PartialEq,
    PartialOrd,
    Deserialize,
    StableType,
    CandidAsDynSizeBytes,
    Default,
    Eq,
    Hash,
)]
pub struct NetworkInfo {
    pub name: String,
    pub chain_id: u32,
    pub network: SupportedNetwork,
    pub rpc_url: String,
    pub key_name: String,
}

impl SupportedNetwork {
    pub fn from(chain_id: u32) -> Self {
        match chain_id {
            1 => SupportedNetwork::Mainnet,
            _ => panic!("Unsupported chain id {}", chain_id),
        }
    }
}

impl NetworkInfo {
    pub fn get_network_info(network: SupportedNetwork) -> NetworkInfo {
        NETWORKS.get(&network).unwrap().clone()
    }
}
lazy_static! {
    pub static ref NETWORKS: HashMap<SupportedNetwork, NetworkInfo> = {
        let mut map = HashMap::new();
        map.insert(
            SupportedNetwork::Mainnet,
            NetworkInfo {
                name: "Mainnet".to_string(),
                chain_id: 1,
                network: SupportedNetwork::Mainnet,
                rpc_url: "https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY".to_string(),
                key_name: "test_key_1".to_string(),
            },
        );
        map.insert(
            SupportedNetwork::Optimism,
            NetworkInfo {
                name: "Optimism".to_string(),
                chain_id: 10,
                network: SupportedNetwork::Optimism,
                rpc_url: "https://opt-mainnet.g.alchemy.com/v2/YOUR_KEY".to_string(),
                key_name: "test_key_1".to_string(),
            },
        );
        map.insert(
            SupportedNetwork::PolygonMumbai,
            NetworkInfo {
                name: "PolygonMumbai".to_string(),
                chain_id: 80001,
                network: SupportedNetwork::PolygonMumbai,
                rpc_url: "https://rpc-mumbai.maticvigil.com".to_string(),
                key_name: "test_key_1".to_string(),
            },
        );

        map
    };
}
pub struct EcdsaKeyEnvs {
    pub network: SupportedNetwork,
}
impl EcdsaKeyEnvs {
    pub fn to_key_name(self) -> String {
        NETWORKS.get(&self.network).unwrap().key_name.clone()
    }
}
