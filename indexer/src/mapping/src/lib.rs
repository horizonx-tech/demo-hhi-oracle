use candid::candid_method;
use candid::CandidType;
use chainsight_generate::did_export;
use chainsight_generate::{manageable, mapping};
use common::types::Balance;
use common::types::TransferEvent;
use ic_cdk::query;
use ic_cdk::update;
pub mod store;
manageable!();
mapping!(TransferEvent);

#[query]
#[candid_method(query)]
fn get_account_balance(account: String) -> Balance {
    store::get_account_balance(account)
}

#[query]
#[candid_method(query)]
fn total_supply() -> Balance {
    store::total_supply()
}
#[update]
#[candid_method(update)]
async fn setup() {
    store::setup();
}
#[query]
#[candid_method(query)]
fn balances_top_n(n: u64) -> Vec<(String, Balance)> {
    store::balances_top_n(n)
}
did_export!("mapping");
