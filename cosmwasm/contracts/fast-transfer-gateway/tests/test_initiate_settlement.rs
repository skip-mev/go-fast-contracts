use common::default_instantiate;
use cosmwasm_std::{testing::mock_info, to_json_binary, Addr, HexBinary, ReplyOn, SubMsg, WasmMsg};
use go_fast::gateway::ExecuteMsg;
use go_fast_transfer_cw::{
    helpers::{bech32_decode, left_pad_bytes},
    state::{self, REMOTE_DOMAINS},
};
use hyperlane::mailbox::{DispatchMsg, ExecuteMsg as MailboxExecuteMsg};

pub mod common;

#[test]
fn test_initiate_settlement() {
    let (mut deps, env) = default_instantiate();

    let solver_address = deps.api.with_prefix("osmo").addr_make("solver");

    let order_id = HexBinary::from_hex("1234").unwrap();

    state::order_fills()
        .create_order_fill(
            deps.as_mut().storage,
            order_id.clone(),
            solver_address.clone(),
            2,
        )
        .unwrap();

    let execute_msg = ExecuteMsg::InitiateSettlement {
        order_ids: vec![order_id],
        repayment_address: HexBinary::from(left_pad_bytes(
            bech32_decode(solver_address.as_str()).unwrap(),
            32,
        )),
    };

    let info = mock_info(solver_address.as_str(), &[]);

    let res = go_fast_transfer_cw::contract::execute(deps.as_mut(), env, info, execute_msg.clone())
        .unwrap();

    // call to hyperlane mailbox contract
    assert_eq!(
        res.messages[0],
        SubMsg {
            id: 0,
            msg: WasmMsg::Execute {
                contract_addr: "mailbox_contract_address".into(),
                msg: to_json_binary(&MailboxExecuteMsg::Dispatch(DispatchMsg {
                    dest_domain: 2,
                    recipient_addr: HexBinary::from_hex(
                        "0000000000000000000000005B16CfB4Fa672d351760a189278406013a61B231",
                    )
                    .unwrap(),
                    msg_body: HexBinary::from_hex(
                        "00b8789db0c2da6b48ff31471423dc7ffa2386902c666fa2691e636c29b539936a1234"
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
fn test_initiate_settlement_multiple_orders() {
    let (mut deps, env) = default_instantiate();

    let solver_address = deps.api.with_prefix("osmo").addr_make("solver");

    let order_id = HexBinary::from_hex("1234").unwrap();
    let order_id2 = HexBinary::from_hex("5678").unwrap();

    state::order_fills()
        .create_order_fill(
            deps.as_mut().storage,
            order_id.clone(),
            solver_address.clone(),
            2,
        )
        .unwrap();

    state::order_fills()
        .create_order_fill(
            deps.as_mut().storage,
            order_id2.clone(),
            solver_address.clone(),
            2,
        )
        .unwrap();

    let execute_msg = ExecuteMsg::InitiateSettlement {
        order_ids: vec![order_id, order_id2],
        repayment_address: HexBinary::from(left_pad_bytes(
            bech32_decode(solver_address.as_str()).unwrap(),
            32,
        )),
    };

    let info = mock_info(solver_address.as_str(), &[]);

    let res = go_fast_transfer_cw::contract::execute(deps.as_mut(), env, info, execute_msg.clone())
        .unwrap();

    // call to hyperlane mailbox contract
    assert_eq!(
        res.messages[0],
        SubMsg {
            id: 0,
            msg: WasmMsg::Execute {
                contract_addr: "mailbox_contract_address".into(),
                msg: to_json_binary(&MailboxExecuteMsg::Dispatch(DispatchMsg {
                    dest_domain: 2,
                    recipient_addr: HexBinary::from_hex(
                        "0000000000000000000000005B16CfB4Fa672d351760a189278406013a61B231",
                    )
                    .unwrap(),
                    msg_body: HexBinary::from_hex(
                        "00b8789db0c2da6b48ff31471423dc7ffa2386902c666fa2691e636c29b539936a12345678"
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
fn test_initiate_settlement_fails_if_sender_is_not_filler() {
    let (mut deps, env) = default_instantiate();

    let solver_address = deps.api.with_prefix("osmo").addr_make("solver");

    let order_id = HexBinary::from_hex("1234").unwrap();

    state::order_fills()
        .create_order_fill(
            deps.as_mut().storage,
            order_id.clone(),
            Addr::unchecked("other_filler"),
            2,
        )
        .unwrap();

    let execute_msg = ExecuteMsg::InitiateSettlement {
        order_ids: vec![order_id],
        repayment_address: HexBinary::from(left_pad_bytes(
            bech32_decode(solver_address.as_str()).unwrap(),
            32,
        )),
    };

    let info = mock_info(solver_address.as_str(), &[]);

    let res = go_fast_transfer_cw::contract::execute(deps.as_mut(), env, info, execute_msg.clone())
        .unwrap_err()
        .to_string();

    assert_eq!(res, "Unauthorized");
}

#[test]
fn test_initiate_settlement_fails_if_sender_is_filler_of_an_order_but_not_all() {
    let (mut deps, env) = default_instantiate();

    let solver_address = deps.api.with_prefix("osmo").addr_make("solver");

    let order_id = HexBinary::from_hex("1234").unwrap();
    let order_id2 = HexBinary::from_hex("5678").unwrap();

    state::order_fills()
        .create_order_fill(
            deps.as_mut().storage,
            order_id.clone(),
            Addr::unchecked("other_filler"),
            2,
        )
        .unwrap();

    state::order_fills()
        .create_order_fill(
            deps.as_mut().storage,
            order_id2.clone(),
            solver_address.clone(),
            2,
        )
        .unwrap();

    let execute_msg = ExecuteMsg::InitiateSettlement {
        order_ids: vec![order_id, order_id2],
        repayment_address: HexBinary::from(left_pad_bytes(
            bech32_decode(solver_address.as_str()).unwrap(),
            32,
        )),
    };

    let info = mock_info(solver_address.as_str(), &[]);

    let res = go_fast_transfer_cw::contract::execute(deps.as_mut(), env, info, execute_msg.clone())
        .unwrap_err()
        .to_string();

    assert_eq!(res, "Unauthorized");
}

#[test]
fn test_initiate_settlement_fails_if_source_domain_is_unknown() {
    let (mut deps, env) = default_instantiate();

    let solver_address = deps.api.with_prefix("osmo").addr_make("solver");

    let order_id = HexBinary::from_hex("1234").unwrap();

    state::order_fills()
        .create_order_fill(
            deps.as_mut().storage,
            order_id.clone(),
            solver_address.clone(),
            3,
        )
        .unwrap();

    let execute_msg = ExecuteMsg::InitiateSettlement {
        order_ids: vec![order_id],
        repayment_address: HexBinary::from(left_pad_bytes(
            bech32_decode(solver_address.as_str()).unwrap(),
            32,
        )),
    };

    let info = mock_info(solver_address.as_str(), &[]);

    let res = go_fast_transfer_cw::contract::execute(deps.as_mut(), env, info, execute_msg.clone())
        .unwrap_err()
        .to_string();

    assert_eq!(res, "Unknown remote domain");
}

#[test]
fn test_initiate_settlement_multiple_orders_fails_if_source_domains_are_different() {
    let (mut deps, env) = default_instantiate();

    REMOTE_DOMAINS
        .save(
            deps.as_mut().storage,
            6,
            &HexBinary::from_hex(
                "0000000000000000000000005B16CfB4Fa672d351760a189278406013a61B231",
            )
            .unwrap(),
        )
        .unwrap();

    let solver_address = deps.api.with_prefix("osmo").addr_make("solver");

    let order_id = HexBinary::from_hex("1234").unwrap();
    let order_id2 = HexBinary::from_hex("5678").unwrap();

    state::order_fills()
        .create_order_fill(
            deps.as_mut().storage,
            order_id.clone(),
            solver_address.clone(),
            2,
        )
        .unwrap();

    state::order_fills()
        .create_order_fill(
            deps.as_mut().storage,
            order_id2.clone(),
            solver_address.clone(),
            6,
        )
        .unwrap();

    let execute_msg = ExecuteMsg::InitiateSettlement {
        order_ids: vec![order_id, order_id2],
        repayment_address: HexBinary::from(left_pad_bytes(
            bech32_decode(solver_address.as_str()).unwrap(),
            32,
        )),
    };

    let info = mock_info(solver_address.as_str(), &[]);

    let res = go_fast_transfer_cw::contract::execute(deps.as_mut(), env, info, execute_msg.clone())
        .unwrap_err()
        .to_string();

    assert_eq!(res, "Source domains must match");
}
