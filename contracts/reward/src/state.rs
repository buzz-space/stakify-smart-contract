use std::fmt;

use cosmwasm_schema::cw_serde; // attribute macro to (de)serialize and make schemas
use cosmwasm_std::{Addr, Uint128}; // address type
use cw_storage_plus::{Item, Map}; // analog of Singletons for storage

#[cw_serde]
pub enum TokenInfo {
    Token { contract_addr: String },
    NativeToken { denom: String },
}

impl fmt::Display for TokenInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenInfo::NativeToken { denom } => write!(f, "{}", denom),
            TokenInfo::Token { contract_addr } => write!(f, "{}", contract_addr),
        }
    }
}

#[cw_serde]
pub struct AssetToken {
    pub info: TokenInfo,
    pub amount: Uint128,
}

impl fmt::Display for AssetToken {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.info, self.amount)
    }
}

#[cw_serde]
pub struct RewardInfo {
    pub owner: Addr,              // owner of campaign
    pub reward_token: AssetToken, // reward token
    pub total_reward: Uint128,
}

#[cw_serde]
pub struct StakerRewardAssetInfo {
    pub reward_debt: Uint128, // can claim reward.
    pub reward_claimed: Uint128,
}

#[cw_serde]
pub struct WatchListReward {
    pub address: String,
    pub reward_debt: Uint128,
    pub reward_claimed: Uint128,
}

// campaign info
pub const REWARD_INFO: Item<RewardInfo> = Item::new("reward_info");

// watch list
pub const WATCH_LIST: Item<Vec<Addr>> = Item::new("watch_list");

// Mapping from staker address to staked nft.
pub const STAKERS_INFO: Map<Addr, StakerRewardAssetInfo> = Map::new("stakers_info");
