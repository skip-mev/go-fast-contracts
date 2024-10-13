use cosmwasm_std::StdError;

#[derive(Debug, thiserror::Error)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Wrong order data type")]
    WrongOrderDataType,
}

pub type ContractResult<T> = Result<T, ContractError>;
pub type ContractResponse = ContractResult<cosmwasm_std::Response>;
