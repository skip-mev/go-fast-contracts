use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const GATEWAY_ADDRESS: Item<Addr> = Item::new("gateway_address");
