use std::vec;

use cosmwasm_std::{coins, to_binary, Coin};
use cw_multi_test::App;

use crate::{
    msg::{IncrementResp, ValueResp},
    state::{State, STATE},
};
use counting_contract_0_1::multitest::CountingContract as CountingContract_0_1;

use super::*;

const COUNTING_LABEL: &str = "counting-contract";
const EMPTY_FUNDS: &[Coin] = &[];
const ATOM: &str = "atom";

#[test]
fn query_value_should_work() {
    let mut app = App::default();

    let contract_id = CountingContract::store_code(&mut app);
    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        sender(),
        COUNTING_LABEL,
        0,
        zero_atom(),
    )
    .unwrap();

    let resp: ValueResp = contract.query_value(&app).unwrap();

    assert_eq!(resp.value, 0);
}

#[test]
fn increment_should_work() {
    let mut app = App::default();
    let contract_id = CountingContract::store_code(&mut app);
    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        sender(),
        COUNTING_LABEL,
        0,
        ten_atom(),
    )
    .unwrap();

    let resp = contract.increment(&mut app, sender(), 10).unwrap();

    let expected_value = 10;
    let data = IncrementResp::new(expected_value);
    assert_eq!(resp.data.unwrap(), to_binary(&data).unwrap());

    assert!(CountingContract::verify_events(
        resp.events,
        "increment",
        sender().as_str()
    ));

    let resp: ValueResp = contract.query_value(&app).unwrap();

    assert_eq!(resp.value, 10);
}

#[test]
fn reset_should_work() {
    let mut app = App::default();
    let contract_id = CountingContract::store_code(&mut app);
    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        sender(),
        COUNTING_LABEL,
        0,
        ten_atom(),
    )
    .unwrap();

    let resp = contract.reset(&mut app, sender(), 10).unwrap();

    assert_eq!(resp.data, Some(to_binary(&IncrementResp::new(10)).unwrap()));

    let resp = contract.query_value(&app).unwrap();

    assert_eq!(resp.value, 10);
}

#[test]
fn reset_not_owner_should_fail() {
    let mut app = App::default();
    let contract_id = CountingContract::store_code(&mut app);
    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        sender(),
        COUNTING_LABEL,
        0,
        ten_atom(),
    )
    .unwrap();
    let err = contract.reset(&mut app, other_sender(), 10).unwrap_err();

    assert_eq!(
        ContractError::UnauthorizedErr {
            owner: sender().to_string()
        },
        err,
    );
}

#[test]
fn donate_should_work() {
    let mut app = App::default();
    let contract_id = CountingContract::store_code(&mut app);
    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        sender(),
        COUNTING_LABEL,
        0,
        ten_atom(),
    )
    .unwrap();

    let resp = contract.donate(&mut app, sender(), EMPTY_FUNDS).unwrap();

    assert!(CountingContract::verify_events(
        resp.events,
        "donate",
        sender().as_str()
    ));

    let resp: ValueResp = contract.query_value(&app).unwrap();

    assert_eq!(resp.value, 0);
}

#[test]
fn donate_expecting_no_funds_should_work() {
    let mut app = App::default();
    let contract_id = CountingContract::store_code(&mut app);
    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        sender(),
        COUNTING_LABEL,
        0,
        zero_atom(),
    )
    .unwrap();

    let resp = contract.donate(&mut app, sender(), &[]).unwrap();

    assert!(CountingContract::verify_events(
        resp.events,
        "donate",
        sender().as_str()
    ));

    let resp = contract.query_value(&app).unwrap();

    assert_eq!(resp.value, 1);
}

#[test]
fn donate_with_funds_should_work() {
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender(), coins(100, ATOM))
            .unwrap();
    });

    let contract_id = CountingContract::store_code(&mut app);
    let contract = CountingContract::instantiate_with_funds_admin(
        &mut app,
        contract_id,
        sender(),
        COUNTING_LABEL,
        0,
        zero_atom(),
        vec![ten_atom()].as_slice(),
        None,
    )
    .unwrap();

    let sender_balance = CountingContract::query_balance(&app, sender(), ATOM).unwrap();
    assert_eq!(sender_balance, Coin::new(90, ATOM));

    let contract_balance = CountingContract::query_balance(&app, contract.addr(), ATOM).unwrap();

    assert_eq!(contract_balance, Coin::new(10, ATOM));

    let resp = contract
        .donate(&mut app, sender(), vec![ten_atom()].as_slice())
        .unwrap();

    let balance = app.wrap().query_balance(sender(), ATOM).unwrap();
    assert_eq!(balance, Coin::new(80, ATOM));

    let contract_balance = CountingContract::query_balance(&app, contract.addr(), ATOM).unwrap();
    assert_eq!(contract_balance, Coin::new(20, ATOM));

    assert!(CountingContract::verify_events(
        resp.events,
        "donate",
        sender().as_str()
    ));

    let resp = contract.query_value(&app).unwrap();

    assert_eq!(resp.value, 1);
}

