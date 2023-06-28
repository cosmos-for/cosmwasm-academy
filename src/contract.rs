use cosmwasm_std::{DepsMut, MessageInfo, Response, StdResult};

use crate::{
    msg::InstantiateMsg,
    state::{COUNTER, DONATION, OWNER},
};

pub fn instantiate(deps: DepsMut, info: MessageInfo, msg: InstantiateMsg) -> StdResult<Response> {
    COUNTER.save(deps.storage, &msg.init)?;
    DONATION.save(deps.storage, &msg.minimal_donation)?;
    OWNER.save(deps.storage, &info.sender)?;

    // let data = InstantiateResp::new(msg.init, msg.minimal_donation);
    Ok(Response::new())
}

pub mod exec {
    use cosmwasm_std::{
        to_binary, BankMsg, Coin, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128,
    };

    use crate::{
        msg::{DonateResp, IncrementResp},
        state::{COUNTER, DONATION, OWNER},
    };

    pub fn increment(deps: DepsMut, value: u64, info: MessageInfo) -> StdResult<Response> {
        let value = COUNTER.update(deps.storage, |counter| -> StdResult<_> {
            Ok(counter + value)
        })?;

        let resp: Response = Response::new()
            .add_attribute("action", "increment")
            .add_attribute("counter", value.to_string().as_str())
            .add_attribute("sender", info.sender.as_str())
            .set_data(to_binary(&IncrementResp::new(value))?);

        Ok(resp)
    }

    pub fn reset(deps: DepsMut, value: u64, info: MessageInfo) -> StdResult<Response> {
        let owner = OWNER.load(deps.storage)?;
        if owner != info.sender {
            return Err(StdError::generic_err("Unauthorized"));
        }

        let value = COUNTER.update(deps.storage, |_| -> StdResult<_> { Ok(value) })?;

        let resp: Response = Response::new()
            .add_attribute("action", "reset")
            .add_attribute("counter", value.to_string().as_str())
            .add_attribute("sender", info.sender.as_str())
            .set_data(to_binary(&IncrementResp::new(value))?);

        Ok(resp)
    }

    pub fn donate(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
        let donation = DONATION.load(deps.storage)?;

        let mut counter = COUNTER.load(deps.storage)?;

        if donation.amount.is_zero()
            || info
                .funds
                .iter()
                .any(|coin| coin.denom == donation.denom && coin.amount >= donation.amount)
        {
            counter += 1;
            COUNTER.save(deps.storage, &counter)?;
        }

        let resp: Response = Response::new()
            .add_attribute("action", "donate")
            .add_attribute("counter", counter.to_string().as_str())
            .add_attribute("sender", info.sender.as_str())
            .set_data(to_binary(&DonateResp::new(counter))?);

        Ok(resp)
    }

    pub fn withdraw(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
        let owner = OWNER.load(deps.storage)?;

        if info.sender != owner {
            return Err(StdError::generic_err("Unauthorized"));
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
    ) -> StdResult<Response> {
        let owner = OWNER.load(deps.storage)?;

        if info.sender != owner {
            return Err(StdError::generic_err("Unauthorized"));
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

        // Validate receiver address
        if deps.api.addr_validate(&receiver).is_err() {
            return Err(StdError::generic_err("Invalid receiver address"));
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

    use crate::{msg::ValueResp, state::COUNTER};

    pub fn value(deps: Deps) -> StdResult<ValueResp> {
        let value = COUNTER.load(deps.storage)?;
        Ok(ValueResp { value })
    }

    pub fn increment(value: u64) -> ValueResp {
        ValueResp { value: value + 1 }
    }
}
