use crate::{
    error::ContractError,
    msg::{InstantiateMsg, Parent},
    state::{ParentDonation, State, PARENT_DONATION, STATE},
};
use cosmwasm_std::{Addr, Coin, DepsMut, MessageInfo, Response, StdResult};
use cw2::{get_contract_version, set_contract_version};
use cw_storage_plus::Item;
use serde::{Deserialize, Serialize};

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn instantiate(deps: DepsMut, info: MessageInfo, msg: InstantiateMsg) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let parent = msg.parent;

    STATE.save(
        deps.storage,
        &State::new(
            msg.counter,
            msg.minimal_donation,
            info.sender,
            parent.as_ref().map(|p| p.donating_period),
        ),
    )?;

    if let Some(parent) = parent {
        PARENT_DONATION.save(
            deps.storage,
            &ParentDonation::new(
                deps.api.addr_validate(&parent.addr)?,
                parent.donating_period,
                parent.part,
            ),
        )?;
    }

    Ok(Response::new())
}

pub fn migrate(mut deps: DepsMut, parent: Option<Parent>) -> Result<Response, ContractError> {
    let contract = get_contract_version(deps.storage)?;

    if CONTRACT_NAME != contract.contract {
        return Err(ContractError::InvalidName {
            contract: CONTRACT_NAME.into(),
        });
    }

    let resp = match contract.version.as_str() {
        "0.1.0" => migrate_0_1_0(deps.branch(), parent).map_err(ContractError::from)?,
        "0.2.0" => migrate_0_2_0(deps.branch(), parent).map_err(ContractError::from)?,
        CONTRACT_VERSION => return Ok(Response::new()),
        version => {
            return Err(ContractError::InvalidVersion {
                version: version.into(),
            })
        }
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(resp)
}

pub fn migrate_0_2_0(deps: DepsMut, parent: Option<Parent>) -> StdResult<Response> {
    #[derive(Deserialize, Serialize)]
    struct OldState {
        counter: u64,
        minimal_donation: Coin,
        owner: Addr,
    }

    const OLD_STATE: Item<OldState> = Item::new("state");

    let OldState {
        counter,
        minimal_donation,
        owner,
    } = OLD_STATE.load(deps.storage)?;

    STATE.save(
        deps.storage,
        &State::new(
            counter,
            minimal_donation,
            owner,
            parent.as_ref().map(|p| p.donating_period),
        ),
    )?;

    if let Some(parent) = parent {
        PARENT_DONATION.save(
            deps.storage,
            &ParentDonation {
                address: deps.api.addr_validate(&parent.addr)?,
                donating_parent_period: parent.donating_period,
                part: parent.part,
            },
        )?;
    }

    Ok(Response::new())
}

pub fn migrate_0_1_0(deps: DepsMut, parent: Option<Parent>) -> StdResult<Response> {
    const COUNTER: Item<u64> = Item::new("counter");
    const DONATION: Item<Coin> = Item::new("donation");
    const OWNER: Item<Addr> = Item::new("owner");

    let counter = COUNTER.load(deps.storage)?;
    let donation = DONATION.load(deps.storage)?;
    let owner = OWNER.load(deps.storage)?;

    STATE.save(
        deps.storage,
        &State::new(
            counter,
            donation,
            owner,
            parent.as_ref().map(|p| p.donating_period),
        ),
    )?;

    if let Some(parent) = parent {
        PARENT_DONATION.save(
            deps.storage,
            &ParentDonation {
                address: deps.api.addr_validate(&parent.addr)?,
                donating_parent_period: parent.donating_period,
                part: parent.part,
            },
        )?;
    }

    Ok(Response::new())
}

pub mod exec {
    use cosmwasm_std::{
        to_binary, BankMsg, Coin, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, WasmMsg,
    };

    use crate::{
        error::ContractError,
        msg::{ExecMsg, IncrementResp},
        state::{State, PARENT_DONATION, STATE},
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
        let state = STATE.load(deps.storage)?;
        if state.owner != info.sender {
            return Err(ContractError::UnauthorizedErr {
                owner: state.owner.into(),
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

    pub fn donate(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
        let mut state = STATE.load(deps.storage)?;
        let mut resp = Response::new();

        if state.minimal_donation.amount.is_zero()
            || info.funds.iter().any(|coin| {
                coin.denom == state.minimal_donation.denom
                    && coin.amount >= state.minimal_donation.amount
            })
        {
            state.counter += 1;

            if let Some(parent) = &mut state.donating_parent {
                *parent -= 1;

                if *parent == 0 {
                    let parent_donation = PARENT_DONATION.load(deps.storage)?;
                    *parent = parent_donation.donating_parent_period;

                    let funds: Vec<_> = deps
                        .querier
                        .query_all_balances(env.contract.address)?
                        .into_iter()
                        .map(|mut coin| {
                            coin.amount = coin.amount * parent_donation.part;
                            coin
                        })
                        .collect();

                    let msg = WasmMsg::Execute {
                        contract_addr: parent_donation.address.to_string(),
                        msg: to_binary(&ExecMsg::Donate {})?,
                        funds,
                    };

                    resp = resp
                        .add_message(msg)
                        .add_attribute("donation_to_parent", parent_donation.address.to_string());
                }
            }

            STATE.save(deps.storage, &state)?;
        }

        resp = resp
            .add_attribute("action", "donate")
            .add_attribute("counter", state.counter.to_string().as_str())
            .add_attribute("sender", info.sender.as_str());

        Ok(resp)
    }

    pub fn withdraw(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let owner = STATE.load(deps.storage)?.owner;

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

        let owner = STATE.load(deps.storage)?.owner;

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
