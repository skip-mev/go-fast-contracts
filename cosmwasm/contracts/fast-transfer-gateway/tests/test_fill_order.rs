use crate::common::default_instantiate;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    coin, from_json, testing::mock_info, to_json_binary, Addr, BankMsg, HexBinary, ReplyOn, SubMsg,
    Uint128, WasmMsg,
};
use go_fast::{
    gateway::{ExecuteMsg, OrderFill, QueryMsg},
    FastTransferOrder,
};
use go_fast_transfer_cw::{
    helpers::{bech32_decode, left_pad_bytes},
    state::CONFIG,
};

pub mod common;

#[cw_serde]
struct TestMsg {
    pub test: String,
}

#[test]
fn test_fill_order() {
    let (mut deps, env) = default_instantiate();

    let user_address = deps.api.with_prefix("osmo").addr_make("user");

    let order = FastTransferOrder {
        sender: HexBinary::from(left_pad_bytes(
            bech32_decode(user_address.as_str()).unwrap(),
            32,
        )),
        recipient: HexBinary::from(left_pad_bytes(
            bech32_decode(user_address.as_str()).unwrap(),
            32,
        )),
        amount_in: Uint128::new(100_000_000),
        amount_out: Uint128::new(98_000_000),
        nonce: 1,
        source_domain: 2,
        destination_domain: 1,
        timeout_timestamp: env.block.time.seconds() + 1000,
        data: None,
    };

    let execute_msg = ExecuteMsg::FillOrder {
        filler: Addr::unchecked("solver"),
        order: order.clone(),
    };

    let res = go_fast_transfer_cw::contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info("solver", &[coin(order.amount_out.u128(), "uusdc")]),
        execute_msg.clone(),
    )
    .unwrap();

    // assert funds were sent to the orders recipient
    assert_eq!(
        res.messages[0],
        SubMsg {
            id: 0,
            msg: BankMsg::Send {
                to_address: user_address.into(),
                amount: vec![coin(order.amount_out.u128(), "uusdc")],
            }
            .into(),
            gas_limit: None,
            reply_on: ReplyOn::Never,
        }
    );

    let order_fill = go_fast_transfer_cw::contract::query(
        deps.as_ref(),
        env,
        QueryMsg::OrderFill {
            order_id: order.id(),
        },
    )
    .unwrap();
    let order_fill: OrderFill = from_json(&order_fill).unwrap();

    assert_eq!(
        order_fill,
        OrderFill {
            order_id: order.id(),
            filler: Addr::unchecked("solver"),
            source_domain: 2
        }
    );
}

#[test]
fn test_fill_order_with_data() {
    let (mut deps, env) = default_instantiate();

    let user_address = deps.api.with_prefix("osmo").addr_make("user");

    let test_payload = to_json_binary(&TestMsg {
        test: "payload".to_string(),
    })
    .unwrap();

    let order = FastTransferOrder {
        sender: HexBinary::from(left_pad_bytes(
            bech32_decode(user_address.as_str()).unwrap(),
            32,
        )),
        recipient: HexBinary::from(left_pad_bytes(
            bech32_decode(user_address.as_str()).unwrap(),
            32,
        )),
        amount_in: Uint128::new(100_000_000),
        amount_out: Uint128::new(98_000_000),
        nonce: 1,
        source_domain: 2,
        destination_domain: 1,
        timeout_timestamp: env.block.time.seconds() + 1000,
        data: Some(HexBinary::from(test_payload.clone())),
    };

    let execute_msg = ExecuteMsg::FillOrder {
        filler: Addr::unchecked("solver"),
        order: order.clone(),
    };

    let res = go_fast_transfer_cw::contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info("solver", &[coin(order.amount_out.u128(), "uusdc")]),
        execute_msg.clone(),
    )
    .unwrap();

    // assert funds were sent to the orders recipient
    assert_eq!(
        res.messages[0],
        SubMsg {
            id: 0,
            msg: WasmMsg::Execute {
                contract_addr: user_address.into(),
                msg: test_payload,
                funds: vec![coin(order.amount_out.u128(), "uusdc")],
            }
            .into(),
            gas_limit: None,
            reply_on: ReplyOn::Never,
        }
    );

    let order_fill = go_fast_transfer_cw::contract::query(
        deps.as_ref(),
        env,
        QueryMsg::OrderFill {
            order_id: order.id(),
        },
    )
    .unwrap();
    let order_fill: OrderFill = from_json(&order_fill).unwrap();

    assert_eq!(
        order_fill,
        OrderFill {
            order_id: order.id(),
            filler: Addr::unchecked("solver"),
            source_domain: 2
        }
    );
}

