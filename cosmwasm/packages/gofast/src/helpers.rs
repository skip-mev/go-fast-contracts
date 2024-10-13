use cosmwasm_std::{HexBinary, StdError, StdResult};

pub fn bech32_decode(target: &str) -> StdResult<Vec<u8>> {
    let (_, addr_bytes) = bech32::decode(target)
        .map_err(|e| StdError::generic_err(format!("invalid bech32 bytes. err: {e}")))?;

    Ok(addr_bytes)
}

pub fn keccak256_hash(bz: &[u8]) -> HexBinary {
    use sha3::{Digest, Keccak256};

    let mut hasher = Keccak256::new();
    hasher.update(bz);
    let hash = hasher.finalize().to_vec();

    hash.into()
}

pub fn left_pad_bytes(bytes: Vec<u8>, length: usize) -> Vec<u8> {
    let mut padded = vec![0u8; length];
    let start = length - bytes.len();
    padded[start..].copy_from_slice(&bytes);
    padded
}
