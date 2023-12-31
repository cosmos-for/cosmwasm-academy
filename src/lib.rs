mod contract;

pub mod error;

pub mod msg;

mod state;

#[cfg(any(test, feature = "tests"))]
pub mod multitest;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use error::ContractError;
use msg::{ExecMsg, InstantiateMsg, MigrateMsg};

use crate::contract::query;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    contract::instantiate(deps, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    contract::migrate(deps, msg.parent)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecMsg,
) -> Result<Response, ContractError> {
    use ExecMsg::*;

    match msg {
        Increment { value } => {
            contract::exec::increment(deps, value, info).map_err(ContractError::from)
        }
        Reset { value } => contract::exec::reset(deps, value, info),
        Donate {} => contract::exec::donate(deps, env, info).map_err(ContractError::from),
        Withdraw {} => contract::exec::withdraw(deps, env, info),
        WithdrawTo { receiver, funds } => {
            contract::exec::withdraw_to(deps, env, info, receiver, funds)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: msg::QueryMsg) -> StdResult<Binary> {
    use msg::QueryMsg::*;

    match msg {
        Value {} => to_binary(&query::value(deps)?),
    }
}
