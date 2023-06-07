mod store;
mod utils;

use std::str::FromStr;

use candid::candid_method;
use chainsight_generate::{did_export, manageable};
use ic_cdk::{
    api::{
        call::CallResult,
        management_canister::http_request::{HttpResponse, TransformArgs},
    },
    query, update,
};
use ic_solidity_bindgen::contract_abis;
use ic_web3_rs::{ethabi::Address, types::U256};
use instant::Duration;
use network::{ctx, nw::SupportedNetwork};
use store::{hhi_canister, mapper, set_hhi_canister, set_mapper, set_network, set_oracle_address};

// parameters
const TASK_INTERVAL_SECS: u64 = 60 * 60;
const TOP_N_FOR_HHI: u64 = 100;

manageable!();
contract_abis!("src/relayer/abi");

#[update]
#[candid_method(update)]
async fn get_ethereum_address() -> String {
    match utils::ethereum_address().await {
        Ok(v) => format!("0x{}", hex::encode(v)),
        Err(msg) => msg,
    }
}

#[query]
#[candid_method(query)]
fn transform(response: TransformArgs) -> HttpResponse {
    let res = response.response;
    // remove headers
    HttpResponse {
        status: res.status,
        headers: Vec::default(),
        body: res.body,
    }
}

#[update]
#[candid_method(update)]
async fn setup(
    hhi_canister_id: String,
    mapper_canister_id: String,
    network: SupportedNetwork,
    oracle_addr: String,
) {
    set_hhi_canister(hhi_canister_id);
    set_mapper(mapper_canister_id);
    set_network(network);
    set_oracle_address(oracle_addr);

    ic_cdk_timers::set_timer_interval(Duration::from_secs(TASK_INTERVAL_SECS), || {
        ic_cdk::spawn(async {
            let result: CallResult<(u128,)> =
                ic_cdk::api::call::call(hhi_canister(), "hhi_of_top_n", (mapper(), TOP_N_FOR_HHI))
                    .await;
            if let Err(msg) = result {
                ic_cdk::println!("error msg by calling hhi_of_top_n: {:?}", msg);
                return;
            }
            match sync_state_internal(result.unwrap().0).await {
                Ok(hash) => ic_cdk::println!("txhash: {:?}", hash),
                Err(msg) => ic_cdk::println!("error msg: {:?}", msg),
            }
        });
    });
}

async fn sync_state_internal(val: u128) -> Result<String, String> {
    match Oracle::new(
        Address::from_str(&store::oracle_address()).unwrap(),
        &ctx(store::get_network()).unwrap(),
    )
    .update_state(U256::from(val), None)
    .await
    {
        Ok(receipt) => Ok(receipt.transaction_hash.to_string()),
        Err(msg) => Err(msg.to_string()),
    }
}
did_export!("relayer");
