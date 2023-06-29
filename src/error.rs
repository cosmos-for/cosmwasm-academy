use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    StdErr(#[from] StdError),
    #[error("Unauthorized -- Only {owner} can do.")]
    UnauthorizedErr { owner: String },
    #[error("Invalid address {address}")]
    InvalidAddressErr { address: String },
    #[error("Invalid contract to migrate from: {contract}")]
    InvalidName { contract: String },
    #[error("Unsupported contract version for migration: {version}")]
    InvalidVersion { version: String },
}
