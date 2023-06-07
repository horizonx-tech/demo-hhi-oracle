use candid::{candid_method, Nat, Principal};
use chainsight_generate::{did_export, manageable};
use ic_cdk::{api::call::CallResult, query, update};
use instant::Duration;
use std::{cell::RefCell, collections::HashMap};
manageable!();

thread_local! {
    static HHI_CANISTER: RefCell<Principal> = RefCell::new(Principal::anonymous());
    static MAPPER: RefCell<String> = RefCell::new(String::default());
    static HHI_DATA_STORE: RefCell<HashMap<u64, Nat>> = RefCell::new(HashMap::new());
}

#[query]
#[candid_method(query)]
fn hhis(size: usize) -> HashMap<u64, Nat> {
    HHI_DATA_STORE.with(|hhi_data_store| {
        let mut hhis = HashMap::new();
        let data = hhi_data_store.borrow_mut();
        let mut keys: Vec<u64> = data.keys().cloned().collect();
        keys.sort();
        keys.reverse();
        for key in keys.iter().take(size) {
            hhis.insert(*key, data.get(key).unwrap().clone());
        }
        hhis
    })
}

#[query]
#[candid_method(query)]
fn data_points() -> u64 {
    HHI_DATA_STORE.with(|hhi_data_store| hhi_data_store.borrow().len() as u64)
}

#[update]
#[candid_method(update)]
async fn setup(hhi_canister_id: String, mapper_canister_id: String) {
    HHI_CANISTER.with(|hhi_canister| {
        *hhi_canister.borrow_mut() = Principal::from_text(hhi_canister_id).unwrap();
    });
    MAPPER.with(|mapper| {
        *mapper.borrow_mut() = mapper_canister_id;
    });
    ic_cdk_timers::set_timer_interval(Duration::from_secs(60 * 60), || {
        ic_cdk::spawn(async {
            let principal = HHI_CANISTER.with(|hhi_canister| *hhi_canister.borrow());
            let mapper = MAPPER.with(|mapper| mapper.borrow().clone());
            let result: CallResult<(Nat,)> =
                ic_cdk::api::call::call(principal, "hhi_of_top_n", (mapper, 100_u64)).await;
            match result {
                Ok(result) => {
                    HHI_DATA_STORE.with(|hhi_data_store| {
                        hhi_data_store
                            .borrow_mut()
                            .insert(ic_cdk::api::time(), result.0);
                    });
                }
                Err(e) => {
                    ic_cdk::println!("error calling subscriber: {:?}", e);
                }
            }
        })
    });
}

thread_local! {
    static HHI_STORE: std::cell::RefCell<HashMap<u64,Nat>> = std::cell::RefCell::new(HashMap::new());
}
did_export!("hhi_snapshot");
