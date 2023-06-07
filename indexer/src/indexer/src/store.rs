use candid::CandidType;
use candid::Deserialize;
use common::indexing::IndexingConfig;
use common::types::TransferEvent;
use ic_stable_memory::collections::SBTreeMap;
use ic_stable_memory::collections::SVec;

use ic_stable_memory::derive::CandidAsDynSizeBytes;
use ic_stable_memory::derive::StableType;

use ic_stable_memory::SBox;
use network::nw::SupportedNetwork;
use std::cell::RefCell;

#[derive(
    CandidType, Debug, Clone, PartialEq, PartialOrd, Deserialize, StableType, CandidAsDynSizeBytes,
)]
pub struct Profile {
    pub config: IndexingConfig,
}

type ProfileStore = SBox<Profile>;

#[derive(
    CandidType, Debug, Clone, PartialEq, PartialOrd, Deserialize, StableType, CandidAsDynSizeBytes,
)]
struct SavedBlock {
    number: u64,
}

type SavedBlockStore = SBox<SavedBlock>;
type Events = SBTreeMap<SBox<u64>, SVec<SBox<TransferEvent>>>;

thread_local! {
    //static  EVENTS_STORE: RefCell<BTreeMap<u64, Vec<TransferEvent>>> = RefCell::new(BTreeMap::new());
    static STATE: RefCell<Events> = RefCell::default();
    static PROFILE: RefCell<Option<ProfileStore>> = RefCell::default();
    static SAVED_BLOCK_STORE: RefCell<Option<SavedBlockStore>> = RefCell::default();
    static ERROR_STORE: RefCell<Vec<String>> = RefCell::default();
}

pub fn new_err(msg: &str) {
    ERROR_STORE.with(|f| f.borrow_mut().push(msg.to_string()));
}

pub fn errors_latest_n(n: u64) -> Vec<String> {
    ERROR_STORE.with(|f| {
        let mut errors: Vec<String> = Vec::new();
        let mut count: u64 = 0;
        for error in f.borrow().iter().rev() {
            errors.push(error.to_owned());
            count += 1;
            if count == n {
                break;
            }
        }
        errors
    })
}

pub fn network() -> SupportedNetwork {
    PROFILE.with(|f| f.borrow().as_ref().unwrap().config.config.network)
}

pub fn block_number_at_deploy() -> u64 {
    PROFILE.with(|f| f.borrow().as_ref().unwrap().config.batch_sync_start_from)
}

pub fn events_count() -> usize {
    STATE.with(|f| {
        let mut count: usize = 0;
        for state in f.borrow().iter() {
            count += state.1.len();
        }
        count
    })
}
pub fn events_latest_n(n: usize) -> Vec<TransferEvent> {
    let mut events: Vec<TransferEvent> = Vec::new();
    STATE.with(|f| {
        let mut count: usize = 0;
        for state in f.borrow().iter().rev() {
            for event in state.1.iter() {
                events.push(event.to_owned());
                count += 1;
                if count == n {
                    return;
                }
            }
        }
    });
    events
}

pub fn saved_block() -> u64 {
    SAVED_BLOCK_STORE.with(|f| f.borrow().as_ref().map(|v| v.number).unwrap_or_default())
}

pub fn get_events_by_block_number(block_number: u64) -> Vec<TransferEvent> {
    STATE.with(|f| {
        f.borrow()
            .get(&block_number)
            .map(|v| {
                return v
                    .iter()
                    .map(|e| e.to_owned())
                    .collect::<Vec<TransferEvent>>();
            })
            .unwrap_or_default()
    })
}

pub fn update_saved_block(block_number: u64) {
    SAVED_BLOCK_STORE.with(|f| {
        *f.borrow_mut() = Some(
            SBox::new(SavedBlock {
                number: block_number,
            })
            .unwrap(),
        );
    });
}

pub fn add_events(block_number: u64, events: Vec<TransferEvent>) {
    STATE.with(|f| {
        let mut state = f.borrow_mut();
        let mut events = events;
        let mut events_boxed: SVec<SBox<TransferEvent>> = SVec::new();
        for event in events.iter_mut() {
            events_boxed
                .push(SBox::new(event.to_owned()).unwrap())
                .unwrap();
        }
        state
            .insert(SBox::new(block_number).unwrap(), events_boxed)
            .unwrap();
    });
}

pub fn setup(p: Profile) {
    ic_stable_memory::stable_memory_init();
    PROFILE.with(|f| {
        *f.borrow_mut() = Some(SBox::new(p.clone()).unwrap());
    });
    SAVED_BLOCK_STORE.with(|f| {
        *f.borrow_mut() = Some(
            SBox::new(SavedBlock {
                number: p.config.deployed_block(),
            })
            .unwrap(),
        );
    });
}

pub fn address() -> String {
    PROFILE.with(|f| {
        let p = f.borrow();
        p.as_ref().unwrap().config.address().to_string()
    })
}
