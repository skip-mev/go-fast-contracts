use cosmwasm_std::{Addr, Coin, Deps, HexBinary, Order, StdError, StdResult};
use go_fast::gateway::{Config, OrderFill, RemoteDomain};
use hyperlane::mailbox::{quote_dispatch, DispatchMsg};

use crate::{
    helpers::encode_settle_order_data,
    state::{self, CONFIG, LOCAL_DOMAIN, REMOTE_DOMAINS},
};

pub fn get_config(deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config)
}

pub fn get_local_domain(deps: Deps) -> StdResult<u32> {
    let local_domain = LOCAL_DOMAIN.load(deps.storage)?;
    Ok(local_domain)
}

pub fn get_remote_domain(deps: Deps, domain: u32) -> StdResult<HexBinary> {
    let remote_domain = REMOTE_DOMAINS.load(deps.storage, domain)?;
    Ok(remote_domain)
}

pub fn get_remote_domains(deps: Deps) -> StdResult<Vec<RemoteDomain>> {
    let remote_domains = REMOTE_DOMAINS
        .range(deps.storage, None, None, Order::Ascending)
        .map(|entry| {
            let (domain, address) = entry?;

            Ok(RemoteDomain { domain, address })
        })
        .collect::<StdResult<Vec<RemoteDomain>>>()?;

    Ok(remote_domains)
}

pub fn get_order_fill(deps: Deps, order_id: HexBinary) -> StdResult<OrderFill> {
    state::order_fills().by_order_id(deps, order_id)
}

pub fn order_fills_by_filler(
    deps: Deps,
    filler: Addr,
    start_after: Option<HexBinary>,
    limit: Option<u32>,
) -> StdResult<Vec<OrderFill>> {
    let start_after = start_after.map(|x| x.to_vec());
    state::order_fills().by_filler(deps, filler, start_after, limit)
}

pub fn quote_initiate_settlement(
    deps: Deps,
    order_ids: Vec<HexBinary>,
    repayment_address: HexBinary,
    source_domain: u32,
) -> StdResult<Vec<Coin>> {
    let config = CONFIG.load(deps.storage)?;

    let remote_contract_address = REMOTE_DOMAINS.may_load(deps.storage, source_domain)?;
    if remote_contract_address.is_none() {
        return Err(StdError::generic_err("Unknown remote domain"));
    }

    let remote_contract_address = remote_contract_address.unwrap();

    let dispatch_msg = DispatchMsg {
        dest_domain: source_domain,
        recipient_addr: remote_contract_address.clone(),
        msg_body: encode_settle_order_data(repayment_address, order_ids),
        hook: Some(config.hook_addr.clone()),
        metadata: None,
    };

    quote_dispatch(deps, config.mailbox_addr, dispatch_msg)
}
