use cosmwasm_std::{coin, testing::mock_info, Attribute, HexBinary, Uint128};
use go_fast::gateway::ExecuteMsg;
use go_fast_transfer_cw::{
    helpers::{bech32_decode, left_pad_bytes},
    msg::SettlementDetails,
    state::SETTLEMENT_DETAILS,
};

use crate::common::default_instantiate;

pub mod common;

#[test]
fn test_submit_order() {
    let (mut deps, env) = default_instantiate();

    let user_address = deps.api.with_prefix("osmo").addr_make("user");

    let user_hex = HexBinary::from(left_pad_bytes(
        bech32_decode(user_address.as_str()).unwrap(),
        32,
    ));

    let execute_msg = ExecuteMsg::SubmitOrder {
        sender: user_hex.clone(),
        recipient: user_hex.clone(),
        amount_in: Uint128::new(100_000_000),
        amount_out: Uint128::new(98_000_000),
        destination_domain: 2,
        timeout_timestamp: env.block.time.seconds() + 1000,
        data: None,
    };

    let info = mock_info(
        user_address.as_str(),
        &[coin(Uint128::new(100_000_000).u128(), "uusdc")],
    );

    let res = go_fast_transfer_cw::contract::execute(deps.as_mut(), env.clone(), info, execute_msg)
        .unwrap();

    let order_id: HexBinary = res.data.unwrap().into();

    let attrs = res.attributes.clone();
    assert_eq!(
        attrs,
        vec![
            Attribute::new("order_id", order_id.to_string()),
            Attribute::new("sender", user_hex.to_string()),
            Attribute::new("recipient", user_hex.to_string()),
            Attribute::new("amount_in", Uint128::new(100_000_000).to_string()),
            Attribute::new("amount_out", Uint128::new(98_000_000).to_string()),
            Attribute::new("nonce", "1".to_string()),
            Attribute::new("source_domain", "1".to_string()),
            Attribute::new("destination_domain", "2".to_string()),
            Attribute::new(
                "timeout_timestamp",
                (env.block.time.seconds() + 1000).to_string(),
            ),
            Attribute::new("data", "".to_string()),
        ]
    );

    let stored_settlement_details = SETTLEMENT_DETAILS
        .load(deps.as_ref().storage, order_id.to_vec())
        .unwrap();

    assert_eq!(
        stored_settlement_details,
        SettlementDetails {
            sender: user_hex.clone(),
            nonce: 1,
            destination_domain: 2,
            amount: Uint128::new(100_000_000),
        }
    );
}

#[test]
fn test_submit_order_fails_on_unknown_destination_domain() {
    let (mut deps, env) = default_instantiate();

    let user_address = deps.api.with_prefix("osmo").addr_make("user");

    let user_hex = HexBinary::from(left_pad_bytes(
        bech32_decode(user_address.as_str()).unwrap(),
        32,
    ));

    let execute_msg = ExecuteMsg::SubmitOrder {
        sender: user_hex.clone(),
        recipient: user_hex.clone(),
        amount_in: Uint128::new(100_000_000),
        amount_out: Uint128::new(98_000_000),
        destination_domain: 3,
        timeout_timestamp: env.block.time.seconds() + 1000,
        data: None,
    };

    let info = mock_info(
        user_address.as_str(),
        &[coin(Uint128::new(100_000_000).u128(), "uusdc")],
    );

    let res = go_fast_transfer_cw::contract::execute(deps.as_mut(), env.clone(), info, execute_msg)
        .unwrap_err()
        .to_string();

    assert_eq!(res, "Unknown remote domain");
}

#[test]
fn test_fill_order_fails_on_incorrect_funds() {
    let (mut deps, env) = default_instantiate();

    let user_address = deps.api.with_prefix("osmo").addr_make("user");

    let user_hex = HexBinary::from(left_pad_bytes(
        bech32_decode(user_address.as_str()).unwrap(),
        32,
    ));

    let execute_msg = ExecuteMsg::SubmitOrder {
        sender: user_hex.clone(),
        recipient: user_hex.clone(),
        amount_in: Uint128::new(100_000_000),
        amount_out: Uint128::new(98_000_000),
        destination_domain: 2,
        timeout_timestamp: env.block.time.seconds() + 1000,
        data: None,
    };

    let info = mock_info(
        user_address.as_str(),
        &[coin(Uint128::new(1_000_000).u128(), "uusdc")],
    );

    let res = go_fast_transfer_cw::contract::execute(deps.as_mut(), env.clone(), info, execute_msg)
        .unwrap_err()
        .to_string();

    assert_eq!(res, "Unexpected funds sent. Expected: [Coin { 100000000 \"uusdc\" }], Actual: [Coin { 1000000 \"uusdc\" }]");
}
