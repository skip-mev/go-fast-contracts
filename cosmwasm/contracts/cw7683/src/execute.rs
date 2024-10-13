use crate::{
    error::ContractResponse,
    query::resolve,
    state::GATEWAY_ADDRESS,
    types::{OnchainCrossChainOrder, OrderData},
};
use cosmwasm_std::{
    from_json, to_json_binary, Binary, DepsMut, HexBinary, MessageInfo, Response, WasmMsg,
};
use go_fast::{
    gateway::ExecuteMsg as GatewayExecuteMsg, helpers::keccak256_hash, FastTransferOrder,
};

pub fn open(deps: DepsMut, info: MessageInfo, order: OnchainCrossChainOrder) -> ContractResponse {
    let gateway_address = GATEWAY_ADDRESS.load(deps.storage)?;

    let order_data: OrderData = from_json(&order.order_data)?;

    let resolved_order = resolve(order.clone())?;

    let order_id = keccak256_hash(&resolved_order.fill_instructions[0].origin_data);

    let msg = GatewayExecuteMsg::SubmitOrder {
        sender: order_data.sender,
        recipient: order_data.recipient,
        amount_in: order_data.amount_in,
        amount_out: order_data.amount_out,
        destination_domain: order_data.destination_domain,
        timeout_timestamp: order.fill_deadline,
        data: order_data.data,
    };

    let msg = WasmMsg::Execute {
        contract_addr: gateway_address.to_string(),
        msg: to_json_binary(&msg)?,
        funds: info.funds,
    };

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "open")
        .add_attribute("order_id", order_id.to_string())
        .add_attribute("order", to_json_binary(&resolved_order)?.to_string()))
}

pub fn fill(
    deps: DepsMut,
    info: MessageInfo,
    order_id: HexBinary,
    origin_data: Binary,
) -> ContractResponse {
    let gateway_address = GATEWAY_ADDRESS.load(deps.storage)?;

    let order: FastTransferOrder = HexBinary::from(origin_data).into();

    let msg = GatewayExecuteMsg::FillOrder {
        filler: info.sender.clone(),
        order,
    };

    let msg = WasmMsg::Execute {
        contract_addr: gateway_address.to_string(),
        msg: to_json_binary(&msg)?,
        funds: info.funds,
    };

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "fill")
        .add_attribute("order_id", order_id.to_string()))
}
