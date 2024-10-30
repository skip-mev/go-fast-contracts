use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Attribute, HexBinary, Uint128};
use helpers::keccak256_hash;

pub mod gateway;
pub mod helpers;

#[cw_serde]
pub struct FastTransferOrder {
    pub sender: HexBinary,
    pub recipient: HexBinary,
    pub amount_in: Uint128,
    pub amount_out: Uint128,
    pub nonce: u32,
    pub source_domain: u32,
    pub destination_domain: u32,
    pub timeout_timestamp: u64,
    pub data: Option<HexBinary>,
}

impl FastTransferOrder {
    pub fn id(&self) -> HexBinary {
        let order_bytes = HexBinary::from(self.clone());
        keccak256_hash(&order_bytes)
    }

    pub fn attributes(&self) -> Vec<Attribute> {
        let data = self.data.clone().unwrap_or_default();

        vec![
            Attribute::new("order_id", self.id().to_string()),
            Attribute::new("sender", self.sender.to_string()),
            Attribute::new("recipient", self.recipient.to_string()),
            Attribute::new("amount_in", self.amount_in.to_string()),
            Attribute::new("amount_out", self.amount_out.to_string()),
            Attribute::new("nonce", self.nonce.to_string()),
            Attribute::new("source_domain", self.source_domain.to_string()),
            Attribute::new("destination_domain", self.destination_domain.to_string()),
            Attribute::new("timeout_timestamp", self.timeout_timestamp.to_string()),
            Attribute::new("data", data.to_string()),
        ]
    }
}

impl From<FastTransferOrder> for HexBinary {
    fn from(order: FastTransferOrder) -> Self {
        let data = order.data.unwrap_or_default();

        order
            .sender
            .iter()
            .chain(order.recipient.iter())
            .chain([0u8; 16].iter())
            .chain(order.amount_in.to_be_bytes().iter())
            .chain([0u8; 16].iter())
            .chain(order.amount_out.to_be_bytes().iter())
            .chain(order.nonce.to_be_bytes().iter())
            .chain(order.source_domain.to_be_bytes().iter())
            .chain(order.destination_domain.to_be_bytes().iter())
            .chain(order.timeout_timestamp.to_be_bytes().iter())
            .chain(data.iter())
            .cloned()
            .collect::<Vec<u8>>()
            .into()
    }
}

impl From<HexBinary> for FastTransferOrder {
    fn from(value: HexBinary) -> Self {
        let sender = HexBinary::from(value[0..32].to_vec());
        let recipient = HexBinary::from(value[32..64].to_vec());
        let amount_in = Uint128::new(u128::from_be_bytes(value[80..96].try_into().unwrap()));
        let amount_out = Uint128::new(u128::from_be_bytes(value[112..128].try_into().unwrap()));
        let nonce = u32::from_be_bytes(value[128..132].try_into().unwrap());
        let source_domain = u32::from_be_bytes(value[132..136].try_into().unwrap());
        let destination_domain = u32::from_be_bytes(value[136..140].try_into().unwrap());
        let timeout_timestamp = u64::from_be_bytes(value[140..148].try_into().unwrap());
        let data = if value.len() > 148 {
            Some(HexBinary::from(value[148..].to_vec()))
        } else {
            None
        };

        Self {
            sender,
            recipient,
            amount_in,
            amount_out,
            nonce,
            source_domain,
            destination_domain,
            timeout_timestamp,
            data,
        }
    }
}

#[cfg(test)]
mod tests {
    use helpers::{bech32_decode, left_pad_bytes};

    use super::*;

    #[test]
    fn test_order_encoding() {
        let order = FastTransferOrder {
            sender: HexBinary::from(left_pad_bytes(
                bech32_decode("osmo12pvc4v625ewl34uqqgm3ezw76durxlky5j4guz8kvhal7em3e5wqz7cnla")
                    .unwrap(),
                32,
            )),
            recipient: HexBinary::from(left_pad_bytes(
                bech32_decode("osmo12pvc4v625ewl34uqqgm3ezw76durxlky5j4guz8kvhal7em3e5wqz7cnla")
                    .unwrap(),
                32,
            )),
            amount_in: Uint128::new(100_000_000),
            amount_out: Uint128::new(98_000_000),
            nonce: 1,
            source_domain: 2,
            destination_domain: 1,
            timeout_timestamp: 1234567890,
            data: None,
        };

        let encoded = HexBinary::from(order.clone());

        let decoded = FastTransferOrder::from(encoded);

        assert_eq!(order, decoded);
    }
}
