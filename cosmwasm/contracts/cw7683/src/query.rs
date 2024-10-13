use cosmwasm_std::{from_json, to_json_binary};
use go_fast::FastTransferOrder;

use crate::{
    contract::GO_FAST_ORDER_TYPE,
    error::{ContractError, ContractResult},
    types::{FillInstruction, OnchainCrossChainOrder, OrderData, Output, ResolvedCrossChainOrder},
};

pub fn resolve(order: OnchainCrossChainOrder) -> ContractResult<ResolvedCrossChainOrder> {
    if order.order_data_type != GO_FAST_ORDER_TYPE {
        return Err(ContractError::WrongOrderDataType);
    }

    let order_data: OrderData = from_json(order.order_data)?;

    let max_spent = vec![Output {
        token: order_data.output_token,
        amount: order_data.amount_out,
        recipient: order_data.recipient.clone(),
        domain: order_data.destination_domain,
    }];

    let min_received = vec![Output {
        token: order_data.input_token,
        amount: order_data.amount_in,
        recipient: order_data.sender.clone(),
        domain: order_data.source_domain,
    }];

    let order = FastTransferOrder {
        sender: order_data.sender.clone(),
        recipient: order_data.recipient,
        amount_in: order_data.amount_in,
        amount_out: order_data.amount_out,
        nonce: order_data.nonce,
        source_domain: order_data.source_domain,
        destination_domain: order_data.destination_domain,
        timeout_timestamp: order_data.timeout_timestamp,
        data: order_data.data.clone(),
    };

    let fill_instructions = vec![FillInstruction {
        destination_domain: order_data.destination_domain,
        destination_settler: order_data.sender.clone(),
        origin_data: to_json_binary(&order)?,
    }];

    Ok(ResolvedCrossChainOrder {
        user: order_data.sender,
        origin_domain: order_data.source_domain,
        open_deadline: u64::MAX,
        fill_deadline: order.timeout_timestamp,
        max_spent,
        min_received,
        fill_instructions,
    })
}
