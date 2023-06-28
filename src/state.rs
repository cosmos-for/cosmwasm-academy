use cosmwasm_std::Coin;
use cw_storage_plus::Item;

pub const COUNTER: Item<u64> = Item::new("counter");
pub const DONATION: Item<Coin> = Item::new("donation");
