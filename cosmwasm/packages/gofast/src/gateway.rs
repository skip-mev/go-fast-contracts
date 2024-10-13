use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, HexBinary, Uint128};

use crate::FastTransferOrder;

#[cw_serde]
pub struct Config {
    pub token_denom: String,
    pub address_prefix: String,
    pub mailbox_addr: String,
}

#[cw_serde]
pub struct RemoteDomain {
    pub domain: u32,
    pub address: HexBinary,
}

#[cw_serde]
pub struct OrderFill {
    pub order_id: HexBinary,
    pub filler: Addr,
    pub source_domain: u32,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub token_denom: String,
    pub address_prefix: String,
    pub mailbox_addr: String,
    pub local_domain: u32,
}

#[cw_serde]
pub enum ExecuteMsg {
    FillOrder {
        filler: Addr,
        order: FastTransferOrder,
    },
    InitiateSettlement {
        order_ids: Vec<HexBinary>,
        repayment_address: HexBinary,
    },
    InitiateTimeout {
        orders: Vec<FastTransferOrder>,
    },
    UpdateConfig {
        config: Config,
    },
    AddRemoteDomain {
        domain: u32,
        address: HexBinary,
    },
    SubmitOrder {
        sender: HexBinary,
        recipient: HexBinary,
        amount_in: Uint128,
        amount_out: Uint128,
        destination_domain: u32,
        timeout_timestamp: u64,
        data: Option<HexBinary>,
    },
    Handle(hyperlane::message_recipient::HandleMsg),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    Config {},

    #[returns(u32)]
    LocalDomain {},

    #[returns(HexBinary)]
    RemoteDomain { domain: u32 },

    #[returns(Vec<RemoteDomain>)]
    RemoteDomains {},

    #[returns(OrderFill)]
    OrderFill { order_id: HexBinary },

    #[returns(Vec<OrderFill>)]
    OrderFillsByFiller {
        filler: Addr,
        start_after: Option<HexBinary>,
        limit: Option<u32>,
    },

    #[returns(Vec<Coin>)]
    QuoteInitiateSettlement {
        order_ids: Vec<HexBinary>,
        repayment_address: HexBinary,
        source_domain: u32,
    },
}
