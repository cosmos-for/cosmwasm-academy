use cosmwasm_std::{to_binary, DepsMut, Response, StdResult};

use crate::{
    msg::{InstantiateMsg, InstantiateResp},
    state::{COUNTER, DONATION},
};

pub fn instantiate(deps: DepsMut, msg: InstantiateMsg) -> StdResult<Response> {
    COUNTER.save(deps.storage, &msg.init)?;
    DONATION.save(deps.storage, &msg.minimal_donation)?;

    let data = InstantiateResp::new(msg.init, msg.minimal_donation);
    Ok(Response::new().set_data(to_binary(&data)?))
}

pub mod exec {
    use cosmwasm_std::{to_binary, DepsMut, MessageInfo, Response, StdResult};

    use crate::{
        msg::{DonateResp, IncrementResp},
        state::{COUNTER, DONATION},
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

        if info
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
