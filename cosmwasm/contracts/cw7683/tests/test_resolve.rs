use cosmwasm_std::{testing::MockApi, to_json_binary, HexBinary, Uint128};
use cw_7683::{
    contract::GO_FAST_ORDER_TYPE,
    types::{FillInstruction, OnchainCrossChainOrder, OrderData, Output, ResolvedCrossChainOrder},
};
use go_fast::helpers::{bech32_decode, left_pad_bytes};

#[test]
fn test_resolve() {
    let user_address = MockApi::default().addr_make("user");
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
        timeout_timestamp: 1234567890,
        data: None,
    };

    let order_data = to_json_binary(&order).unwrap();

    let onchain_order = OnchainCrossChainOrder {
        fill_deadline: 1234567890,
        order_data_type: GO_FAST_ORDER_TYPE.to_string(),
        order_data: order_data.clone(),
    };

    let expected = ResolvedCrossChainOrder {
        user: user_address_hex.clone(),
        origin_domain: 2,
        open_deadline: 18446744073709551615,
        fill_deadline: 1234567890,
        max_spent: vec![Output {
            token: "uatom".to_string(),
            amount: Uint128::new(98_000_000),
            recipient: user_address_hex.clone(),
            domain: 1,
        }],
        min_received: vec![Output {
            token: "uosmo".to_string(),
            amount: Uint128::new(100_000_000),
            recipient: user_address_hex,
            domain: 2,
        }],
        fill_instructions: vec![FillInstruction {
            destination_domain: 1,
            destination_settler: HexBinary::from_hex(
                "04f8996da763b7a969b1028ee3007569eaf3a635486ddab211d512c85b9df8fb",
            )
            .unwrap(),
            origin_data: HexBinary::from_hex("7b2273656e646572223a2230346638393936646137363362376139363962313032386565333030373536396561663361363335343836646461623231316435313263383562396466386662222c22726563697069656e74223a2230346638393936646137363362376139363962313032386565333030373536396561663361363335343836646461623231316435313263383562396466386662222c22616d6f756e745f696e223a22313030303030303030222c22616d6f756e745f6f7574223a223938303030303030222c226e6f6e6365223a312c22736f757263655f646f6d61696e223a322c2264657374696e6174696f6e5f646f6d61696e223a312c2274696d656f75745f74696d657374616d70223a313233343536373839302c2264617461223a6e756c6c7d").unwrap().into(),
        }],
    };

    let resolved_order = cw_7683::query::resolve(onchain_order).unwrap();

    assert_eq!(expected, resolved_order);
}
