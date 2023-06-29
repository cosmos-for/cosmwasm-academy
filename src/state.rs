use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::Item;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct State {
    pub counter: u64,
    pub minimal_donation: Coin,
}

impl State {
    pub fn new(counter: u64, minimal_donation: Coin) -> Self {
        Self {
            counter,
            minimal_donation,
        }
    }
}

pub const STATE: Item<State> = Item::new("state");
pub const OWNER: Item<Addr> = Item::new("owner");
