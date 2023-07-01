use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Decimal};

#[cw_serde]
pub struct InstantiateMsg {
    #[serde(default)]
    pub counter: u64,
    pub minimal_donation: Coin,
    pub parent: Option<Parent>,
}

impl InstantiateMsg {
    pub fn new(counter: u64, minimal_donation: Coin, parent: Option<Parent>) -> Self {
        Self {
            counter,
            minimal_donation,
            parent,
        }
    }
}

#[cw_serde]
pub struct Parent {
    pub addr: String,
    pub donating_period: u64,
    pub part: Decimal,
}

#[cw_serde]
pub struct InstantiateResp {
    pub value: u64,
    pub minimal_donation: Coin,
    pub owner: Addr,
}

impl InstantiateResp {
    pub fn new(value: u64, minimal_donation: Coin, owner: Addr) -> Self {
        Self {
            value,
            minimal_donation,
            owner,
        }
    }
}

#[cw_serde]
pub struct MigrateMsg {
    pub parent: Option<Parent>,
}

#[cw_serde]
pub enum ExecMsg {
    Increment {
        value: u64,
    },
    Reset {
        #[serde(default)]
        value: u64,
    },
    Donate {},
    Withdraw {},
    WithdrawTo {
        receiver: String,
        funds: Vec<Coin>,
    },
}

#[cw_serde]
pub struct IncrementResp {
    pub value: u64,
}

impl IncrementResp {
    pub fn new(value: u64) -> Self {
        Self { value }
    }
}

#[cw_serde]
pub struct DonateResp {
    pub value: u64,
}

impl DonateResp {
    pub fn new(value: u64) -> Self {
        Self { value }
    }
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ValueResp)]
    Value {},
}

#[cw_serde]
pub struct ValueResp {
    pub value: u64,
}
