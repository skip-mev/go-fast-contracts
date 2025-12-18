use cosmwasm_std::{Coin, StdError};
use cw_ownable::OwnershipError;

#[derive(Debug, thiserror::Error)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Unexpected order id")]
    UnexpectedOrderId,

    #[error(transparent)]
    Ownership(#[from] OwnershipError),

    #[error("Unexpected funds sent. Expected: {expected:?}, Actual: {actual:?}")]
    UnexpectedFunds {
        expected: Vec<Coin>,
        actual: Vec<Coin>,
    },

    #[error("Order already filled")]
    OrderAlreadyFilled,

    #[error("Order timed out")]
    OrderTimedOut,

    #[error("Invalid local domain")]
    InvalidLocalDomain,

    #[error("Unknown remote domain")]
    UnknownRemoteDomain,

    #[error("Source domains must match")]
    SourceDomainsMustMatch,

    #[error("Order not timed out")]
    OrderNotTimedOut,

    #[error("Order not found")]
    OrderNotFound,

    #[error("Incorrect domain for settlement")]
    IncorrectDomainForSettlement,

    #[error("Order recipient cannot be mailbox")]
    OrderRecipientCannotBeMailbox,

    #[error("Duplicate order")]
    DuplicateOrder,

    #[error("Invalid repayment address")]
    InvalidRepaymentAddress,
}

pub type ContractResult<T> = Result<T, ContractError>;
pub type ContractResponse = ContractResult<cosmwasm_std::Response>;
