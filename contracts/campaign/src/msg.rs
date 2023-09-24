use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

use crate::state::{
    AssetToken, CampaignInfo, LockupTerm, NftInfo, NftKey, NftStake, RewardRate,
    StakerRewardAssetInfo,
};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
    pub owner: String, // owner of campaign
    // info detail
    pub campaign_name: String,
    pub campaign_image: String,
    pub campaign_description: String,

    pub limit_per_staker: u64,
    pub reward_token_info: AssetToken, // reward token
    pub allowed_collection: String,    // staking collection nft
    pub lockup_term: Vec<LockupTerm>,  // flexible, 15days, 30days, 60days

    pub start_time: u64, // start time must be from T + 1
    pub end_time: u64,   // max 3 years
}

#[cw_serde]
pub enum ExecuteMsg {
    AddRewardToken {
        amount: Uint128,
    },
    // user can stake 1 or many nfts to this campaign
    StakeNfts {
        stake_info: NftStake,
    },

    UnStakeNft {
        unstake_info: NftKey,
        token_id: String,
    },

    // user can claim reward
    ClaimReward {
        amount: Uint128,
    },

    WithdrawReward {},

    ResetPool {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(CampaignInfo)]
    CampaignInfo {},

    #[returns(NftInfo)]
    NftInfo { nft_key: NftKey },

    #[returns(Vec<NftInfo>)]
    Nfts {
        lockup_term: u64,
        start_after: Option<u64>,
        limit: Option<u32>,
    },

    #[returns(StakerRewardAssetInfo)]
    NftStaked { owner: Addr },

    #[returns(Uint128)]
    TotalPendingReward {},

    #[returns(Vec<RewardRate>)]
    TermRewardRates { term_value: u64 },
}
