use crate::common::default_instantiate;
use common::submit_order;
use cosmwasm_std::{coin, testing::mock_info, BankMsg, HexBinary, ReplyOn, SubMsg, Uint128};
use go_fast::{gateway::ExecuteMsg, FastTransferOrder};
use go_fast_transfer_cw::{
    helpers::{bech32_decode, left_pad_bytes},
    msg::{OrderStatus, TimeoutOrdersMessage},
    state::{ORDER_STATUSES, REMOTE_DOMAINS},
};
use hyperlane::message_recipient::HandleMsg;

pub mod common;

#[test]
fn test_refund_orders() {
    let (mut deps, env) = default_instantiate();

    let user_address = deps.api.with_prefix("osmo").addr_make("user");

    let user_hex = HexBinary::from(left_pad_bytes(
        bech32_decode(user_address.as_str()).unwrap(),
        32,
    ));

    let order_a = FastTransferOrder {
        sender: user_hex.clone(),
        recipient: user_hex.clone(),
        amount_in: Uint128::new(100_000_000),
        amount_out: Uint128::new(98_000_000),
        nonce: 1,
        source_domain: 1,
        destination_domain: 2,
        timeout_timestamp: env.block.time.seconds() + 1000,
        data: None,
    };

    let order_b = FastTransferOrder {
        sender: user_hex.clone(),
        recipient: user_hex.clone(),
        amount_in: Uint128::new(100_000_000),
        amount_out: Uint128::new(98_000_000),
        nonce: 2,
        source_domain: 1,
        destination_domain: 2,
        timeout_timestamp: env.block.time.seconds() + 1000,
        data: None,
    };

    submit_order(
        deps.as_mut(),
        &env,
        &mock_info(
            user_address.as_str(),
            &[coin(order_a.amount_in.u128(), "uusdc")],
        ),
        &order_a,
    )
    .unwrap();

    submit_order(
        deps.as_mut(),
        &env,
        &mock_info(
            user_address.as_str(),
            &[coin(order_b.amount_in.u128(), "uusdc")],
        ),
        &order_b,
    )
    .unwrap();

    let remote_contract = REMOTE_DOMAINS
        .load(deps.as_ref().storage, order_a.destination_domain)
        .unwrap();

    let timeout_orders_message = TimeoutOrdersMessage {
        order_ids: vec![order_a.id(), order_b.id()],
    };

    let info = mock_info("mailbox_contract_address", &[]);

    let execute_msg = ExecuteMsg::Handle(HandleMsg {
        origin: order_a.destination_domain,
        sender: remote_contract,
        body: timeout_orders_message.encode(),
    });

    let res = go_fast_transfer_cw::contract::execute(deps.as_mut(), env.clone(), info, execute_msg)
        .unwrap();

    assert_eq!(
        res.messages[0],
        SubMsg {
            id: 0,
            msg: BankMsg::Send {
                to_address: user_address.clone().into(),
                amount: vec![coin(order_a.amount_in.u128(), "uusdc")],
            }
            .into(),
            gas_limit: None,
            reply_on: ReplyOn::Never,
        }
    );

    assert_eq!(
        res.messages[1],
        SubMsg {
            id: 0,
            msg: BankMsg::Send {
                to_address: user_address.into(),
                amount: vec![coin(order_b.amount_in.u128(), "uusdc")],
            }
            .into(),
            gas_limit: None,
            reply_on: ReplyOn::Never,
        }
    );

    assert_eq!(
        ORDER_STATUSES
            .load(deps.as_ref().storage, order_a.id().to_vec())
            .unwrap(),
        OrderStatus::Refunded
    );

    assert_eq!(
        ORDER_STATUSES
            .load(deps.as_ref().storage, order_b.id().to_vec())
            .unwrap(),
        OrderStatus::Refunded
    );
}

