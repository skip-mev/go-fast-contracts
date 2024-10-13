use cosmwasm_schema::cw_serde;
use cosmwasm_std::HexBinary;

#[cw_serde]
#[derive(Default)]
pub struct HandleMsg {
    pub origin: u32,
    pub sender: HexBinary,
    pub body: HexBinary,
}