#[test]
fn test_fill_order_fails_when_order_recipient_is_mailbox() {
    let (mut deps, env) = default_instantiate();

    let user_address = deps.api.with_prefix("osmo").addr_make("user");

    let test_payload = to_json_binary(&BankMsg::Send {
        to_address: "solver".to_string().into(),
        amount: vec![coin(98_000_000, "uusdc")],
    })
    .unwrap();

    let mailbox_address = CONFIG.load(deps.as_ref().storage).unwrap().mailbox_addr;

    let order = FastTransferOrder {
        sender: HexBinary::from(left_pad_bytes(
            bech32_decode(user_address.as_str()).unwrap(),
            32,
        )),
        recipient: HexBinary::from(left_pad_bytes(
            bech32_decode(mailbox_address.as_str()).unwrap(),
            32,
        )),
        amount_in: Uint128::new(100_000_000),
        amount_out: Uint128::new(98_000_000),
        nonce: 1,
        source_domain: 2,
        destination_domain: 1,
        timeout_timestamp: env.block.time.seconds() + 1000,
        data: Some(HexBinary::from(test_payload.clone())),
    };

    let execute_msg = ExecuteMsg::FillOrder {
        filler: Addr::unchecked("solver"),
        order: order.clone(),
    };

    let res = go_fast_transfer_cw::contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info("solver", &[coin(order.amount_out.u128(), "uusdc")]),
        execute_msg.clone(),
    )
    .unwrap_err()
    .to_string();

    assert_eq!(res, "Order recipient cannot be mailbox");
}

#[test]
fn test_fill_order_fails_on_incorrect_amount() {
    let (mut deps, env) = default_instantiate();

    let user_address = deps.api.with_prefix("osmo").addr_make("user");

    let order = FastTransferOrder {
        sender: HexBinary::from(left_pad_bytes(
            bech32_decode(user_address.as_str()).unwrap(),
            32,
        )),
        recipient: HexBinary::from(left_pad_bytes(
            bech32_decode(user_address.as_str()).unwrap(),
            32,
        )),
        amount_in: Uint128::new(100_000_000),
        amount_out: Uint128::new(98_000_000),
        nonce: 1,
        source_domain: 2,
        destination_domain: 1,
        timeout_timestamp: env.block.time.seconds() + 1000,
        data: None,
    };

    let execute_msg = ExecuteMsg::FillOrder {
        filler: Addr::unchecked("solver"),
        order: order.clone(),
    };

    let res = go_fast_transfer_cw::contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info("solver", &[coin(9_000_000, "uusdc")]),
        execute_msg.clone(),
    )
    .unwrap_err()
    .to_string();

    assert_eq!(
        res,
        "Unexpected funds sent. Expected: [Coin { 98000000 \"uusdc\" }], Actual: [Coin { 9000000 \"uusdc\" }]"
    );
}

