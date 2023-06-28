use cosmwasm_std::Coin;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    #[serde(default)]
    pub init: u64,
    pub minimal_donation: Coin,
}

impl InstantiateMsg {
    pub fn new(init: u64, minimal_donation: Coin) -> Self {
        Self {
            init,
            minimal_donation,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg2 {
    pub init: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
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

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
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
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct IncrementResp {
    pub value: u64,
}

impl IncrementResp {
    pub fn new(value: u64) -> Self {
        Self { value }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct DonateResp {
    pub value: u64,
}

impl DonateResp {
    pub fn new(value: u64) -> Self {
        Self { value }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Value {},
    Incremented { value: u64 },
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct ValueResp {
    pub value: u64,
}
