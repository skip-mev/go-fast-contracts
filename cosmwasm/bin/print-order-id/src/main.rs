use cosmwasm_std::{HexBinary, Uint128};
use go_fast::{helpers::keccak256_hash, FastTransferOrder};

fn main() {
    let order = FastTransferOrder {
        sender: keccak256_hash("order_sender".as_bytes()),
        recipient: keccak256_hash("order_recipient".as_bytes()),
        amount_in: Uint128::new(1_000000),
        amount_out: Uint128::new(2_000000),
        nonce: 5,
        source_domain: 1,
        destination_domain: 2,
        timeout_timestamp: 1234567890,
        data: Some(HexBinary::from("order_data".as_bytes())),
    };

    println!("== Output ==");
    println!("{}", order.id());
}
