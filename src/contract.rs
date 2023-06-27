use cosmwasm_std::{to_binary, DepsMut, Response, StdResult};

use crate::{msg::InstantiateResp, state::COUNTER};

pub fn instantiate(deps: DepsMut, init: u64) -> StdResult<Response> {
    COUNTER.save(deps.storage, &init)?;
    let data = InstantiateResp::new(init);
    Ok(Response::new().set_data(to_binary(&data)?))
}

pub mod exec {
    use cosmwasm_std::{DepsMut, StdResult, Response, to_binary};

    use crate::{state::COUNTER, msg::IncrementResp};

    pub fn increment(deps: DepsMut, value: u64, sender: &str) -> StdResult<Response> {
        let value = COUNTER.update(deps.storage, |counter| -> StdResult<_> {
            Ok(counter + value)
        })?;
    
        let resp: Response = Response::new()
            .add_attribute("action", "increment")
            .add_attribute("counter", value.to_string().as_str())
            .add_attribute("sender", sender)
            .set_data(to_binary(&IncrementResp::new(value))?);

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
