use ic_solidity_bindgen::Web3Context;
use ic_web3_rs::ethabi::Address;
pub mod nw;
use nw::{NetworkInfo, SupportedNetwork};

pub fn ctx(network: SupportedNetwork) -> Result<Web3Context, ic_web3_rs::error::Error> {
    let network_info = NetworkInfo::get_network_info(network);
    Web3Context::new(
        &network_info.rpc_url,
        Address::from_low_u64_be(0),
        u64::from(network_info.chain_id),
        network_info.key_name,
    )
}
