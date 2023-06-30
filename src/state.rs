use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::Item;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub counter: u64,
    pub minimal_donation: Coin,
    pub owner: Addr,
}

impl State {
    pub fn new(counter: u64, minimal_donation: Coin, owner: Addr) -> Self {
        Self {
            counter,
            minimal_donation,
            owner,
        }
    }
}

pub const STATE: Item<State> = Item::new("state");
