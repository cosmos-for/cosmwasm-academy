use cosmwasm_std::{Coin, DepsMut, MessageInfo, Response, StdResult};
use cw_storage_plus::Item;
use cw2::{get_contract_version, set_contract_version};
use crate::{
    msg::InstantiateMsg,
    state::{State, OWNER, STATE}, error::ContractError,
};

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn instantiate(deps: DepsMut, info: MessageInfo, msg: InstantiateMsg) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    STATE.save(deps.storage, &State::new(msg.counter, msg.minimal_donation))?;
    OWNER.save(deps.storage, &info.sender)?;

    Ok(Response::new())
}

pub fn migrate(mut deps: DepsMut) -> Result<Response, ContractError> {
    let contract = get_contract_version(deps.storage)?;

    if CONTRACT_NAME != contract.contract {
        return Err(ContractError::InvalidName { contract: CONTRACT_NAME.into() });
    }

    let resp = match contract.version.as_str() {
        "0.1.0" => migrate_0_1_0(deps.branch()).map_err(ContractError::from)?,
        CONTRACT_VERSION => return Ok(Response::new()),
        version => return Err(ContractError::InvalidVersion { version: version.to_owned() }),
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(resp)
}
pub fn migrate_0_1_0(deps: DepsMut) -> StdResult<Response> {
    const COUNTER: Item<u64> = Item::new("counter");
    const DONATION: Item<Coin> = Item::new("donation");

    let counter = COUNTER.load(deps.storage)?;
    let donation = DONATION.load(deps.storage)?;

    STATE.save(deps.storage, &State::new(counter, donation))?;

    Ok(Response::new())
}

pub mod exec {
    use cosmwasm_std::{
        to_binary, BankMsg, Coin, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
    };

    use crate::{
        error::ContractError,
        msg::IncrementResp,
        state::{State, OWNER, STATE},
    };

    pub fn increment(deps: DepsMut, value: u64, info: MessageInfo) -> StdResult<Response> {
        let new_state = STATE.update(deps.storage, |state| -> StdResult<_> {
            Ok(State {
                counter: state.counter + value,
                ..state
            })
        })?;

        let resp: Response = Response::new()
            .add_attribute("action", "increment")
            .add_attribute("counter", new_state.counter.to_string().as_str())
            .add_attribute("sender", info.sender.as_str())
            .set_data(to_binary(&IncrementResp::new(new_state.counter))?);

        Ok(resp)
    }

    pub fn reset(deps: DepsMut, value: u64, info: MessageInfo) -> Result<Response, ContractError> {
        let owner = OWNER.load(deps.storage)?;
        if owner != info.sender {
            return Err(ContractError::UnauthorizedErr {
                owner: owner.into(),
            });
        }

        let state = STATE.update(deps.storage, |state| -> StdResult<_> {
            Ok(State {
                counter: value,
                ..state
            })
        })?;

        let resp: Response = Response::new()
            .add_attribute("action", "reset")
            .add_attribute("counter", state.counter.to_string().as_str())
            .add_attribute("sender", info.sender.as_str())
            .set_data(to_binary(&IncrementResp::new(state.counter))?);

        Ok(resp)
    }

    pub fn donate(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
        let state = STATE.load(deps.storage)?;

        if state.minimal_donation.amount.is_zero()
            || info.funds.iter().any(|coin| {
                coin.denom == state.minimal_donation.denom
                    && coin.amount >= state.minimal_donation.amount
            })
        {
            STATE.save(
                deps.storage,
                &State {
                    counter: state.counter + 1,
                    ..state
                },
            )?;
        }

        let resp: Response = Response::new()
            .add_attribute("action", "donate")
            .add_attribute("counter", state.counter.to_string().as_str())
            .add_attribute("sender", info.sender.as_str());
        // .set_data(to_binary(&DonateResp::new(state.counter))?);

        Ok(resp)
    }

    pub fn withdraw(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let owner = OWNER.load(deps.storage)?;

        if info.sender != owner {
            return Err(ContractError::UnauthorizedErr {
                owner: owner.into(),
            });
        }

        let contract_balances = deps.querier.query_all_balances(env.contract.address)?;
        let bank_msg = BankMsg::Send {
            to_address: owner.to_string(),
            amount: contract_balances,
        };

        let resp: Response = Response::new()
            .add_message(bank_msg)
            .add_attribute("action", "withdraw")
            .add_attribute("sender", info.sender.as_str());

        Ok(resp)
    }

    pub fn withdraw_to(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        receiver: String,
        funds: Vec<Coin>,
    ) -> Result<Response, ContractError> {
        // Validate receiver address
        if deps.api.addr_validate(&receiver).is_err() {
            return Err(ContractError::InvalidAddressErr { address: receiver });
        }

        let owner = OWNER.load(deps.storage)?;

        if info.sender != owner {
            return Err(ContractError::UnauthorizedErr {
                owner: owner.into(),
            });
        }

        let mut contract_balances = deps.querier.query_all_balances(env.contract.address)?;

        if !funds.is_empty() {
            for coin in &mut contract_balances {
                let limit = funds
                    .iter()
                    .find(|c| c.denom == coin.denom)
                    .map(|c| c.amount)
                    .unwrap_or(Uint128::zero());

                coin.amount = std::cmp::min(coin.amount, limit);
            }
        }

        let bank_msg = BankMsg::Send {
            to_address: receiver,
            amount: contract_balances,
        };

        let resp: Response = Response::new()
            .add_message(bank_msg)
            .add_attribute("action", "withdraw")
            .add_attribute("sender", info.sender.as_str());

        Ok(resp)
    }
}

pub mod query {
    use cosmwasm_std::{Deps, StdResult};

    use crate::{msg::ValueResp, state::STATE};

    pub fn value(deps: Deps) -> StdResult<ValueResp> {
        let value = STATE.load(deps.storage)?.counter;
        Ok(ValueResp { value })
    }
}
