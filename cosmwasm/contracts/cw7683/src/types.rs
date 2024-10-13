use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, HexBinary, Uint128};

#[cw_serde]
pub struct OnchainCrossChainOrder {
    pub fill_deadline: u64,
    pub order_data_type: String,
    pub order_data: Binary,
}

#[cw_serde]
pub struct ResolvedCrossChainOrder {
    pub user: HexBinary,
    pub origin_domain: u32,
    pub open_deadline: u64,
    pub fill_deadline: u64,
    pub max_spent: Vec<Output>,
    pub min_received: Vec<Output>,
    pub fill_instructions: Vec<FillInstruction>,
}

#[cw_serde]
pub struct Output {
    pub token: String,
    pub amount: Uint128,
    pub recipient: HexBinary,
    pub domain: u32,
}

#[cw_serde]
pub struct FillInstruction {
    pub destination_domain: u32,
    pub destination_settler: HexBinary,
    pub origin_data: Binary,
}

#[cw_serde]
pub struct OrderData {
    pub sender: HexBinary,
    pub recipient: HexBinary,
    pub input_token: String,
    pub output_token: String,
    pub amount_in: Uint128,
    pub amount_out: Uint128,
    pub source_domain: u32,
    pub destination_domain: u32,
    pub nonce: u32,
    pub timeout_timestamp: u64,
    pub data: Option<HexBinary>,
}
