use common::default_instantiate;
use cosmwasm_std::{
    testing::mock_info, to_json_binary, HexBinary, ReplyOn, SubMsg, Uint128, WasmMsg,
};
use go_fast::{gateway::ExecuteMsg, helpers::keccak256_hash, FastTransferOrder};
use go_fast_transfer_cw::{
    helpers::{bech32_decode, bech32_encode, left_pad_bytes},
    state::{self},
};
use hyperlane::mailbox::{DispatchMsg, ExecuteMsg as MailboxExecuteMsg};

pub mod common;

#[test]
fn test_initiate_timeout() {
    let (mut deps, env) = default_instantiate();

    let solver_address = deps.api.with_prefix("osmo").addr_make("solver");

    let user_address = deps.api.with_prefix("osmo").addr_make("user");

    let order_a = FastTransferOrder {
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

    let order_b = FastTransferOrder {
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

    let execute_msg = ExecuteMsg::InitiateTimeout {
        orders: vec![order_a, order_b],
    };

    let info = mock_info(solver_address.as_str(), &[]);

    let res = go_fast_transfer_cw::contract::execute(deps.as_mut(), env, info, execute_msg.clone())
        .unwrap();

    assert_eq!(
        res.messages[0],
        SubMsg {
            id: 0,
            msg: WasmMsg::Execute {
                contract_addr: bech32_encode(
                    "osmo",
                    &keccak256_hash("mailbox_contract_address".as_bytes()),
                )
                .unwrap()
                .into_string(),
                msg: to_json_binary(&MailboxExecuteMsg::Dispatch(DispatchMsg {
                    dest_domain: 2,
                    recipient_addr: HexBinary::from_hex(
                        "0000000000000000000000005B16CfB4Fa672d351760a189278406013a61B231",
                    )
                    .unwrap(),
                    msg_body: HexBinary::from_hex(
                        "01b52d0eadcef62db278b39caf9af717fb004d9dc610849c083120e1d477c75f8eb52d0eadcef62db278b39caf9af717fb004d9dc610849c083120e1d477c75f8e"
                    )
                    .unwrap(),
                    hook: Some(
                        "hook_contract_address".into()
                    ),
                    metadata: None
                }))
                .unwrap(),
                funds: vec![]
            }
            .into(),
            gas_limit: None,
            reply_on: ReplyOn::Never,
        }
    );
}

#[test]
fn test_initiate_timeout_fails_if_source_domains_dont_match() {
    let (mut deps, env) = default_instantiate();

    let solver_address = deps.api.with_prefix("osmo").addr_make("solver");

    let user_address = deps.api.with_prefix("osmo").addr_make("user");

    let order_a = FastTransferOrder {
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

    let order_b = FastTransferOrder {
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
        timeout_timestamp: env.block.time.seconds() - 1000,
        data: None,
    };

    let execute_msg = ExecuteMsg::InitiateTimeout {
        orders: vec![order_a, order_b],
    };

    let info = mock_info(solver_address.as_str(), &[]);

    let res = go_fast_transfer_cw::contract::execute(deps.as_mut(), env, info, execute_msg.clone())
        .unwrap_err()
        .to_string();

    assert_eq!(res, "Source domains must match");
}

#[test]
fn test_initiate_timeout_fails_if_time_is_before_the_timeout_timestamp() {
    let (mut deps, env) = default_instantiate();

    let solver_address = deps.api.with_prefix("osmo").addr_make("solver");

    let user_address = deps.api.with_prefix("osmo").addr_make("user");

    let order_a = FastTransferOrder {
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

    let order_b = FastTransferOrder {
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

    let execute_msg = ExecuteMsg::InitiateTimeout {
        orders: vec![order_a, order_b],
    };

    let info = mock_info(solver_address.as_str(), &[]);

    let res = go_fast_transfer_cw::contract::execute(deps.as_mut(), env, info, execute_msg.clone())
        .unwrap_err()
        .to_string();

    assert_eq!(res, "Order not timed out");
}

#[test]
fn test_initiate_timeout_fails_if_order_has_been_filled() {
    let (mut deps, env) = default_instantiate();

    let solver_address = deps.api.with_prefix("osmo").addr_make("solver");

    let user_address = deps.api.with_prefix("osmo").addr_make("user");

    let order_a = FastTransferOrder {
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

    let order_b = FastTransferOrder {
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

    state::order_fills()
        .create_order_fill(
            deps.as_mut().storage,
            order_a.id(),
            solver_address.clone(),
            2,
        )
        .unwrap();

    let execute_msg = ExecuteMsg::InitiateTimeout {
        orders: vec![order_a, order_b],
    };

    let info = mock_info(solver_address.as_str(), &[]);

    let res = go_fast_transfer_cw::contract::execute(deps.as_mut(), env, info, execute_msg.clone())
        .unwrap_err()
        .to_string();

    assert_eq!(res, "Order already filled");
}

#[test]
fn test_initiate_timeout_fails_if_source_domain_is_unknown() {
    let (mut deps, env) = default_instantiate();

    let solver_address = deps.api.with_prefix("osmo").addr_make("solver");

    let user_address = deps.api.with_prefix("osmo").addr_make("user");

    let order_a = FastTransferOrder {
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
        timeout_timestamp: env.block.time.seconds() - 1000,
        data: None,
    };

    let execute_msg = ExecuteMsg::InitiateTimeout {
        orders: vec![order_a],
    };

    let info = mock_info(solver_address.as_str(), &[]);

    let res = go_fast_transfer_cw::contract::execute(deps.as_mut(), env, info, execute_msg.clone())
        .unwrap_err()
        .to_string();

    assert_eq!(res, "Unknown remote domain");
}

#[test]
fn test_initiate_timeout_fails_if_destination_domain_is_not_the_local_domain() {
    let (mut deps, env) = default_instantiate();

    let solver_address = deps.api.with_prefix("osmo").addr_make("solver");

    let user_address = deps.api.with_prefix("osmo").addr_make("user");

    let order_a = FastTransferOrder {
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
        timeout_timestamp: env.block.time.seconds() - 1000,
        data: None,
    };

    let order_b = FastTransferOrder {
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

    let execute_msg = ExecuteMsg::InitiateTimeout {
        orders: vec![order_a, order_b],
    };

    let info = mock_info(solver_address.as_str(), &[]);

    let res = go_fast_transfer_cw::contract::execute(deps.as_mut(), env, info, execute_msg.clone())
        .unwrap_err()
        .to_string();

    assert_eq!(res, "Invalid local domain");
}