#[test]
fn test_fill_order_fails_on_incorrect_denom() {
    let (mut deps, env) = default_instantiate();

    let user_address = deps.api.with_prefix("osmo").addr_make("user");

    let order = FastTransferOrder {
        sender: HexBinary::from(left_pad_bytes(
            bech32_decode(user_address.as_str()).unwrap(),
            32,
        )),
        recipient: HexBinary::from(left_pad_bytes(
            bech32_decode(user_address.as_str()).unwrap(),
            32,
        )),
        amount_in: Uint128::new(100_000_000),
        amount_out: Uint128::new(98_000_000),
        nonce: 1,
        source_domain: 2,
        destination_domain: 1,
        timeout_timestamp: env.block.time.seconds() + 1000,
        data: None,
    };

    let execute_msg = ExecuteMsg::FillOrder {
        filler: Addr::unchecked("solver"),
        order: order.clone(),
    };

    let res = go_fast_transfer_cw::contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info("solver", &[coin(order.amount_out.u128(), "uosmo")]),
        execute_msg.clone(),
    )
    .unwrap_err()
    .to_string();

    assert_eq!(
        res,
        "Unexpected funds sent. Expected: [Coin { 98000000 \"uusdc\" }], Actual: [Coin { 98000000 \"uosmo\" }]"
    );
}

#[test]
fn test_fill_order_fails_on_no_funds() {
    let (mut deps, env) = default_instantiate();

    let user_address = deps.api.with_prefix("osmo").addr_make("user");

    let order = FastTransferOrder {
        sender: HexBinary::from(left_pad_bytes(
            bech32_decode(user_address.as_str()).unwrap(),
            32,
        )),
        recipient: HexBinary::from(left_pad_bytes(
            bech32_decode(user_address.as_str()).unwrap(),
            32,
        )),
        amount_in: Uint128::new(100_000_000),
        amount_out: Uint128::new(98_000_000),
        nonce: 1,
        source_domain: 2,
        destination_domain: 1,
        timeout_timestamp: env.block.time.seconds() + 1000,
        data: None,
    };

    let execute_msg = ExecuteMsg::FillOrder {
        filler: Addr::unchecked("solver"),
        order: order.clone(),
    };

    let res = go_fast_transfer_cw::contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info("solver", &[]),
        execute_msg.clone(),
    )
    .unwrap_err()
    .to_string();

    assert_eq!(
        res,
        "Unexpected funds sent. Expected: [Coin { 98000000 \"uusdc\" }], Actual: []"
    );
}

#[test]
fn test_fill_order_fails_on_incorrect_local_domain() {
    let (mut deps, env) = default_instantiate();

    let user_address = deps.api.with_prefix("osmo").addr_make("user");

    let order = FastTransferOrder {
        sender: HexBinary::from(left_pad_bytes(
            bech32_decode(user_address.as_str()).unwrap(),
            32,
        )),
        recipient: HexBinary::from(left_pad_bytes(
            bech32_decode(user_address.as_str()).unwrap(),
            32,
        )),
        amount_in: Uint128::new(100_000_000),
        amount_out: Uint128::new(98_000_000),
        nonce: 1,
        source_domain: 2,
        destination_domain: 3,
        timeout_timestamp: env.block.time.seconds() + 1000,
        data: None,
    };

    let execute_msg = ExecuteMsg::FillOrder {
        filler: Addr::unchecked("solver"),
        order: order.clone(),
    };

    let res = go_fast_transfer_cw::contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info("solver", &[coin(order.amount_out.u128(), "uusdc")]),
        execute_msg.clone(),
    )
    .unwrap_err()
    .to_string();

    assert_eq!(res, "Invalid local domain");
}

#[test]
fn test_fill_order_fails_on_unknown_remote_domain() {
    let (mut deps, env) = default_instantiate();

    let user_address = deps.api.with_prefix("osmo").addr_make("user");

    let order = FastTransferOrder {
        sender: HexBinary::from(left_pad_bytes(
            bech32_decode(user_address.as_str()).unwrap(),
            32,
        )),
        recipient: HexBinary::from(left_pad_bytes(
            bech32_decode(user_address.as_str()).unwrap(),
            32,
        )),
        amount_in: Uint128::new(100_000_000),
        amount_out: Uint128::new(98_000_000),
        nonce: 1,
        source_domain: 3,
        destination_domain: 1,
        timeout_timestamp: env.block.time.seconds() + 1000,
        data: None,
    };

    let execute_msg = ExecuteMsg::FillOrder {
        filler: Addr::unchecked("solver"),
        order: order.clone(),
    };

    let res = go_fast_transfer_cw::contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info("solver", &[coin(order.amount_out.u128(), "uusdc")]),
        execute_msg.clone(),
    )
    .unwrap_err()
    .to_string();

    assert_eq!(res, "Unknown remote domain");
}

