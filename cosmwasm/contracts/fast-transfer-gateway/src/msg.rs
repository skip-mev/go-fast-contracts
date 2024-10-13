use std::vec;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{HexBinary, Uint128};

#[cw_serde]
#[derive(Default)]
pub enum OrderStatus {
    #[default]
    Unfilled,
    Filled,
    Refunded,
}

#[cw_serde]
pub enum Command {
    SettleOrders(SettleOrdersMessage),
    TimeoutOrders(TimeoutOrdersMessage),
}

impl TryInto<Command> for HexBinary {
    type Error = String;

    fn try_into(self) -> Result<Command, Self::Error> {
        let command_bytes = self.to_vec();

        let command_type = command_bytes[0];

        match command_type {
            0 => {
                let repayment_address = command_bytes[1..33].to_vec();

                let mut order_ids: Vec<HexBinary> = vec![];

                for i in (33..command_bytes.len()).step_by(32) {
                    let order_id = command_bytes[i..i + 32].to_vec();
                    order_ids.push(order_id.into());
                }

                Ok(Command::SettleOrders(SettleOrdersMessage {
                    order_ids,
                    repayment_address: repayment_address.into(),
                }))
            }
            1 => {
                let mut order_ids: Vec<HexBinary> = vec![];

                for i in (1..command_bytes.len()).step_by(32) {
                    let order_id = command_bytes[i..i + 32].to_vec();
                    order_ids.push(order_id.into());
                }

                Ok(Command::TimeoutOrders(TimeoutOrdersMessage { order_ids }))
            }
            _ => Err(format!("Invalid command type: {}", command_type)),
        }
    }
}

#[cw_serde]
pub struct SettlementDetails {
    pub sender: HexBinary,
    pub nonce: u32,
    pub destination_domain: u32,
    pub amount: Uint128,
}

#[cw_serde]
pub struct SettleOrdersMessage {
    pub order_ids: Vec<HexBinary>,
    pub repayment_address: HexBinary,
}

impl SettleOrdersMessage {
    pub fn encode(&self) -> HexBinary {
        [0u8]
            .iter()
            .chain(self.repayment_address.iter())
            .chain(self.order_ids.iter().flat_map(|id| id.iter()))
            .cloned()
            .collect::<Vec<u8>>()
            .into()
    }
}

#[cw_serde]
pub struct TimeoutOrdersMessage {
    pub order_ids: Vec<HexBinary>,
}

impl TimeoutOrdersMessage {
    pub fn encode(&self) -> HexBinary {
        [1u8]
            .iter()
            .chain(self.order_ids.iter().flat_map(|id| id.iter()))
            .cloned()
            .collect::<Vec<u8>>()
            .into()
    }
}
