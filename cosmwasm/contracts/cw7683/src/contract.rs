use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response,
};
use cw2::set_contract_version;

use crate::{
    error::{ContractError, ContractResponse, ContractResult},
    execute::{fill, open},
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    query::resolve,
    state::GATEWAY_ADDRESS,
};

// version info for migration info
const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const GO_FAST_ORDER_TYPE: &str = "GO_FAST_ORDER";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: Empty) -> Result<Response, ContractError> {
    // No state migrations performed, just returned a Response
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResponse {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(info.sender.as_str()))?;

    GATEWAY_ADDRESS.save(deps.storage, &msg.gateway_address)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, _env: Env, info: MessageInfo, msg: ExecuteMsg) -> ContractResponse {
    match msg {
        ExecuteMsg::Open { order } => open(deps, info, order),
        ExecuteMsg::Fill {
            order_id,
            origin_data,
            filler_data: _,
        } => fill(deps, info, order_id, origin_data),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::Resolve { order } => to_json_binary(&resolve(order)?),
    }
    .map_err(From::from)
}
