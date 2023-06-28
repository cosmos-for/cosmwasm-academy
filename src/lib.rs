mod contract;

pub mod error;

pub mod msg;

mod state;

use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use error::ContractError;
use msg::{ExecMsg, InstantiateMsg};

use crate::contract::query;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    contract::instantiate(deps, info, msg)
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecMsg) -> Result<Response, ContractError> {
    use ExecMsg::*;

    match msg {
        Increment { value } => contract::exec::increment(deps, value, info).map_err(ContractError::from),
        Reset { value } => contract::exec::reset(deps, value, info),
        Donate {} => contract::exec::donate(deps, info).map_err(ContractError::from),
        Withdraw {} => contract::exec::withdraw(deps, env, info),
        WithdrawTo { receiver, funds } => {
            contract::exec::withdraw_to(deps, env, info, receiver, funds)
        }
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

    fn other_sender() -> Addr {
        Addr::unchecked("sei1aan9kqywf4rf274cal0hj6eyly6wu0uv7edxy2")
    }

    fn owner() -> Addr {
        Addr::unchecked("sei1zj6fjsc2gkce878ukzg6g9wy8cl8p554dlggxd")
    }

    fn instantiate_msg() -> InstantiateMsg {
        InstantiateMsg::new(0, ten_atom())
    }

    fn zero_funds_instantiate_msg() -> InstantiateMsg {
        InstantiateMsg::new(0, zero_atom())
    }

    fn ten_atom() -> Coin {
        Coin::new(10, "atom")
    }

    fn zero_atom() -> Coin {
        Coin::new(0, "atom")
    }

    const COUNTING_LABEL: &str = "counting-contract";
    const EMPTY_FUNDS: &[Coin] = &[];
    const NO_ADMIN: Option<String> = None;

    #[test]
    fn query_value_should_work() {
        let mut app = App::default();

        let contract_id = app.store_code(counting_contract());
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                sender(),
                &instantiate_msg(),
                EMPTY_FUNDS,
                COUNTING_LABEL,
                NO_ADMIN,
            )
            .unwrap();

        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &msg::QueryMsg::Value {})
            .unwrap();

        assert_eq!(resp.value, 0);
    }

    #[test]
    fn increment_query_should_work() {
        let mut app = App::default();
        let contract_id = app.store_code(counting_contract());
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                sender(),
                &instantiate_msg(),
                EMPTY_FUNDS,
                COUNTING_LABEL,
                NO_ADMIN,
            )
            .unwrap();

        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Incremented { value: 3 })
            .unwrap();

        assert_eq!(resp.value, 4);
    }

    #[test]
    fn increment_should_work() {
        let mut app = App::default();
        let contract_id = app.store_code(counting_contract());
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                sender(),
                &instantiate_msg(),
                EMPTY_FUNDS,
                COUNTING_LABEL,
                NO_ADMIN,
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
    fn reset_should_work() {
        let mut app = App::default();
        let contract_id = app.store_code(counting_contract());
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                sender(),
                &instantiate_msg(),
                EMPTY_FUNDS,
                COUNTING_LABEL,
                NO_ADMIN,
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
    fn reset_not_owner_should_fail() {
        let mut app = App::default();
        let contract_id = app.store_code(counting_contract());
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                sender(),
                &instantiate_msg(),
                EMPTY_FUNDS,
                COUNTING_LABEL,
                NO_ADMIN,
            )
            .unwrap();
        let err = app
            .execute_contract(
                other_sender(),
                contract_addr,
                &ExecMsg::Reset { value: 10 },
                EMPTY_FUNDS,
            )
            .unwrap_err();

        assert_eq!(
            ContractError::UnauthorizedErr { owner: sender().to_string() },
            err.downcast().unwrap(),
        );
    }

    #[test]
    fn donate_should_work() {
        let mut app = App::default();
        let contract_id = app.store_code(counting_contract());
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                sender(),
                &instantiate_msg(),
                EMPTY_FUNDS,
                COUNTING_LABEL,
                NO_ADMIN,
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
    fn donate_expecting_no_funds_should_work() {
        let mut app = App::default();
        let contract_id = app.store_code(counting_contract());
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                sender(),
                &zero_funds_instantiate_msg(),
                EMPTY_FUNDS,
                COUNTING_LABEL,
                NO_ADMIN,
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

    #[test]
    fn donate_with_funds_should_work() {
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
                COUNTING_LABEL,
                NO_ADMIN,
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

    #[test]
    fn withdraw_should_work() {
        let mut app = AppBuilder::new().build(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &sender(), coins(100, "atom"))
                .unwrap();

            router
                .bank
                .init_balance(storage, &other_sender(), vec![ten_atom()])
                .unwrap();
        });

        let contract_id = app.store_code(counting_contract());
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                owner(),
                &zero_funds_instantiate_msg(),
                &[],
                COUNTING_LABEL,
                NO_ADMIN,
            )
            .unwrap();

        let contract_balance = app
            .wrap()
            .query_balance(contract_addr.clone(), "atom")
            .unwrap();
        assert_eq!(contract_balance, Coin::new(0, "atom"));

        app.execute_contract(
            sender(),
            contract_addr.clone(),
            &ExecMsg::Donate {},
            vec![ten_atom()].as_slice(),
        )
        .unwrap();

        app.execute_contract(
            other_sender(),
            contract_addr.clone(),
            &ExecMsg::Donate {},
            vec![ten_atom()].as_slice(),
        )
        .unwrap();

        let contract_balance = app
            .wrap()
            .query_balance(contract_addr.clone(), "atom")
            .unwrap();
        assert_eq!(contract_balance, Coin::new(20, "atom"));

        app.execute_contract(owner(), contract_addr, &ExecMsg::Withdraw {}, &[])
            .unwrap();

        let sender_balance = app.wrap().query_balance(sender(), "atom").unwrap();
        assert_eq!(sender_balance, Coin::new(90, "atom"));

        let other_balance = app.wrap().query_balance(other_sender(), "atom").unwrap();
        assert_eq!(other_balance, Coin::new(0, "atom"));

        let owner_balance = app.wrap().query_balance(owner(), "atom").unwrap();
        assert_eq!(owner_balance, Coin::new(20, "atom"));
    }

    #[test]
    fn withdraw_not_owner_should_fail() {
        let mut app = App::default();

        let contract_id = app.store_code(counting_contract());
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                owner(),
                &zero_funds_instantiate_msg(),
                &[],
                COUNTING_LABEL,
                NO_ADMIN,
            )
            .unwrap();

        let err = app.execute_contract(sender(), contract_addr, &ExecMsg::Withdraw {}, &[])
            .unwrap_err();

        assert_eq!(        
            ContractError::UnauthorizedErr { owner: owner().to_string() },
            err.downcast().unwrap(),
        )
    }

    #[test]
    fn withdraw_to_should_work() {
        let mut app = AppBuilder::new().build(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &sender(), coins(100, "atom"))
                .unwrap();

            router
                .bank
                .init_balance(storage, &other_sender(), vec![ten_atom()])
                .unwrap();
        });

        let contract_id = app.store_code(counting_contract());
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                owner(),
                &zero_funds_instantiate_msg(),
                &[],
                COUNTING_LABEL,
                NO_ADMIN,
            )
            .unwrap();

        let contract_balance = app
            .wrap()
            .query_balance(contract_addr.clone(), "atom")
            .unwrap();
        assert_eq!(contract_balance, Coin::new(0, "atom"));

        app.execute_contract(
            sender(),
            contract_addr.clone(),
            &ExecMsg::Donate {},
            vec![ten_atom()].as_slice(),
        )
        .unwrap();

        app.execute_contract(
            other_sender(),
            contract_addr.clone(),
            &ExecMsg::Donate {},
            vec![ten_atom()].as_slice(),
        )
        .unwrap();

        let contract_balance = app
            .wrap()
            .query_balance(contract_addr.clone(), "atom")
            .unwrap();
        assert_eq!(contract_balance, Coin::new(20, "atom"));

        let send_funds = coins(10, "atom");
        app.execute_contract(
            owner(),
            contract_addr,
            &ExecMsg::WithdrawTo {
                receiver: other_sender().to_string(),
                funds: send_funds,
            },
            &[],
        )
        .unwrap();

        let sender_balance = app.wrap().query_balance(sender(), "atom").unwrap();
        assert_eq!(sender_balance, Coin::new(90, "atom"));

        let other_balance = app.wrap().query_balance(other_sender(), "atom").unwrap();
        assert_eq!(other_balance, Coin::new(10, "atom"));
    }

    #[test]
    fn withdraw_to_not_owner_should_fail() {
        let mut app = App::default();

        let contract_id = app.store_code(counting_contract());
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                owner(),
                &zero_funds_instantiate_msg(),
                &[],
                COUNTING_LABEL,
                NO_ADMIN,
            )
            .unwrap();

        let send_funds = coins(10, "atom");
        let err = app.execute_contract(
            sender(),
            contract_addr,
            &ExecMsg::WithdrawTo {
                receiver: other_sender().to_string(),
                funds: send_funds,
            },
            &[],
        )
        .unwrap_err();

        assert_eq!(
            ContractError::UnauthorizedErr { owner: owner().to_string() },
            err.downcast().unwrap(),
        )
    }

    #[test]
    fn withdraw_to_invalid_address_should_fail() {
        let mut app = App::default();

        let contract_id = app.store_code(counting_contract());
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                owner(),
                &zero_funds_instantiate_msg(),
                &[],
                COUNTING_LABEL,
                NO_ADMIN,
            )
            .unwrap();

        let send_funds = coins(10, "atom");
        let err = app.execute_contract(
            owner(),
            contract_addr,
            &ExecMsg::WithdrawTo {
                receiver: "".into(),
                funds: send_funds,
            },
            &[],
        )
        .unwrap_err();

        assert_eq!(
            ContractError::InvalidAddressErr { address: "".into() },
            err.downcast().unwrap(),
        )
    }
}
