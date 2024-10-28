use cosmwasm_std::{
    coin, to_json_binary, Addr, BankMsg, Binary, CosmosMsg, DepsMut, Env, HexBinary, MessageInfo,
    Response, Uint128, WasmMsg,
};
use cw_ownable::assert_owner;
use go_fast::{gateway::Config, FastTransferOrder};
use hyperlane::mailbox::{get_default_hook, DispatchMsg, ExecuteMsg as MailboxExecuteMsg};

use crate::{
    error::{ContractError, ContractResponse},
    helpers::{
        assert_correct_funds, assert_local_domain, assert_order_is_expired,
        assert_order_is_not_expired, assert_order_not_filled, assert_remote_domain, bech32_encode,
        get_order_settlement_details,
    },
    msg::{Command, OrderStatus, SettleOrdersMessage, SettlementDetails, TimeoutOrdersMessage},
    state::{
        self, next_nonce, CONFIG, LOCAL_DOMAIN, ORDER_STATUSES, REMOTE_DOMAINS, SETTLEMENT_DETAILS,
    },
};

pub fn update_config(deps: DepsMut, info: MessageInfo, config: Config) -> ContractResponse {
    assert_owner(deps.storage, &info.sender)?;

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

pub fn add_remote_domain(
    deps: DepsMut,
    info: MessageInfo,
    domain: u32,
    address: HexBinary,
) -> ContractResponse {
    assert_owner(deps.storage, &info.sender)?;

    REMOTE_DOMAINS.save(deps.storage, domain, &address)?;

    Ok(Response::default())
}

pub fn fill_order(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    filler: Addr,
    order: FastTransferOrder,
) -> ContractResponse {
    let config = CONFIG.load(deps.storage)?;

    assert_order_is_not_expired(&env, &order)?;

    assert_correct_funds(&info, config.token_denom.as_str(), order.amount_out)?;

    assert_local_domain(deps.as_ref(), order.destination_domain)?;
    assert_remote_domain(deps.as_ref(), order.source_domain)?;

    let order_id = order.id();
    assert_order_not_filled(deps.as_ref(), order_id.clone())?;

    let recipient_address = bech32_encode(&config.address_prefix, &order.recipient)?;

    if recipient_address == config.mailbox_addr {
        return Err(ContractError::OrderRecipientCannotBeMailbox);
    }

    let msg: CosmosMsg = match order.data {
        Some(data) => WasmMsg::Execute {
            contract_addr: recipient_address.clone().to_string(),
            msg: Binary::from(data),
            funds: info.funds,
        }
        .into(),
        None => BankMsg::Send {
            to_address: recipient_address.into(),
            amount: info.funds,
        }
        .into(),
    };

    state::order_fills().create_order_fill(deps.storage, order_id, filler, order.source_domain)?;

    Ok(Response::new().add_message(msg))
}

pub fn initiate_settlement(
    deps: DepsMut,
    info: MessageInfo,
    order_ids: Vec<HexBinary>,
    repayment_address: HexBinary,
) -> ContractResponse {
    let config = CONFIG.load(deps.storage)?;

    let mut fills_to_settle = Vec::new();

    for order_id in &order_ids {
        let order_fill = state::order_fills().by_order_id(deps.as_ref(), order_id.clone())?;
        if order_fill.filler != info.sender {
            return Err(ContractError::Unauthorized);
        }

        if fills_to_settle.contains(&order_fill) {
            return Err(ContractError::DuplicateOrder);
        }

        fills_to_settle.push(order_fill);
    }

    let source_domain = fills_to_settle[0].source_domain;
    if !fills_to_settle
        .iter()
        .all(|fill| fill.source_domain == source_domain)
    {
        return Err(ContractError::SourceDomainsMustMatch);
    }

    let remote_contract_address = REMOTE_DOMAINS.may_load(deps.storage, source_domain)?;
    if remote_contract_address.is_none() {
        return Err(ContractError::UnknownRemoteDomain);
    }

    let remote_contract_address = remote_contract_address.unwrap();

    let default_hook = get_default_hook(deps.as_ref(), config.mailbox_addr.clone())?;

    let settle_orders_message = SettleOrdersMessage {
        repayment_address,
        order_ids,
    };

    let msg = WasmMsg::Execute {
        contract_addr: config.mailbox_addr.clone(),
        msg: to_json_binary(&MailboxExecuteMsg::Dispatch(DispatchMsg {
            dest_domain: source_domain,
            recipient_addr: remote_contract_address.clone(),
            msg_body: settle_orders_message.encode(),
            hook: Some(default_hook),
            metadata: None,
        }))?,
        funds: info.funds,
    };

    Ok(Response::new().add_message(msg))
}

pub fn initiate_timeout(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    orders: Vec<FastTransferOrder>,
) -> ContractResponse {
    let config = CONFIG.load(deps.storage)?;

    for order in &orders {
        assert_order_is_expired(&env, order)?;
        assert_order_not_filled(deps.as_ref(), order.id())?;
        assert_local_domain(deps.as_ref(), order.destination_domain)?;
    }

    let order_ids = orders
        .iter()
        .map(|order| order.id())
        .collect::<Vec<HexBinary>>();

    let source_domain = orders[0].source_domain;
    if !orders
        .iter()
        .all(|order| order.source_domain == source_domain)
    {
        return Err(ContractError::SourceDomainsMustMatch);
    }

    let remote_contract_address = REMOTE_DOMAINS.may_load(deps.storage, source_domain)?;
    if remote_contract_address.is_none() {
        return Err(ContractError::UnknownRemoteDomain);
    }

    let remote_contract_address = remote_contract_address.unwrap();

    let default_hook = get_default_hook(deps.as_ref(), config.mailbox_addr.clone())?;

    let timeout_orders_message = TimeoutOrdersMessage { order_ids };

    let msg = WasmMsg::Execute {
        contract_addr: config.mailbox_addr.clone(),
        msg: to_json_binary(&MailboxExecuteMsg::Dispatch(DispatchMsg {
            dest_domain: source_domain,
            recipient_addr: remote_contract_address.clone(),
            msg_body: timeout_orders_message.encode(),
            hook: Some(default_hook),
            metadata: None,
        }))?,
        funds: info.funds,
    };

    Ok(Response::new().add_message(msg))
}

#[allow(clippy::too_many_arguments)]
pub fn submit_order(
    deps: DepsMut,
    info: MessageInfo,
    sender: HexBinary,
    recipient: HexBinary,
    amount_in: Uint128,
    amount_out: Uint128,
    destination_domain: u32,
    timeout_timestamp: u64,
    data: Option<HexBinary>,
) -> ContractResponse {
    let config = CONFIG.load(deps.storage)?;

    assert_correct_funds(&info, config.token_denom.as_str(), amount_in)?;
    assert_remote_domain(deps.as_ref(), destination_domain)?;

    let local_domain = LOCAL_DOMAIN.load(deps.storage)?;
    let nonce = next_nonce(deps.storage)?;

    let order = FastTransferOrder {
        sender: sender.clone(),
        recipient,
        amount_in,
        amount_out,
        nonce,
        source_domain: local_domain,
        destination_domain,
        timeout_timestamp,
        data,
    };

    SETTLEMENT_DETAILS.save(
        deps.storage,
        order.id().to_vec(),
        &SettlementDetails {
            sender,
            nonce,
            destination_domain,
            amount: amount_in,
        },
    )?;

    Ok(Response::new()
        .set_data(order.id())
        .add_attributes(order.attributes()))
}

pub fn handle(
    deps: DepsMut,
    info: MessageInfo,
    msg: hyperlane::message_recipient::HandleMsg,
) -> ContractResponse {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.mailbox_addr {
        return Err(ContractError::Unauthorized);
    }

    let remote_contract = REMOTE_DOMAINS.load(deps.storage, msg.origin)?;
    if msg.sender != remote_contract {
        return Err(ContractError::Unauthorized);
    }

    let command: Command = msg.body.try_into().unwrap();

    match command {
        Command::SettleOrders(settle_orders_message) => {
            settle_orders(deps, msg.origin, settle_orders_message)
        }
        Command::TimeoutOrders(timeout_orders_message) => {
            refund_orders(deps, msg.origin, timeout_orders_message)
        }
    }
}

pub fn settle_orders(
    deps: DepsMut,
    msg_origin_domain: u32,
    msg: SettleOrdersMessage,
) -> ContractResponse {
    let config = CONFIG.load(deps.storage)?;

    let mut amount_to_repay = Uint128::new(0);
    let mut attrs = Vec::new();

    for order_id in msg.order_ids {
        let status = ORDER_STATUSES
            .load(deps.storage, order_id.to_vec())
            .unwrap_or_default();

        if status != OrderStatus::Unfilled {
            attrs.push(("action".to_string(), "order_already_settled".to_string()));
            attrs.push(("order_id".to_string(), order_id.to_string()));
            continue;
        }

        let order_settlement_details = get_order_settlement_details(deps.storage, &order_id)?;
        if order_settlement_details.destination_domain != msg_origin_domain {
            return Err(ContractError::IncorrectDomainForSettlement);
        }

        amount_to_repay += order_settlement_details.amount;

        ORDER_STATUSES.save(deps.storage, order_id.to_vec(), &OrderStatus::Filled)?;
        attrs.push(("action".to_string(), "order_settled".to_string()));
        attrs.push(("order_id".to_string(), order_id.to_string()));
    }

    let repayment_address = bech32_encode(&config.address_prefix, &msg.repayment_address)?;

    let msg = BankMsg::Send {
        to_address: repayment_address.into(),
        amount: vec![coin(amount_to_repay.u128(), config.token_denom)],
    };

    Ok(Response::new().add_message(msg).add_attributes(attrs))
}

pub fn refund_orders(
    deps: DepsMut,
    msg_origin_domain: u32,
    msg: TimeoutOrdersMessage,
) -> ContractResponse {
    let config = CONFIG.load(deps.storage)?;

    let mut attrs = Vec::new();
    let mut msgs = Vec::new();

    for order_id in msg.order_ids {
        let status = ORDER_STATUSES
            .load(deps.storage, order_id.to_vec())
            .unwrap_or_default();

        if status != OrderStatus::Unfilled {
            continue;
        }

        let order_settlement_details = get_order_settlement_details(deps.storage, &order_id)?;
        if order_settlement_details.destination_domain != msg_origin_domain {
            return Err(ContractError::IncorrectDomainForSettlement);
        }

        let sender = bech32_encode(&config.address_prefix, &order_settlement_details.sender)?;

        let msg = BankMsg::Send {
            to_address: sender.into(),
            amount: vec![coin(
                order_settlement_details.amount.u128(),
                config.token_denom.clone(),
            )],
        };

        msgs.push(msg);
        attrs.push(("action".to_string(), "order_refunded".to_string()));
        attrs.push(("order_id".to_string(), order_id.to_string()));
        ORDER_STATUSES.save(deps.storage, order_id.to_vec(), &OrderStatus::Refunded)?;
    }

    Ok(Response::new().add_messages(msgs).add_attributes(attrs))
}