#[test]
pub fn test_fill_order_fails_on_already_filled_order() {
    let (mut deps, env) = default_instantiate();

    let user_address = deps.api.with_prefix("osmo").addr_make("user");

    let order = FastTransferOrder {
        sender: HexBinary::from(left_pad_bytes(
            bech32_decode(user_address.as_str()).unwrap(),
            32,
        )),
        recipient: HexBinary::from(left_pad_bytes(
            bech32_decode(user_address.as_str()).unwrap(),
            32,
        )),
        amount_in: Uint128::new(100_000_000),
        amount_out: Uint128::new(98_000_000),
        nonce: 1,
        source_domain: 2,
        destination_domain: 1,
        timeout_timestamp: env.block.time.seconds() + 1000,
        data: None,
    };

    let execute_msg = ExecuteMsg::FillOrder {
        filler: Addr::unchecked("solver"),
        order: order.clone(),
    };

    go_fast_transfer_cw::contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info("solver", &[coin(order.amount_out.u128(), "uusdc")]),
        execute_msg.clone(),
    )
    .unwrap();

    let res = go_fast_transfer_cw::contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info("solver", &[coin(order.amount_out.u128(), "uusdc")]),
        execute_msg.clone(),
    )
    .unwrap_err()
    .to_string();

    assert_eq!(res, "Order already filled");
}

#[test]
fn test_fill_order_fails_on_timed_out_order() {
    let (mut deps, env) = default_instantiate();

    let user_address = deps.api.with_prefix("osmo").addr_make("user");

    let order = FastTransferOrder {
        sender: HexBinary::from(left_pad_bytes(
            bech32_decode(user_address.as_str()).unwrap(),
            32,
        )),
        recipient: HexBinary::from(left_pad_bytes(
            bech32_decode(user_address.as_str()).unwrap(),
            32,
        )),
        amount_in: Uint128::new(100_000_000),
        amount_out: Uint128::new(98_000_000),
        nonce: 1,
        source_domain: 2,
        destination_domain: 1,
        timeout_timestamp: env.block.time.seconds() - 1000,
        data: None,
    };

    let execute_msg = ExecuteMsg::FillOrder {
        filler: Addr::unchecked("solver"),
        order: order.clone(),
    };

    let res = go_fast_transfer_cw::contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info("solver", &[coin(order.amount_out.u128(), "uusdc")]),
        execute_msg.clone(),
    )
    .unwrap_err()
    .to_string();

    assert_eq!(res, "Order timed out");
}

#[test]
fn test_fill_order_fails_on_timed_out_order_exact() {
    let (mut deps, env) = default_instantiate();

    let user_address = deps.api.with_prefix("osmo").addr_make("user");

    let order = FastTransferOrder {
        sender: HexBinary::from(left_pad_bytes(
            bech32_decode(user_address.as_str()).unwrap(),
            32,
        )),
        recipient: HexBinary::from(left_pad_bytes(
            bech32_decode(user_address.as_str()).unwrap(),
            32,
        )),
        amount_in: Uint128::new(100_000_000),
        amount_out: Uint128::new(98_000_000),
        nonce: 1,
        source_domain: 2,
        destination_domain: 1,
        timeout_timestamp: env.block.time.seconds(),
        data: None,
    };

    let execute_msg = ExecuteMsg::FillOrder {
        filler: Addr::unchecked("solver"),
        order: order.clone(),
    };

    let res = go_fast_transfer_cw::contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info("solver", &[coin(order.amount_out.u128(), "uusdc")]),
        execute_msg.clone(),
    )
    .unwrap_err()
    .to_string();

    assert_eq!(res, "Order timed out");
}
