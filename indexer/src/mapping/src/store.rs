use candid::CandidType;
use candid::Deserialize;
use candid::Nat;
use common::types::Balance;
use common::types::TransferEvent;
use ic_stable_memory::collections::SBTreeMap;
use ic_stable_memory::collections::SVec;
use ic_stable_memory::SBox;
use ic_web3_rs::types::Address;
use std::cell::RefCell;
use std::str::FromStr;

#[derive(Clone, Debug, CandidType, Deserialize)]
struct Subscriber {
    topic: String,
}

const NULL_ADDRESS: &str = "0x0000000000000000000000000000000000000000";
type BalanceStore = SBTreeMap<Balance, SVec<SBox<String>>>;
type AccountBalancesStore = SBTreeMap<SBox<String>, SBox<Balance>>;

thread_local! {
    static ACCOUNT_BALANCES: RefCell<AccountBalancesStore> = RefCell::new(SBTreeMap::new());
    static BALANCES: RefCell<BalanceStore> = RefCell::new(SBTreeMap::new());
    static TOTAL_SUPPLY: RefCell<Balance> = RefCell::new(Nat::from(0));
    static KNOWN_BLOCKS: RefCell<Vec<u64>> = RefCell::new(Vec::new());
}

pub fn get_account_balance(account: String) -> Balance {
    ACCOUNT_BALANCES.with(|f| match f.borrow().get(&account) {
        Some(balance) => balance.clone(),
        None => Nat::from(0),
    })
}

fn update_balance(account: String, balance: Balance) {
    ACCOUNT_BALANCES.with(|account_balance| {
        let balance_before = get_account_balance(account.clone());
        account_balance
            .borrow_mut()
            .insert(
                SBox::new(account.clone()).unwrap(),
                SBox::new(balance.clone()).unwrap(),
            )
            .unwrap();
        add_to_balances(account.clone(), balance);
        remove_from_balances(account.clone(), balance_before);
    })
}
fn remove_from_balances(account: String, balance: Balance) {
    BALANCES.with(|balances| {
        let mut balances = balances.borrow_mut();
        balances.get_mut(&balance).map(|mut balance_vec| {
            balance_vec
                .iter()
                .position(|x| x.to_owned().eq(&account))
                .map(|idx| balance_vec.remove(idx));
        });
    });
    rm_balances_elem_if_empty(balance);
}

fn rm_balances_elem_if_empty(balance: Balance) {
    BALANCES.with(|balances| {
        let mut balances = balances.borrow_mut();
        if balances.get(&balance).map(|x| x.len()).unwrap_or(0) == 0 {
            balances.remove(&balance);
        }
    })
}

fn add_to_balances(account: String, balance: Balance) {
    BALANCES.with(|balances| {
        let mut balances = balances.borrow_mut();
        if balances.get(&balance).is_none() {
            let mut new_elem: SVec<SBox<String>> = SVec::new();
            new_elem.push(SBox::new(account).unwrap()).unwrap();
            balances.insert(balance, new_elem).unwrap();
        } else {
            balances
                .get_mut(&balance)
                .unwrap()
                .push(SBox::new(account).unwrap())
                .unwrap();
        }
    })
}
pub fn setup() {
    ic_stable_memory::stable_memory_init();
}
pub fn known_block(block_number: u64) -> bool {
    KNOWN_BLOCKS.with(|f| f.borrow().contains(&block_number))
}

pub fn total_supply() -> Balance {
    TOTAL_SUPPLY.with(|f| f.borrow().to_owned())
}

pub fn balances_top_n(n: u64) -> Vec<(String, Balance)> {
    BALANCES.with(|f| {
        let mut count: u64 = 0;
        let mut result: Vec<(String, Balance)> = Vec::new();
        for accounts in f.borrow().iter().rev() {
            count += accounts.1.len() as u64;
            for account in accounts.1.iter() {
                if count > n {
                    return result;
                }
                result.push((account.to_owned(), accounts.0.to_owned()));
            }
        }
        result
    })
}

pub fn update(event: TransferEvent) {
    let from: String = event.from.clone();
    let to = event.to.clone();

    let value = event.value;
    if is_null_address(&from) {
        handle_mint(&value.clone())
    } else {
        let from_balance = get_account_balance(from.clone());
        if from_balance.to_owned().ge(&value) {
            update_balance(from.clone(), from_balance - value.clone())
        }
    }
    if is_null_address(&to) {
        handle_burn(&value.clone())
    } else {
        let to_balance = get_account_balance(to.clone());
        update_balance(to.clone(), to_balance + value.clone())
    }
}

fn handle_mint(amount: &Balance) {
    TOTAL_SUPPLY.with(|f| {
        let mut total_balance: std::cell::RefMut<Balance> = f.borrow_mut();
        *total_balance += amount.clone();
    });
}

fn handle_burn(amount: &Balance) {
    TOTAL_SUPPLY.with(|f| {
        let mut total_balance: std::cell::RefMut<Balance> = f.borrow_mut();
        if total_balance.to_owned().ge(amount) {
            *total_balance -= amount.clone();
        }
    });
}

fn is_null_address(address: &String) -> bool {
    Address::from_str(address)
        .unwrap()
        .eq(&Address::from_str(NULL_ADDRESS).unwrap())
}
