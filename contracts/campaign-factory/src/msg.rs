use crate::state::{ConfigResponse, CreateCampaign, FactoryCampaign};
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    /// Campaign code ID
    pub campaign_code_id: u64,
    pub allow_create_for_all: bool,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// UpdateConfig update relevant code IDs
    UpdateConfig {
        campaign_code_id: Option<u64>,
        allow_create_for_all: Option<bool>,
    },
    /// CreateCampaign instantiates pair contract
    CreateCampaign { create_campaign: CreateCampaign },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},

    #[returns(FactoryCampaign)]
    Campaign { campaign_id: u64 },

    #[returns(Vec<FactoryCampaign>)]
    Campaigns {
        start_after: Option<u64>,
        limit: Option<u32>,
    },

    #[returns(Vec<String>)]
    CampaignAddrs {},
}
