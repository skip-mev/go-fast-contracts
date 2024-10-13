use cosmwasm_std::{coin, testing::mock_info, to_json_binary, HexBinary, SubMsg, Uint128, WasmMsg};
use cw_7683::{
    contract::GO_FAST_ORDER_TYPE,
    msg::ExecuteMsg,
    types::{OnchainCrossChainOrder, OrderData},
};
use go_fast::{
    gateway::ExecuteMsg as GatewayExecuteMsg,
    helpers::{bech32_decode, left_pad_bytes},
};

pub mod common;

#[test]
fn test_open() {
    let (mut deps, env) = common::default_instantiate();

    let user_address = deps.api.addr_make("user");
    let user_address_hex = HexBinary::from(left_pad_bytes(
        bech32_decode(user_address.as_str()).unwrap(),
        32,
    ));

    let input_token = "uosmo";
    let output_token = "uatom";

    let order = OrderData {
        sender: user_address_hex.clone(),
        recipient: user_address_hex.clone(),
        input_token: input_token.to_string(),
        output_token: output_token.to_string(),
        amount_in: Uint128::new(100_000_000),
        amount_out: Uint128::new(98_000_000),
        source_domain: 2,
        destination_domain: 1,
        nonce: 1,
        timeout_timestamp: env.block.time.seconds() + 1000,
        data: None,
    };

    let order_data = to_json_binary(&order).unwrap();

    let msg = ExecuteMsg::Open {
        order: OnchainCrossChainOrder {
            fill_deadline: env.block.time.seconds() + 1000,
            order_data_type: GO_FAST_ORDER_TYPE.to_string(),
            order_data,
        },
    };

    let info = mock_info(
        user_address.as_str(),
        &[coin(order.amount_in.u128(), input_token)],
    );

    let res = cw_7683::contract::execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    let expected_msg = SubMsg::new(WasmMsg::Execute {
        contract_addr: "go-fast-gateway".to_string(),
        msg: to_json_binary(&GatewayExecuteMsg::SubmitOrder {
            sender: user_address_hex.clone(),
            recipient: user_address_hex.clone(),
            amount_in: order.amount_in,
            amount_out: order.amount_out,
            destination_domain: order.destination_domain,
            timeout_timestamp: order.timeout_timestamp,
            data: None,
        })
        .unwrap(),
        funds: info.funds,
    });

    assert_eq!(res.messages.len(), 1);
    assert_eq!(res.messages[0], expected_msg);
}
