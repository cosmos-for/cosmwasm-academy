mod contract;

pub mod msg;

mod state;

use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use msg::{InstantiateMsg, ExecMsg};

use crate::contract::query;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    contract::instantiate(deps, msg.init)
}

#[entry_point]
pub fn execute(deps: DepsMut, _env: Env, info: MessageInfo, msg: ExecMsg) -> StdResult<Response> {
    use ExecMsg::*;

    match msg {
        Incremented { value } => contract::exec::increment(deps, value, info.sender.as_str())
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: msg::QueryMsg) -> StdResult<Binary> {
    use msg::QueryMsg::*;

    match msg {
        Value {} => to_binary(&query::value(deps)?),
        Incremented { value } => to_binary(&query::increment(value)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use cosmwasm_std::{Addr, Coin, Empty};
    use cw_multi_test::{App, Contract, ContractWrapper, Executor};

    use crate::msg::{InstantiateMsg2, QueryMsg, ValueResp, IncrementResp};

    fn counting_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(execute, instantiate, query);
        Box::new(contract)
    }

    fn sender() -> Addr {
        Addr::unchecked("sei18rszd3tmgpjvjwq2qajtmn5jqvtscd2yuygl4z")
    }

    const LABEL: &str = "counting-contract";
    const SEND_FUNDS: &[Coin] = &[];
    const ADMIN: Option<String> = None;

    #[test]
    fn query_value() {
        let mut app = App::default();

        let contract_id = app.store_code(counting_contract());
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                sender(),
                &InstantiateMsg::new(10),
                SEND_FUNDS,
                LABEL,
                ADMIN,
            )
            .unwrap();

        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &msg::QueryMsg::Value {})
            .unwrap();

        assert_eq!(resp.value, 10);
    }

    #[test]
    fn query_increment() {
        let mut app = App::default();
        let contract_id = app.store_code(counting_contract());
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                sender(),
                &InstantiateMsg2 { init: 20 },
                SEND_FUNDS,
                LABEL,
                ADMIN,
            )
            .unwrap();

        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Incremented { value: 3 })
            .unwrap();

        assert_eq!(resp.value, 4);
    }

    #[test]
    fn execute_increment() {
        let mut app = App::default();
        let contract_id = app.store_code(counting_contract());
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                sender(),
                &InstantiateMsg2 { init: 20 },
                SEND_FUNDS,
                LABEL,
                ADMIN,
            )
            .unwrap();
        let resp = app 
            .execute_contract(sender(), contract_addr.clone(), &ExecMsg::Incremented { value: 10 }, SEND_FUNDS)
            .unwrap();

        assert_eq!(
            resp.data,
            Some(to_binary(&IncrementResp::new(30)).unwrap())
        );

        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Value {  })
            .unwrap();

        assert_eq!(resp.value, 30);
    }
}
