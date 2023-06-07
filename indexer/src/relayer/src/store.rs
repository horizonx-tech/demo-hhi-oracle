use ic_cdk::export::Principal;
use ic_cdk_timers::TimerId;
use network::nw::SupportedNetwork;
use std::cell::RefCell;

thread_local! {
    static HHI_CANISTER: RefCell<Principal> = RefCell::new(Principal::anonymous());
    static MAPPER: RefCell<String> = RefCell::new(String::default());
    static ORACLE_ADDRESS: RefCell<String> = RefCell::default();
    static TIMER_ID: RefCell<TimerId> = RefCell::default();
    static NETWORK: RefCell<SupportedNetwork> = RefCell::default();
}

pub fn hhi_canister() -> Principal {
    HHI_CANISTER.with(|hhi_canister| *hhi_canister.borrow())
}
pub fn set_hhi_canister(hhi_canister_id: String) {
    HHI_CANISTER.with(|hhi_canister| {
        *hhi_canister.borrow_mut() = Principal::from_text(hhi_canister_id).unwrap();
    });
}

pub fn mapper() -> String {
    MAPPER.with(|mapper| mapper.borrow().clone())
}
pub fn set_mapper(mapper_canister_id: String) {
    MAPPER.with(|mapper| {
        *mapper.borrow_mut() = mapper_canister_id;
    });
}

pub fn oracle_address() -> String {
    ORACLE_ADDRESS.with(|oracle_address| oracle_address.borrow().clone())
}
pub fn set_oracle_address(oracle_address: String) {
    ORACLE_ADDRESS.with(|oracle_address_| {
        *oracle_address_.borrow_mut() = oracle_address;
    });
}

pub fn set_network(nw: SupportedNetwork) {
    NETWORK.with(|value| {
        *value.borrow_mut() = nw;
    });
}
pub fn get_network() -> SupportedNetwork {
    NETWORK.with(|value| *value.borrow())
}
