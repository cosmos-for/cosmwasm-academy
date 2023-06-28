use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::Item;

pub const COUNTER: Item<u64> = Item::new("counter");
pub const DONATION: Item<Coin> = Item::new("donation");
pub const OWNER: Item<Addr> = Item::new("owner");
