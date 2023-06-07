use candid::{candid_method, Nat, Principal};
use chainsight_generate::{did_export, manageable};
use common::types::Balance;
use ic_cdk::{
    api::call::{call, CallResult, RejectionCode},
    query, update,
};
use std::ops::{Add, Div, Mul};
manageable!();
const BASE: u128 = 1_000_000_000_000_u128;

async fn top_n(mapping: Principal, n: u64) -> Result<Vec<(String, Balance)>, RejectionCode> {
    let result: CallResult<(Vec<(String, Balance)>,)> =
        call(mapping, "balances_top_n", (&n,)).await;
    match result {
        Ok(result) => Ok(result.0),
        Err(e) => {
            ic_cdk::println!("error calling subscriber: {:?}", e);
            Err(e.0)
        }
    }
}

#[update]
#[candid_method(update)]
async fn hhi_of_top_n(mapper: String, n: u64) -> Nat {
    let principal = Principal::from_text(mapper).unwrap();
    let balances = top_n(principal, n).await;
    let amounts: Vec<Nat> = balances.unwrap().iter().map(|v| v.clone().1).collect();
    let total_supply = total_supply(principal).await.unwrap();
    ic_cdk::println!("total_supply: {:?}", total_supply);
    amounts
        .iter()
        .map(|balance| hhi(balance.clone(), total_supply.clone()))
        .fold(Balance::from(0), |acc, balance| acc.add(balance))
        .mul(Balance::from(100))
        .mul(Balance::from(100))
        .div(BASE)
        .div(BASE)
}

async fn total_supply(mapping: Principal) -> Result<Nat, RejectionCode> {
    let result: CallResult<(Nat,)> = call(mapping, "total_supply", ()).await;
    match result {
        Ok(result) => Ok(result.0),
        Err(e) => {
            ic_cdk::println!("error calling subscriber: {:?}", e);
            Err(e.0)
        }
    }
}

fn hhi(balance: Balance, total_amount: Balance) -> Balance {
    let dominance = balance.mul(BASE).div(total_amount);
    dominance.clone().mul(dominance)
}
did_export!("hhi");
