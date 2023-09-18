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
pub struct LockupTerm {
    pub value: u64,
    pub percent: Uint128,
}

impl fmt::Display for LockupTerm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.value, self.percent)
    }
}

#[cw_serde]
pub struct CampaignInfo {
    pub owner: Addr, // owner of campaign
    // info detail
    pub campaign_name: String,
    pub campaign_image: String,
    pub campaign_description: String,
    pub total_reward_claimed: Uint128, // default 0
    pub total_reward: Uint128,         // default 0
    pub limit_per_staker: u64,         // max nft can stake
    pub reward_token: AssetToken,      // reward token
    pub allowed_collection: Addr,      // staking collection nft
    pub lockup_term: Vec<LockupTerm>,
    pub reward_per_second: Uint128,
    pub start_time: u64, // start time must be from T + 1
    pub end_time: u64,   // max 3 years
}

#[cw_serde]
pub struct StakerRewardAssetInfo {
    pub token_ids: Vec<String>,
    pub reward_debt: Uint128, // can claim reward.
    pub reward_claimed: Uint128,
}

#[cw_serde]
pub struct NftInfo {
    pub token_id: String,
    pub owner: Addr,
    pub pending_reward: Uint128,
    pub lockup_term: LockupTerm, // value = seconds
    pub is_end_reward: bool,
    pub start_time: u64,
    pub time_calc: u64,
    pub end_time: u64,
}

#[cw_serde]
pub struct NftStake {
    pub token_id: String,
    pub lockup_term: u64,
}

#[cw_serde]
pub struct RewardRate {
    pub timestamp: u64,
    pub rate: u64,
}

impl fmt::Display for RewardRate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.timestamp, self.rate)
    }
}

// campaign info
pub const CAMPAIGN_INFO: Item<CampaignInfo> = Item::new("campaign_info");

// Mapping from staker address to staked nft.
pub const STAKERS_INFO: Map<Addr, StakerRewardAssetInfo> = Map::new("stakers_info");

// list token_id nft
pub const TOKEN_IDS: Map<String, Vec<String>> = Map::new("token_ids");

// list nft staked
pub const NFTS: Map<String, NftInfo> = Map::new("nfts");

pub const TERM_REWARD_RATES: Map<String, Vec<RewardRate>> = Map::new("term_reward_rates");
pub const TOTAL_STAKING_BY_TERM: Map<String, u64> = Map::new("total_staking_by_term");
pub const TERM_EXPIRATION_TIMES: Map<String, Vec<u64>> = Map::new("expiration_times");

pub const PREVIOUS_TOTAL_REWARD: Item<Uint128> = Item::new("previous_total_reward");

// result query
#[cw_serde]
pub struct CampaignInfoResult {
    pub owner: Addr,
    pub campaign_name: String,
    pub campaign_image: String,
    pub campaign_description: String,
    pub total_nft_staked: u64,
    pub total_reward_claimed: Uint128,
    pub total_reward: Uint128,
    pub limit_per_staker: u64,
    pub reward_token_info: AssetToken,
    pub allowed_collection: Addr,
    pub lockup_term: Vec<LockupTerm>,
    pub reward_per_second: Uint128,
    pub start_time: u64,
    pub end_time: u64,
}
