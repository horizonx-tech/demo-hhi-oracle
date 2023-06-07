use candid::Principal;
use chainsight_generate::{did_export, manageable};
use common::config::{Token, TokenConfig};
use common::indexing::IndexingConfig;
use common::types::TransferEvent;
use core::publisher;
use ic_cdk::api::management_canister::http_request::{HttpResponse, TransformArgs};
use ic_cdk::export::candid::candid_method;
use ic_cdk::update;
use ic_cdk_macros::query;
use ic_solidity_bindgen::contract_abis;
use ic_web3_rs::ethabi::Address;
use ic_web3_rs::transforms::processors::get_filter_changes_processor;
use ic_web3_rs::transforms::transform::TransformProcessor;
use ic_web3_rs::transports::ic_http_client::CallOptions;
use network::ctx;
use network::nw::SupportedNetwork;
use std::cell::RefCell;
use std::collections::HashMap;
use std::str::FromStr;
use store::{update_saved_block, Profile};
pub mod store;
manageable!();
contract_abis!("abis/");

thread_local! {
    static GET_EVENT_CHUNKS: RefCell<u64> = RefCell::new(500);
}

#[query]
#[candid_method(query)]
fn event_chunks() -> u64 {
    GET_EVENT_CHUNKS.with(|v| *v.borrow())
}

#[update]
#[candid_method(update)]
fn set_event_chunks(n: u64) {
    GET_EVENT_CHUNKS.with(|v| *v.borrow_mut() = n);
}

#[query]
#[candid_method(query)]
fn subscribers() -> Vec<Principal> {
    publisher::subscribers()
}

#[update]
#[candid_method(update)]
fn add_subscriber() {
    publisher::add_subscriber(ic_cdk::caller());
}

#[query]
#[candid_method(query)]
fn block_number_at_deploy() -> u64 {
    store::block_number_at_deploy()
}

#[query]
#[candid_method(query)]
fn latest_block_number() -> u64 {
    store::saved_block()
}

#[query(name = "getEventsByBlockNumber")]
#[candid_method(query, rename = "getEventsByBlockNumber")]
fn get_events_by_block_number(block_number: u64) -> Vec<TransferEvent> {
    store::get_events_by_block_number(block_number)
}

#[update]
#[candid_method(update)]
async fn update_events(events: HashMap<u64, Vec<TransferEvent>>) {
    ic_cdk::println!("update events invoked: blocks: {}", events.len());
    events
        .clone()
        .into_iter()
        .for_each(|(k, v)| store::add_events(k, v));
    let data: Vec<TransferEvent> = events.values().flatten().cloned().collect();
    let chunks = data.chunks(800);
    for chunk in chunks {
        publisher::publish(chunk.to_vec()).await;
    }
}

#[query(name = "transform")]
#[candid_method(query, rename = "transform")]
fn transform(response: TransformArgs) -> HttpResponse {
    let res = response.response;
    // remove header
    HttpResponse {
        status: res.status,
        headers: Vec::default(),
        body: res.body,
    }
}

#[query]
#[candid_method(query)]
fn events_latest_n(size: usize) -> Vec<TransferEvent> {
    store::events_latest_n(size)
}

#[query]
#[candid_method(query)]
fn events_count() -> usize {
    store::events_count()
}

fn sync_completed() -> bool {
    latest_block_number().ge(&block_number_at_deploy())
}

#[query]
#[candid_method(query)]
fn transform_events(raw: TransformArgs) -> HttpResponse {
    get_filter_changes_processor().transform(raw)
}

#[update]
#[candid_method(update)]
async fn setup(network: SupportedNetwork, token: Token, latest_block: u64) {
    store::setup(Profile {
        config: IndexingConfig::new(TokenConfig::new(network, token), latest_block),
    });
    let interval = std::time::Duration::from_secs(1 * 60 * 60); // 1 hour
    ic_cdk_timers::set_timer_interval(interval, || {
        if !sync_completed() {
            ic_cdk::spawn(async {
                let result = save_logs().await;
                match result {
                    Ok(_) => {}
                    Err(_) => {}
                }
            })
        }
    });
    ic_cdk::println!("setup done")
}
#[update]
#[candid_method(update)]
async fn save_logs() -> Result<String, String> {
    let saved = latest_block_number();

    //if saved.lt(&block_number_at_deploy()) {
    //    return Ok("".to_string());
    //}
    let next: u64 = saved + event_chunks();
    let result: Result<String, String> = save_logs_from_to(saved, next).await;
    result
}

#[query]
#[candid_method(query)]
async fn errors_latest_n(n: u64) -> Vec<String> {
    store::errors_latest_n(n)
}

async fn save_logs_from_to(from: u64, to: u64) -> Result<String, String> {
    let events: HashMap<u64, Vec<TransferEvent>> = ERC20::new(
        Address::from_str(store::address().as_str()).unwrap(),
        &ctx(store::network()).unwrap(),
    )
    .event_transfer(from, to, CallOptions::default())
    .await
    .unwrap_or_else(|e| {
        store::new_err(e.to_string().as_str());
        HashMap::new()
    })
    .into_iter()
    .map(|(k, v)| (k, v.into_iter().map(TransferEvent::from).collect()))
    .collect();
    events
        .clone()
        .into_iter()
        .for_each(|(k, v)| store::add_events(k, v));
    let data: Vec<TransferEvent> = events.values().flatten().cloned().collect();
    let chunks = data.chunks(800);
    for chunk in chunks {
        publisher::publish(chunk.to_vec()).await;
    }
    update_saved_block(to);

    Ok("ok".to_string())
}

did_export!("indexer");
