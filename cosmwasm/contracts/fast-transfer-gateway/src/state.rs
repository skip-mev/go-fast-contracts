use cosmwasm_std::{HexBinary, StdResult, Storage};
use cw_storage_plus::{Item, Map};
use go_fast::gateway::Config;

use crate::{
    fills::Fills,
    msg::{OrderStatus, SettlementDetails},
};

pub const NONCE: Item<u32> = Item::new("nonce");

pub const LOCAL_DOMAIN: Item<u32> = Item::new("local_domain");

pub const REMOTE_DOMAINS: Map<u32, HexBinary> = Map::new("remote_domains");

pub const SETTLEMENT_DETAILS: Map<Vec<u8>, SettlementDetails> = Map::new("settlement_details");
pub const ORDER_STATUSES: Map<Vec<u8>, OrderStatus> = Map::new("order_statuses");

pub const CONFIG: Item<Config> = Item::new("config");

pub fn order_fills() -> Fills<'static> {
    Fills::new("fills", "filler_index")
}

pub fn next_nonce(storage: &mut dyn Storage) -> StdResult<u32> {
    let nonce = NONCE.load(storage)?;
    let new_nonce = nonce + 1;
    NONCE.save(storage, &new_nonce)?;
    Ok(new_nonce)
}
