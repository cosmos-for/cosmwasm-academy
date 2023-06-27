use cosmwasm_std::{to_binary, DepsMut, Response, StdResult};

use crate::{msg::InstantiateResp, state::COUNTER};

pub fn instantiate(deps: DepsMut, init: u64) -> StdResult<Response> {
    COUNTER.save(deps.storage, &init)?;
    let data = InstantiateResp::new(init);
    Ok(Response::new().set_data(to_binary(&data)?))
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
