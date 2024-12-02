use cosmwasm_std::{Addr, Deps, HexBinary, Order as ListOrder, StdResult, Storage};
use cw_storage_plus::{Bound, Index, IndexList, IndexedMap, MultiIndex};
use go_fast::gateway::OrderFill;

pub struct Fills<'a> {
    fills: IndexedMap<'a, Vec<u8>, OrderFill, FillIndexes<'a>>,
}

pub struct FillIndexes<'a> {
    pub filler: MultiIndex<'a, Addr, OrderFill, Vec<u8>>,
}

#[allow(clippy::needless_lifetimes)]
impl<'a> IndexList<OrderFill> for FillIndexes<'a> {
    fn get_indexes(
        &'_ self,
    ) -> Box<dyn Iterator<Item = &'_ dyn cw_storage_plus::Index<OrderFill>> + '_> {
        let v: Vec<&dyn Index<OrderFill>> = vec![&self.filler];
        Box::new(v.into_iter())
    }
}

impl<'a> Fills<'a> {
    pub fn new(fills_namespace: &'a str, filler_index_namespace: &'a str) -> Self {
        let indexes = FillIndexes {
            filler: MultiIndex::new(
                |_pk, d| d.filler.clone(),
                fills_namespace,
                filler_index_namespace,
            ),
        };

        Self {
            fills: IndexedMap::new(fills_namespace, indexes),
        }
    }

    pub fn create_order_fill(
        &self,
        storage: &mut dyn Storage,
        order_id: HexBinary,
        filler: Addr,
        source_domain: u32,
    ) -> StdResult<OrderFill> {
        let fill = OrderFill {
            order_id: order_id.clone(),
            filler,
            source_domain,
        };

        self.fills.save(storage, order_id.to_vec(), &fill)?;

        Ok(fill)
    }

    pub fn by_order_id(&self, deps: Deps, order_id: HexBinary) -> StdResult<OrderFill> {
        self.fills.load(deps.storage, order_id.to_vec())
    }

    pub fn by_filler(
        &self,
        deps: Deps,
        filler: Addr,
        start_after: Option<Vec<u8>>,
        limit: Option<u32>,
    ) -> StdResult<Vec<OrderFill>> {
        let limit = limit.unwrap_or(10) as usize;
        let start: Option<Bound<Vec<u8>>> = start_after.map(Bound::exclusive);

        let fills = &self
            .fills
            .idx
            .filler
            .prefix(filler)
            .range(deps.storage, None, start, ListOrder::Descending)
            .take(limit)
            .map(|x| x.unwrap().1)
            .collect::<Vec<_>>();

        Ok(fills.clone())
    }
}
