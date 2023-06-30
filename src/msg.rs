use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Coin;

#[cw_serde]
pub struct InstantiateMsg {
    #[serde(default)]
    pub counter: u64,
    pub minimal_donation: Coin,
}

impl InstantiateMsg {
    pub fn new(counter: u64, minimal_donation: Coin) -> Self {
        Self {
            counter,
            minimal_donation,
        }
    }
}

#[cw_serde]
pub struct InstantiateMsg2 {
    pub init: u64,
}

#[cw_serde]
pub struct InstantiateResp {
    pub value: u64,
    pub minimal_donation: Coin,
}

impl InstantiateResp {
    pub fn new(value: u64, minimal_donation: Coin) -> Self {
        Self {
            value,
            minimal_donation,
        }
    }
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