#[test]
fn test_refund_orders_fails_on_unknown_order_id() {
    let (mut deps, env) = default_instantiate();

    let user_address = deps.api.with_prefix("osmo").addr_make("user");

    let user_hex = HexBinary::from(left_pad_bytes(
        bech32_decode(user_address.as_str()).unwrap(),
        32,
    ));

    let order_a = FastTransferOrder {
        sender: user_hex.clone(),
        recipient: user_hex.clone(),
        amount_in: Uint128::new(100_000_000),
        amount_out: Uint128::new(98_000_000),
        nonce: 1,
        source_domain: 1,
        destination_domain: 2,
        timeout_timestamp: env.block.time.seconds() + 1000,
        data: None,
    };

    let order_b = FastTransferOrder {
        sender: user_hex.clone(),
        recipient: user_hex.clone(),
        amount_in: Uint128::new(100_000_000),
        amount_out: Uint128::new(98_000_000),
        nonce: 2,
        source_domain: 1,
        destination_domain: 2,
        timeout_timestamp: env.block.time.seconds() + 1000,
        data: None,
    };

    submit_order(
        deps.as_mut(),
        &env,
        &mock_info(
            user_address.as_str(),
            &[coin(order_a.amount_in.u128(), "uusdc")],
        ),
        &order_a,
    )
    .unwrap();

    let remote_contract = REMOTE_DOMAINS
        .load(deps.as_ref().storage, order_a.destination_domain)
        .unwrap();

    let timeout_orders_message = TimeoutOrdersMessage {
        order_ids: vec![order_a.id(), order_b.id()],
    };

    let info = mock_info("mailbox_contract_address", &[]);

    let execute_msg = ExecuteMsg::Handle(HandleMsg {
        origin: order_a.destination_domain,
        sender: remote_contract,
        body: timeout_orders_message.encode(),
    });

    let res = go_fast_transfer_cw::contract::execute(deps.as_mut(), env.clone(), info, execute_msg)
        .unwrap_err()
        .to_string();

    assert_eq!(res, "Order not found");
}

#[test]
fn test_refund_orders_fails_if_orders_destination_domain_is_not_domain_message_originated_from() {
    let (mut deps, env) = default_instantiate();

    REMOTE_DOMAINS
        .save(
            deps.as_mut().storage,
            3,
            &HexBinary::from_hex(
                "0000000000000000000000005B16CfB4Fa672d351760a189278406013a61B231",
            )
            .unwrap(),
        )
        .unwrap();

    let user_address = deps.api.with_prefix("osmo").addr_make("user");

    let user_hex = HexBinary::from(left_pad_bytes(
        bech32_decode(user_address.as_str()).unwrap(),
        32,
    ));

    let order_a = FastTransferOrder {
        sender: user_hex.clone(),
        recipient: user_hex.clone(),
        amount_in: Uint128::new(100_000_000),
        amount_out: Uint128::new(98_000_000),
        nonce: 1,
        source_domain: 1,
        destination_domain: 2,
        timeout_timestamp: env.block.time.seconds() + 1000,
        data: None,
    };

    let order_b = FastTransferOrder {
        sender: user_hex.clone(),
        recipient: user_hex.clone(),
        amount_in: Uint128::new(100_000_000),
        amount_out: Uint128::new(98_000_000),
        nonce: 2,
        source_domain: 1,
        destination_domain: 3,
        timeout_timestamp: env.block.time.seconds() + 1000,
        data: None,
    };

    submit_order(
        deps.as_mut(),
        &env,
        &mock_info(
            user_address.as_str(),
            &[coin(order_a.amount_in.u128(), "uusdc")],
        ),
        &order_a,
    )
    .unwrap();

    submit_order(
        deps.as_mut(),
        &env,
        &mock_info(
            user_address.as_str(),
            &[coin(order_b.amount_in.u128(), "uusdc")],
        ),
        &order_b,
    )
    .unwrap();

    let remote_contract = REMOTE_DOMAINS
        .load(deps.as_ref().storage, order_a.destination_domain)
        .unwrap();

    let timeout_orders_message = TimeoutOrdersMessage {
        order_ids: vec![order_a.id(), order_b.id()],
    };

    let info = mock_info("mailbox_contract_address", &[]);

    let execute_msg = ExecuteMsg::Handle(HandleMsg {
        origin: order_a.destination_domain,
        sender: remote_contract,
        body: timeout_orders_message.encode(),
    });

    let res = go_fast_transfer_cw::contract::execute(deps.as_mut(), env.clone(), info, execute_msg)
        .unwrap_err()
        .to_string();

    assert_eq!(res, "Incorrect domain for settlement");
}
