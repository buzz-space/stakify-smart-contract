use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("##Unauthorized##")]
    Unauthorized {},

    #[error("## You are not the owner of this NFT ##")]
    NotOwner { token_id: String },

    #[error("## Invalid Token ##")]
    InvalidToken {},

    #[error("## Invalid address in watch list ##")]
    InvalidWatchList {},
}
