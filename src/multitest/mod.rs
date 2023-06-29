mod tests;

use cosmwasm_std::{Addr, Coin, StdResult};
use cw_multi_test::{App, AppResponse, ContractWrapper, Executor};

use crate::{
    error::ContractError,
    execute, instantiate,
    msg::{ExecMsg, InstantiateMsg, QueryMsg, ValueResp},
    query,
};

pub struct CountingContract(Addr);

impl CountingContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn store_code(app: &mut App) -> u64 {
        let contract = ContractWrapper::new(execute, instantiate, query);
        app.store_code(Box::new(contract))
    }

    #[track_caller]
    pub fn instantiate(
        app: &mut App,
        code_id: u64,
        sender: Addr,
        label: &str,
        counter: impl Into<Option<u64>>,
        minimal_donation: Coin,
    ) -> StdResult<CountingContract> {
        Self::instantiate_with_funds(app, code_id, sender, label, counter, minimal_donation, &[])
    }

    #[track_caller]
    pub fn instantiate_with_funds(
        app: &mut App,
        code_id: u64,
        sender: Addr,
        label: &str,
        counter: impl Into<Option<u64>>,
        minimal_donation: Coin,
        send_funds: &[Coin],
    ) -> StdResult<CountingContract> {
        let counter = counter.into().unwrap_or_default();
        app.instantiate_contract(
            code_id,
            sender,
            &InstantiateMsg::new(counter, minimal_donation),
            send_funds,
            label,
            None,
        )
        .map_err(|e| e.downcast().unwrap())
        .map(CountingContract)
    }

    pub fn query_value(&self, app: &App) -> StdResult<ValueResp> {
        app.wrap()
            .query_wasm_smart(self.addr(), &QueryMsg::Value {})
    }

    pub fn query_balance(app: &App, addr: Addr, denation: &str) -> StdResult<Coin> {
        app.wrap().query_balance(addr, denation)
    }

    #[track_caller]
    pub fn donate(
        &self,
        app: &mut App,
        sender: Addr,
        funds: &[Coin],
    ) -> Result<AppResponse, ContractError> {
        app.execute_contract(sender, self.addr(), &ExecMsg::Donate {}, funds)
            .map_err(|e| e.downcast().unwrap())
    }

    #[track_caller]
    pub fn withdraw(&self, app: &mut App, sender: Addr) -> Result<AppResponse, ContractError> {
        app.execute_contract(sender, self.addr(), &ExecMsg::Withdraw {}, &[])
            .map_err(|e| e.downcast().unwrap())
    }

    #[track_caller]
    pub fn withdraw_to(
        &self,
        app: &mut App,
        sender: Addr,
        receiver: String,
        send_funds: Vec<Coin>,
    ) -> Result<AppResponse, ContractError> {
        app.execute_contract(
            sender,
            self.addr(),
            &ExecMsg::WithdrawTo {
                receiver,
                funds: send_funds,
            },
            &[],
        )
        .map_err(|e| e.downcast().unwrap())
    }

    #[track_caller]
    pub fn increment(
        &self,
        app: &mut App,
        sender: Addr,
        value: u64,
    ) -> Result<AppResponse, ContractError> {
        app.execute_contract(sender, self.addr(), &ExecMsg::Increment { value }, &[])
            .map_err(|e| e.downcast().unwrap())
    }

    #[track_caller]
    pub fn reset(
        &self,
        app: &mut App,
        sender: Addr,
        value: u64,
    ) -> Result<AppResponse, ContractError> {
        app.execute_contract(sender, self.addr(), &ExecMsg::Reset { value }, &[])
            .map_err(|e| e.downcast().unwrap())
    }
}

pub fn sender() -> Addr {
    Addr::unchecked("sei18rszd3tmgpjvjwq2qajtmn5jqvtscd2yuygl4z")
}

pub fn other_sender() -> Addr {
    Addr::unchecked("sei1aan9kqywf4rf274cal0hj6eyly6wu0uv7edxy2")
}

pub fn owner() -> Addr {
    Addr::unchecked("sei1zj6fjsc2gkce878ukzg6g9wy8cl8p554dlggxd")
}

pub fn instantiate_msg() -> InstantiateMsg {
    InstantiateMsg::new(0, ten_atom())
}

pub fn zero_funds_instantiate_msg() -> InstantiateMsg {
    InstantiateMsg::new(0, zero_atom())
}

pub fn ten_atom() -> Coin {
    Coin::new(10, "atom")
}

pub fn zero_atom() -> Coin {
    Coin::new(0, "atom")
}
