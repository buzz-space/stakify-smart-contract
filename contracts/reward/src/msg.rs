use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Uint128, Addr};

use crate::state::{AssetToken, RewardInfo, WatchListReward, StakerRewardAssetInfo};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,                 // owner of campaign
    pub reward_token_info: AssetToken, // reward token
}

#[cw_serde]
pub enum ExecuteMsg {
    AddRewardToken { amount: Uint128 },

    SetWatchListReward { watch_list: Vec<WatchListReward> },

    TransferReward{watch_list: Vec<Addr>}
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(RewardInfo)]
    RewardInfo {},

    #[returns(Vec<Addr>)]
    WatchList {},

    #[returns(StakerRewardAssetInfo)]
    StakerInfo {
        address: Addr
    },
}
