use cosmwasm_std::{
    coin, testing::mock_info, to_json_binary, Binary, HexBinary, SubMsg, Uint128, WasmMsg,
};
use cw_7683::msg::ExecuteMsg;
use go_fast::{
    gateway::ExecuteMsg as GatewayExecuteMsg,
    helpers::{bech32_decode, left_pad_bytes},
    FastTransferOrder,
};

pub mod common;
#[test]
fn test_fill() {
    let (mut deps, env) = common::default_instantiate();

    let user_address = deps.api.addr_make("user");
    let user_address_hex = HexBinary::from(left_pad_bytes(
        bech32_decode(user_address.as_str()).unwrap(),
        32,
    ));

    let order = FastTransferOrder {
        sender: user_address_hex.clone(),
        recipient: user_address_hex.clone(),
        amount_in: Uint128::new(100_000_000),
        amount_out: Uint128::new(98_000_000),
        source_domain: 2,
        destination_domain: 1,
        nonce: 1,
        timeout_timestamp: env.block.time.seconds() + 1000,
        data: None,
    };

    let execute_msg = ExecuteMsg::Fill {
        order_id: order.id(),
        origin_data: HexBinary::from(order.clone()).into(),
        filler_data: Binary::default(),
    };

    let info = mock_info("solver", &[coin(order.amount_out.u128(), "uusdc")]);

    let res = cw_7683::contract::execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        execute_msg.clone(),
    )
    .unwrap();

    let expected_msg = SubMsg::new(WasmMsg::Execute {
        contract_addr: "go-fast-gateway".to_string(),
        msg: to_json_binary(&GatewayExecuteMsg::FillOrder {
            filler: info.sender.clone(),
            order,
        })
        .unwrap(),
        funds: info.funds,
    });

    assert_eq!(res.messages.len(), 1);
    assert_eq!(res.messages[0], expected_msg);
}
