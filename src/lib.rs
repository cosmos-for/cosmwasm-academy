mod contract;

pub mod msg;

mod state;

use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult,
};
use msg::InstantiateMsg;

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
pub fn execute(_deps: DepsMut, _env: Env, _info: MessageInfo, _msg: Empty) -> StdResult<Response> {
    Ok(Response::new())
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

    use crate::msg::{InstantiateMsg2, QueryMsg, ValueResp};

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
}