#[test]
fn withdraw_should_work() {
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender(), coins(100, ATOM))
            .unwrap();

        router
            .bank
            .init_balance(storage, &other_sender(), vec![ten_atom()])
            .unwrap();
    });

    let contract_id = CountingContract::store_code(&mut app);
    let contract = CountingContract::instantiate_with_funds_admin(
        &mut app,
        contract_id,
        owner(),
        COUNTING_LABEL,
        0,
        zero_atom(),
        &[],
        None,
    )
    .unwrap();

    let contract_balance = CountingContract::query_balance(&app, contract.addr(), ATOM).unwrap();
    assert_eq!(contract_balance, Coin::new(0, ATOM));

    contract
        .donate(&mut app, sender(), vec![ten_atom()].as_slice())
        .unwrap();
    contract
        .donate(&mut app, other_sender(), vec![ten_atom()].as_slice())
        .unwrap();

    let contract_balance = CountingContract::query_balance(&app, contract.addr(), ATOM).unwrap();
    assert_eq!(contract_balance, Coin::new(20, ATOM));

    contract.withdraw(&mut app, owner()).unwrap();

    let sender_balance = CountingContract::query_balance(&app, sender(), ATOM).unwrap();
    assert_eq!(sender_balance, Coin::new(90, ATOM));

    let other_balance = CountingContract::query_balance(&app, other_sender(), ATOM).unwrap();
    assert_eq!(other_balance, Coin::new(0, ATOM));

    let owner_balance = CountingContract::query_balance(&app, owner(), ATOM).unwrap();
    assert_eq!(owner_balance, Coin::new(20, ATOM));
}

#[test]
fn withdraw_not_owner_should_fail() {
    let mut app = App::default();

    let contract_id = CountingContract::store_code(&mut app);
    let contract = CountingContract::instantiate_with_funds_admin(
        &mut app,
        contract_id,
        owner(),
        COUNTING_LABEL,
        0,
        zero_atom(),
        &[],
        None,
    )
    .unwrap();

    let err = contract.withdraw(&mut app, other_sender()).unwrap_err();

    assert_eq!(
        ContractError::UnauthorizedErr {
            owner: owner().to_string()
        },
        err,
    )
}

#[test]
fn withdraw_to_should_work() {
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender(), coins(100, ATOM))
            .unwrap();

        router
            .bank
            .init_balance(storage, &other_sender(), vec![ten_atom()])
            .unwrap();
    });

    let contract_id = CountingContract::store_code(&mut app);
    let contract = CountingContract::instantiate_with_funds_admin(
        &mut app,
        contract_id,
        owner(),
        COUNTING_LABEL,
        0,
        zero_atom(),
        &[],
        None,
    )
    .unwrap();

    let contract_balance = CountingContract::query_balance(&app, contract.addr(), ATOM).unwrap();
    assert_eq!(contract_balance, Coin::new(0, ATOM));

    contract
        .donate(&mut app, sender(), vec![ten_atom()].as_slice())
        .unwrap();
    contract
        .donate(&mut app, other_sender(), vec![ten_atom()].as_slice())
        .unwrap();

    let contract_balance = CountingContract::query_balance(&app, contract.addr(), ATOM).unwrap();
    assert_eq!(contract_balance, Coin::new(20, ATOM));

    let send_funds = coins(10, ATOM);
    contract
        .withdraw_to(&mut app, owner(), other_sender().to_string(), send_funds)
        .unwrap();

    let sender_balance = app.wrap().query_balance(sender(), ATOM).unwrap();
    assert_eq!(sender_balance, Coin::new(90, ATOM));

    let other_balance = app.wrap().query_balance(other_sender(), ATOM).unwrap();
    assert_eq!(other_balance, Coin::new(10, ATOM));
}

#[test]
fn withdraw_to_not_owner_should_fail() {
    let mut app = App::default();

    let contract_id = CountingContract::store_code(&mut app);
    let contract = CountingContract::instantiate_with_funds_admin(
        &mut app,
        contract_id,
        owner(),
        COUNTING_LABEL,
        0,
        zero_atom(),
        &[],
        None,
    )
    .unwrap();

    let send_funds = coins(10, ATOM);
    let err = contract
        .withdraw_to(&mut app, sender(), other_sender().to_string(), send_funds)
        .unwrap_err();

    assert_eq!(
        ContractError::UnauthorizedErr {
            owner: owner().to_string()
        },
        err,
    )
}

#[test]
fn withdraw_to_invalid_address_should_fail() {
    let mut app = App::default();

    let contract_id = CountingContract::store_code(&mut app);
    let contract = CountingContract::instantiate_with_funds_admin(
        &mut app,
        contract_id,
        owner(),
        COUNTING_LABEL,
        0,
        zero_atom(),
        &[],
        None,
    )
    .unwrap();

    let send_funds = coins(10, ATOM);
    let err = contract
        .withdraw_to(&mut app, sender(), "ABC".into(), send_funds)
        .unwrap_err();

    assert_eq!(
        ContractError::InvalidAddressErr {
            address: "ABC".into()
        },
        err,
    )
}

#[test]
fn migrate_should_work() {
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender(), coins(10, ATOM))
            .unwrap();
    });

    let old_code_id = CountingContract_0_1::store_code(&mut app);
    let new_code_id = CountingContract::store_code(&mut app);

    let contract = CountingContract_0_1::instantiate_with_funds(
        &mut app,
        old_code_id,
        owner(),
        COUNTING_LABEL,
        0,
        zero_atom(),
        &[],
        other_sender(),
    )
    .unwrap();

    contract
        .donate(&mut app, sender(), &coins(10, ATOM))
        .unwrap();

    let contract =
        CountingContract::migrate(&mut app, contract.addr(), new_code_id, other_sender()).unwrap();

    let resp = contract.query_value(&app).unwrap();
    assert_eq!(resp.value, 1);

    let state = STATE.query(&app.wrap(), contract.addr()).unwrap();

    assert_eq!(state, State::new(1, zero_atom()))
}
