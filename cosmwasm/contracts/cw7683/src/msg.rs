use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, HexBinary};

use crate::types::{OnchainCrossChainOrder, ResolvedCrossChainOrder};

#[cw_serde]
pub struct InstantiateMsg {
    pub gateway_address: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {
    Open {
        order: OnchainCrossChainOrder,
    },
    Fill {
        order_id: HexBinary,
        origin_data: Binary,
        filler_data: Binary,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ResolvedCrossChainOrder)]
    Resolve { order: OnchainCrossChainOrder },
}
