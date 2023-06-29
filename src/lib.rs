mod contract;

pub mod error;

pub mod msg;

mod state;

#[cfg(test)]
pub mod multitest;

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
    }
}
