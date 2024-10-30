use crate::{
    error::{ContractError, ContractResult},
    msg::SettlementDetails,
    state::{self, LOCAL_DOMAIN, REMOTE_DOMAINS, SETTLEMENT_DETAILS},
};
use bech32::{Bech32, Hrp};
use cosmwasm_std::{
    coins, Addr, Deps, Env, HexBinary, MessageInfo, StdError, StdResult, Storage, Timestamp,
    Uint128,
};
use go_fast::FastTransferOrder;

/// Asserts that exactly `amount` of `denom` is sent to the contract, with no
/// extra funds.
pub fn assert_correct_funds(
    info: &MessageInfo,
    denom: &str,
    amount: Uint128,
) -> ContractResult<()> {
    if info.funds.len() != 1 || info.funds[0].denom != denom || info.funds[0].amount != amount {
        return Err(ContractError::UnexpectedFunds {
            expected: coins(amount.u128(), denom),
            actual: info.funds.clone(),
        });
    }

    Ok(())
}

pub fn assert_order_not_filled(deps: Deps, order_id: HexBinary) -> ContractResult<()> {
    if state::order_fills()
        .by_order_id(deps, order_id.clone())
        .is_ok()
    {
        return Err(ContractError::OrderAlreadyFilled);
    }

    Ok(())
}

pub fn assert_order_is_expired(env: &Env, order: &FastTransferOrder) -> ContractResult<()> {
    let timeout_timestamp = Timestamp::from_seconds(order.timeout_timestamp);
    if env.block.time.seconds() < timeout_timestamp.seconds() {
        return Err(ContractError::OrderNotTimedOut);
    }

    Ok(())
}

pub fn assert_order_is_not_expired(env: &Env, order: &FastTransferOrder) -> ContractResult<()> {
    let timeout_timestamp = Timestamp::from_seconds(order.timeout_timestamp);
    if env.block.time.seconds() >= timeout_timestamp.seconds() {
        return Err(ContractError::OrderTimedOut);
    }

    Ok(())
}

pub fn assert_local_domain(deps: Deps, domain: u32) -> ContractResult<()> {
    if domain != LOCAL_DOMAIN.load(deps.storage)? {
        return Err(ContractError::InvalidLocalDomain);
    }

    Ok(())
}

pub fn assert_remote_domain(deps: Deps, domain: u32) -> ContractResult<()> {
    if !REMOTE_DOMAINS.has(deps.storage, domain) {
        return Err(ContractError::UnknownRemoteDomain);
    }

    Ok(())
}

pub fn bech32_decode(target: &str) -> StdResult<Vec<u8>> {
    let (_, addr_bytes) = bech32::decode(target)
        .map_err(|e| StdError::generic_err(format!("invalid bech32 bytes. err: {e}")))?;

    Ok(addr_bytes)
}

pub fn bech32_encode(hrp: &str, raw_addr: &[u8]) -> StdResult<Addr> {
    if raw_addr.len() != 32 && raw_addr.len() != 20 {
        return Err(StdError::generic_err(format!(
            "invalid raw address length. expected: 32 or 20. got: {}",
            raw_addr.len()
        )));
    }

    if raw_addr.len() == 32 {
        let mut bz = 0u128.to_be_bytes();
        bz[4..].copy_from_slice(&raw_addr[0..12]);
        let check = u128::from_be_bytes(bz);

        if check == 0 {
            return bech32_encode(hrp, &raw_addr[12..]);
        }
    }

    let enc_addr = bech32::encode::<Bech32>(Hrp::parse_unchecked(hrp), raw_addr)
        .map_err(|e| StdError::generic_err(format!("invalid bech32 address. err: {e}")))?;

    Ok(Addr::unchecked(enc_addr))
}

pub fn left_pad_bytes(bytes: Vec<u8>, length: usize) -> Vec<u8> {
    let mut padded = vec![0u8; length];
    let start = length - bytes.len();
    padded[start..].copy_from_slice(&bytes);
    padded
}

pub fn encode_settle_order_data(
    repayment_address: HexBinary,
    order_ids: Vec<HexBinary>,
) -> HexBinary {
    repayment_address
        .iter()
        .chain(order_ids.iter().flat_map(|id| id.iter()))
        .cloned()
        .collect::<Vec<u8>>()
        .into()
}

pub fn keccak256_hash(bz: &[u8]) -> HexBinary {
    use sha3::{Digest, Keccak256};

    let mut hasher = Keccak256::new();
    hasher.update(bz);
    let hash = hasher.finalize().to_vec();

    hash.into()
}

pub fn get_order_settlement_details(
    storage: &dyn Storage,
    order_id: &HexBinary,
) -> ContractResult<SettlementDetails> {
    SETTLEMENT_DETAILS
        .load(storage, order_id.to_vec())
        .map_err(|err| match err {
            StdError::NotFound { .. } => ContractError::OrderNotFound,
            _ => ContractError::Std(err),
        })
}
