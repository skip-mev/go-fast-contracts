use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response,
    StdResult,
};
use cw2::set_contract_version;

use crate::{
    error::{ContractError, ContractResponse},
    execute::{
        add_remote_domain, fill_order, handle, initiate_settlement, initiate_timeout, submit_order,
        update_config,
    },
    query::{
        get_config, get_local_domain, get_order_fill, get_remote_domain, get_remote_domains,
        order_fills_by_filler, quote_initiate_settlement,
    },
    state::{CONFIG, LOCAL_DOMAIN, NONCE},
};
use go_fast::gateway::{Config, ExecuteMsg, InstantiateMsg, QueryMsg};

// version info for migration info
const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

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

    let config = Config {
        token_denom: msg.token_denom,
        address_prefix: msg.address_prefix,
        mailbox_addr: msg.mailbox_addr,
        hook_addr: msg.hook_addr,
    };

    CONFIG.save(deps.storage, &config)?;

    LOCAL_DOMAIN.save(deps.storage, &msg.local_domain)?;

    NONCE.save(deps.storage, &0)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> ContractResponse {
    match msg {
        ExecuteMsg::FillOrder { filler, order } => fill_order(deps, env, info, filler, order),
        ExecuteMsg::InitiateSettlement {
            order_ids,
            repayment_address,
        } => initiate_settlement(deps, info, order_ids, repayment_address),
        ExecuteMsg::InitiateTimeout { orders } => initiate_timeout(deps, env, info, orders),
        ExecuteMsg::UpdateConfig { config } => update_config(deps, info, config),
        ExecuteMsg::AddRemoteDomain { domain, address } => {
            add_remote_domain(deps, info, domain, address)
        }
        ExecuteMsg::SubmitOrder {
            sender,
            recipient,
            amount_in,
            amount_out,
            destination_domain,
            timeout_timestamp,
            data,
        } => submit_order(
            deps,
            info,
            sender,
            recipient,
            amount_in,
            amount_out,
            destination_domain,
            timeout_timestamp,
            data,
        ),
        ExecuteMsg::Handle(handle_msg) => handle(deps, info, handle_msg),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&get_config(deps)?),
        QueryMsg::LocalDomain {} => to_json_binary(&get_local_domain(deps)?),
        QueryMsg::RemoteDomain { domain } => to_json_binary(&get_remote_domain(deps, domain)?),
        QueryMsg::RemoteDomains {} => to_json_binary(&get_remote_domains(deps)?),
        QueryMsg::OrderFill { order_id } => to_json_binary(&get_order_fill(deps, order_id)?),
        QueryMsg::QuoteInitiateSettlement {
            order_ids,
            repayment_address,
            source_domain,
        } => to_json_binary(&quote_initiate_settlement(
            deps,
            order_ids,
            repayment_address,
            source_domain,
        )?),
        QueryMsg::OrderFillsByFiller {
            filler,
            start_after,
            limit,
        } => to_json_binary(&order_fills_by_filler(deps, filler, start_after, limit)?),
    }
}
