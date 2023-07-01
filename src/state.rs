use cosmwasm_std::{Addr, Coin, Decimal};
use cw_storage_plus::Item;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub counter: u64,
    pub minimal_donation: Coin,
    pub owner: Addr,
    pub donating_parent: Option<u64>,
}

impl State {
    pub fn new(
        counter: u64,
        minimal_donation: Coin,
        owner: Addr,
        donating_parent: Option<u64>,
    ) -> Self {
        Self {
            counter,
            minimal_donation,
            owner,
            donating_parent,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ParentDonation {
    pub address: Addr,
    pub donating_parent_period: u64,
    pub part: Decimal,
}

impl ParentDonation {
    pub fn new(address: Addr, donating_parent_period: u64, part: Decimal) -> Self {
        Self {
            address,
            donating_parent_period,
            part,
        }
    }
}

pub const STATE: Item<State> = Item::new("state");
pub const PARENT_DONATION: Item<ParentDonation> = Item::new("parent_donation");
