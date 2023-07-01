#[cfg(test)]
mod tests;

use cosmwasm_std::{Addr, Attribute, Coin, Event, StdResult};
use cw_multi_test::{App, AppResponse, ContractWrapper, Executor};

use crate::{
    error::ContractError,
    execute, instantiate, migrate,
    msg::{ExecMsg, InstantiateMsg, MigrateMsg, Parent, QueryMsg, ValueResp},
    query,
};

pub struct CountingContract(Addr);

impl CountingContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn store_code(app: &mut App) -> u64 {
        let contract = ContractWrapper::new(execute, instantiate, query).with_migrate(migrate);
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
        Self::instantiate_with_funds_admin(
            app,
            code_id,
            sender,
            label,
            counter,
            minimal_donation,
            &[],
            None,
            None,
        )
    }

    #[track_caller]
    #[allow(clippy::too_many_arguments)]
    pub fn instantiate_with_funds_admin(
        app: &mut App,
        code_id: u64,
        sender: Addr,
        label: &str,
        counter: impl Into<Option<u64>>,
        minimal_donation: Coin,
        send_funds: &[Coin],
        admin: impl Into<Option<String>>,
        parent: impl Into<Option<Parent>>,
    ) -> StdResult<CountingContract> {
        let counter = counter.into().unwrap_or_default();
        let admin = admin.into();
        let parent = parent.into();

        app.instantiate_contract(
            code_id,
            sender,
            &InstantiateMsg::new(counter, minimal_donation, parent),
            send_funds,
            label,
            admin,
        )
        .map_err(|e| e.downcast().unwrap())
        .map(CountingContract)
    }

    #[track_caller]
    pub fn migrate(
        app: &mut App,
        contract_addr: Addr,
        code_id: u64,
        sender: Addr,
        parent: impl Into<Option<Parent>>,
    ) -> StdResult<Self> {
        let parent = parent.into();
        app.migrate_contract(
            sender,
            contract_addr.clone(),
            &MigrateMsg { parent },
            code_id,
        )
        .map_err(|e| e.downcast().unwrap())
        .map(|_| Self(contract_addr))
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

    #[track_caller]
    pub fn verify_events(events: Vec<Event>, action: &str, sender: &str) -> bool {
        let wasm_event = events.iter().find(|e| e.ty == "wasm").unwrap();

        let b = vec![
            Attribute::new("action", action),
            Attribute::new("sender", sender),
        ];

        b.iter().all(|item| wasm_event.attributes.contains(item))
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

pub fn parent() -> Addr {
    Addr::unchecked("inj1g9v8suckezwx93zypckd4xg03r26h6ejlmsptz")
}

pub fn instantiate_msg() -> InstantiateMsg {
    InstantiateMsg::new(0, ten_atom(), None)
}

pub fn zero_funds_instantiate_msg() -> InstantiateMsg {
    InstantiateMsg::new(0, zero_atom(), None)
}

pub fn ten_atom() -> Coin {
    Coin::new(10, "atom")
}

pub fn zero_atom() -> Coin {
    Coin::new(0, "atom")
}
