mod contract;

pub mod msg;

mod state;

use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use msg::{ExecMsg, InstantiateMsg};

use crate::contract::query;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    contract::instantiate(deps, msg)
}

#[entry_point]
pub fn execute(deps: DepsMut, _env: Env, info: MessageInfo, msg: ExecMsg) -> StdResult<Response> {
    use ExecMsg::*;

    match msg {
        Increment { value } => contract::exec::increment(deps, value, info),
        Reset { value } => contract::exec::reset(deps, value, info),
        Donate {} => contract::exec::donate(deps, info),
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

    use cosmwasm_std::{coins, Addr, Attribute, Coin, Empty};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    use crate::msg::{IncrementResp, QueryMsg, ValueResp};

    fn counting_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(execute, instantiate, query);
        Box::new(contract)
    }

    fn sender() -> Addr {
        Addr::unchecked("sei18rszd3tmgpjvjwq2qajtmn5jqvtscd2yuygl4z")
    }

    fn instantiate_msg() -> InstantiateMsg {
        InstantiateMsg::new(0, ten_atom())
    }

    fn ten_atom() -> Coin {
        Coin::new(10, "atom")
    }
    const LABEL: &str = "counting-contract";
    const EMPTY_FUNDS: &[Coin] = &[];
    const ADMIN: Option<String> = None;

    #[test]
    fn query_value() {
        let mut app = App::default();

        let contract_id = app.store_code(counting_contract());
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                sender(),
                &instantiate_msg(),
                EMPTY_FUNDS,
                LABEL,
                ADMIN,
            )
            .unwrap();

        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &msg::QueryMsg::Value {})
            .unwrap();

        assert_eq!(resp.value, 0);
    }

    #[test]
    fn query_increment() {
        let mut app = App::default();
        let contract_id = app.store_code(counting_contract());
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                sender(),
                &instantiate_msg(),
                EMPTY_FUNDS,
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
                &instantiate_msg(),
                EMPTY_FUNDS,
                LABEL,
                ADMIN,
            )
            .unwrap();
        let resp = app
            .execute_contract(
                sender(),
                contract_addr.clone(),
                &ExecMsg::Increment { value: 10 },
                EMPTY_FUNDS,
            )
            .unwrap();

        let expected_value = 10;
        let data = IncrementResp::new(expected_value);
        assert_eq!(resp.data.unwrap(), to_binary(&data).unwrap());

        let wasm_event = resp.events.iter().find(|e| e.ty == "wasm").unwrap();

        let b = vec![
            Attribute::new("action", "increment"),
            Attribute::new("sender", sender()),
            Attribute::new("counter", expected_value.to_string()),
        ];

        assert!(b.iter().all(|item| wasm_event.attributes.contains(item)));

        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Value {})
            .unwrap();

        assert_eq!(resp.value, 10);
    }

    #[test]
    fn execute_reset() {
        let mut app = App::default();
        let contract_id = app.store_code(counting_contract());
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                sender(),
                &instantiate_msg(),
                EMPTY_FUNDS,
                LABEL,
                ADMIN,
            )
            .unwrap();
        let resp = app
            .execute_contract(
                sender(),
                contract_addr.clone(),
                &ExecMsg::Reset { value: 10 },
                EMPTY_FUNDS,
            )
            .unwrap();

        assert_eq!(resp.data, Some(to_binary(&IncrementResp::new(10)).unwrap()));

        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Value {})
            .unwrap();

        assert_eq!(resp.value, 10);
    }

    #[test]
    fn execute_donate() {
        let mut app = App::default();
        let contract_id = app.store_code(counting_contract());
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                sender(),
                &instantiate_msg(),
                EMPTY_FUNDS,
                LABEL,
                ADMIN,
            )
            .unwrap();
        let resp = app
            .execute_contract(
                sender(),
                contract_addr.clone(),
                &ExecMsg::Donate {},
                EMPTY_FUNDS,
            )
            .unwrap();

        let expected_value = 0;
        let data = IncrementResp::new(expected_value);
        assert_eq!(resp.data.unwrap(), to_binary(&data).unwrap());

        let wasm_event = resp.events.iter().find(|e| e.ty == "wasm").unwrap();

        let b = vec![
            Attribute::new("action", "donate"),
            Attribute::new("sender", sender()),
            Attribute::new("counter", expected_value.to_string()),
        ];

        assert!(b.iter().all(|item| wasm_event.attributes.contains(item)));

        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Value {})
            .unwrap();

        assert_eq!(resp.value, 0);
    }

    #[test]
    fn execute_donate_with_funds() {
        let mut app = AppBuilder::new().build(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &sender(), coins(100, "atom"))
                .unwrap();
        });

        let contract_id = app.store_code(counting_contract());
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                sender(),
                &instantiate_msg(),
                vec![ten_atom()].as_slice(),
                LABEL,
                ADMIN,
            )
            .unwrap();

        let sender_balance = app.wrap().query_balance(sender(), "atom").unwrap();
        assert_eq!(sender_balance, Coin::new(90, "atom"));

        let contract_balance = app
            .wrap()
            .query_balance(contract_addr.clone(), "atom")
            .unwrap();
        assert_eq!(contract_balance, Coin::new(10, "atom"));

        let resp = app
            .execute_contract(
                sender(),
                contract_addr.clone(),
                &ExecMsg::Donate {},
                vec![ten_atom()].as_slice(),
            )
            .unwrap();

        let balance = app.wrap().query_balance(sender(), "atom").unwrap();
        assert_eq!(balance, Coin::new(80, "atom"));

        let contract_balance = app
            .wrap()
            .query_balance(contract_addr.clone(), "atom")
            .unwrap();
        assert_eq!(contract_balance, Coin::new(20, "atom"));

        let expected_value = 1;
        let data = IncrementResp::new(expected_value);
        assert_eq!(resp.data.unwrap(), to_binary(&data).unwrap());

        let wasm_event = resp.events.iter().find(|e| e.ty == "wasm").unwrap();

        let b = vec![
            Attribute::new("action", "donate"),
            Attribute::new("sender", sender()),
            Attribute::new("counter", expected_value.to_string()),
        ];

        assert!(b.iter().all(|item| wasm_event.attributes.contains(item)));

        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Value {})
            .unwrap();

        assert_eq!(resp.value, 1);
    }
}
