#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, QueryRequest, Response,
    StdError, StdResult, Uint128, WasmMsg, WasmQuery,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{
    AssetToken, CampaignInfo, Config, NftInfo, NftKey, NftStake, RewardRate, StakerRewardAssetInfo,
    TokenInfo, CAMPAIGN_INFO, CONFIG, NFTS, NUMBER_OF_NFTS, PREVIOUS_TOTAL_REWARD, STAKERS_INFO,
    TERM_EXPIRATION_TIMES, TERM_REWARD_RATES, TOTAL_NFTS, TOTAL_STAKING_BY_TERM,
};
use crate::utils::{calculate_reward, stake_nft, update_reward_rate};
use cw20::Cw20ExecuteMsg;
use cw721::{Cw721ExecuteMsg, Cw721QueryMsg};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:campaign";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const MAX_TIME_VALID: u64 = 94608000; // 3 years
const MAX_LENGTH_NAME: usize = 100;
const MAX_LENGTH_IMAGE: usize = 500;
const MAX_LENGTH_DESCRIPTION: usize = 500;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // set version to contract
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // validate token contract address
    match msg.reward_token_info.info.clone() {
        TokenInfo::Token { contract_addr } => {
            deps.api.addr_validate(&contract_addr)?;
        }
        TokenInfo::NativeToken { denom: _ } => {
            return Err(ContractError::InvalidToken {});
        }
    }

    // Not allow start time is greater than end time
    if msg.start_time >= msg.end_time {
        return Err(ContractError::Std(StdError::generic_err(
            "## Start time is greater than end time ##",
        )));
    }

    // campaign during max 3 years
    if (msg.end_time - msg.start_time) > MAX_TIME_VALID {
        return Err(ContractError::LimitStartDate {});
    }

    // validate limit character campaign name & campaign description
    if msg.campaign_name.len() > MAX_LENGTH_NAME {
        return Err(ContractError::LimitCharacter {
            max: MAX_LENGTH_NAME.to_string(),
        });
    }

    if msg.campaign_image.len() > MAX_LENGTH_IMAGE {
        return Err(ContractError::LimitCharacter {
            max: MAX_LENGTH_IMAGE.to_string(),
        });
    }

    if msg.campaign_description.len() > MAX_LENGTH_DESCRIPTION {
        return Err(ContractError::LimitCharacter {
            max: MAX_LENGTH_DESCRIPTION.to_string(),
        });
    }

    let total_percent = msg.lockup_term.iter().fold(Uint128::zero(), |acc, term| {
        acc.checked_add(term.percent).unwrap()
    });

    if total_percent != Uint128::from(100u128) {
        return Err(ContractError::InvalidFunds {});
    }

    let config = Config {
        admin: deps.api.addr_validate(&msg.admin).unwrap(),
    };

    // save config can reset pool
    CONFIG.save(deps.storage, &config)?;

    // campaign info
    let campaign = CampaignInfo {
        owner: deps.api.addr_validate(&msg.owner).unwrap(),
        campaign_name: msg.campaign_name.clone(),
        campaign_image: msg.campaign_image.clone(),
        campaign_description: msg.campaign_description.clone(),
        total_reward_claimed: Uint128::zero(),
        total_reward: Uint128::zero(),
        limit_per_staker: msg.limit_per_staker,
        total_eligible: msg.total_eligible,
        reward_token: AssetToken {
            info: msg.reward_token_info.info.clone(),
            amount: Uint128::zero(),
        },
        allowed_collection: deps.api.addr_validate(&msg.allowed_collection).unwrap(),
        lockup_term: msg.lockup_term.clone(),
        reward_per_second: Uint128::zero(),
        start_time: msg.start_time,
        end_time: msg.end_time,
    };

    // save campaign info
    CAMPAIGN_INFO.save(deps.storage, &campaign)?;

    // init TERM_REWARD_RATES, TOTAL_STAKING_BY_TERM, EXPIRATION_TIMES, NUMBER_OF_NFTS
    for term in msg.lockup_term.iter() {
        TERM_REWARD_RATES.save(deps.storage, term.value, &vec![])?;
        TOTAL_STAKING_BY_TERM.save(deps.storage, term.value, &0u64)?;
        TERM_EXPIRATION_TIMES.save(deps.storage, term.value, &vec![])?;
        NUMBER_OF_NFTS.save(deps.storage, term.value, &0u64)?;
    }

    PREVIOUS_TOTAL_REWARD.save(deps.storage, &Uint128::zero())?;

    TOTAL_NFTS.save(deps.storage, &0u64)?;

    // we need emit the information of reward token to response
    let reward_token_info_str = match msg.reward_token_info.info {
        TokenInfo::Token { contract_addr } => contract_addr,
        TokenInfo::NativeToken { denom } => denom,
    };

    // emit the information of instantiated campaign
    Ok(Response::new().add_attributes([
        ("action", "instantiate"),
        ("owner", &msg.owner),
        ("campaign_name", &msg.campaign_name),
        ("campaign_image", &msg.campaign_image),
        ("campaign_description", &msg.campaign_description),
        ("limit_per_staker", &msg.limit_per_staker.to_string()),
        ("total_eligible", &msg.total_eligible.to_string()),
        ("reward_token_info", &reward_token_info_str),
        ("allowed_collection", &msg.allowed_collection),
        ("lockup_term", &format!("{:?}", &msg.lockup_term)),
        ("start_time", &msg.start_time.to_string()),
        ("end_time", &msg.end_time.to_string()),
    ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AddRewardToken { amount } => execute_add_reward_token(deps, env, info, amount),
        ExecuteMsg::StakeNfts { stake_info } => execute_stake_nft(deps, env, info, stake_info),
        ExecuteMsg::UnStakeNft { unstake_info } => {
            execute_unstake_nft(deps, env, info, unstake_info)
        }
        ExecuteMsg::UnStakeAndClaimNft { un_stakes } => {
            execute_unstake_nft_and_claim(deps, env, info, un_stakes)
        }
        ExecuteMsg::ClaimReward { amount } => execute_claim_reward(deps, env, info, amount),
        ExecuteMsg::WithdrawReward {} => execute_withdraw_reward(deps, env, info),
        ExecuteMsg::ResetPool {} => execute_reset_pool(deps, env, info),
        ExecuteMsg::UpdateAdmin { admin } => execute_update_admin(deps, env, info, admin),
    }
}

pub fn execute_add_reward_token(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // load campaign info
    let mut campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;

    let current_time = env.block.time.seconds();

    // only owner can add reward token to campaign
    if campaign_info.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // only reward_per_second == 0 || start_time > current_time can add reward
    if campaign_info.reward_per_second != Uint128::zero()
        && campaign_info.start_time <= current_time
    {
        return Err(ContractError::InvalidTimeToAddReward {});
    }

    let mut res = Response::new();

    // we need determine the reward token is native token or cw20 token
    match campaign_info.reward_token.info.clone() {
        TokenInfo::Token { contract_addr } => {
            // execute cw20 transfer msg from info.sender to contract
            res = res.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                    owner: info.sender.to_string(),
                    recipient: env.contract.address.to_string(),
                    amount,
                })?,
                funds: vec![],
            }));

            // add token info to response
            res = res.add_attribute("reward_token_info", contract_addr);

            // update amount, reward_per_second token in campaign
            campaign_info.reward_token.amount =
                campaign_info.reward_token.amount.saturating_add(amount);

            campaign_info.reward_per_second = campaign_info
                .reward_token
                .amount
                .checked_div(Uint128::from(
                    campaign_info.end_time - campaign_info.start_time,
                ))
                .unwrap_or(campaign_info.reward_token.amount);
            campaign_info.total_reward = campaign_info.total_reward.saturating_add(amount);

            // save campaign
            CAMPAIGN_INFO.save(deps.storage, &campaign_info)?;
        }
        TokenInfo::NativeToken { denom: _ } => {}
    }
    Ok(res.add_attributes([
        ("action", "add_reward_token"),
        ("owner", campaign_info.owner.as_ref()),
        ("reward_token_amount", &amount.to_string()),
    ]))
}

pub fn execute_stake_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    stake_info: NftStake,
) -> Result<Response, ContractError> {
    // load campaign info
    let campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;

    // the reward token must be added to campaign before staking nft
    if campaign_info.reward_per_second == Uint128::zero() {
        return Err(ContractError::EmptyReward {});
    }

    let mut total_nfts = TOTAL_NFTS.load(deps.storage)?;

    // if total_eligible != 0 then total nft in pool must < total_eligible
    if campaign_info.total_eligible != 0
        && (total_nfts + (stake_info.token_ids.len() as u64)) > campaign_info.total_eligible
    {
        return Err(ContractError::LimitPerCollection {});
    }

    // check invalid lockup_term
    if !campaign_info
        .lockup_term
        .iter()
        .any(|t| t.value == stake_info.lockup_term)
    {
        return Err(ContractError::InvalidLockupTerm {});
    }

    // load lockup_term in campaign info
    let lockup_term = campaign_info
        .lockup_term
        .iter()
        .find(|&term| term.value == stake_info.lockup_term)
        .cloned()
        .unwrap();

    // if stake before campaign active then time stake = campaign.start_time
    let mut current_time = env.block.time.seconds();
    if campaign_info.start_time > env.block.time.seconds() {
        current_time = campaign_info.start_time;
    }

    // only current_time < end_time can stake nft
    if campaign_info.end_time <= current_time {
        return Err(ContractError::InvalidTimeToStakeNft {});
    }

    // load staker_info or default if staker has not staked nft
    let mut staker_info = STAKERS_INFO
        .may_load(deps.storage, info.sender.clone())?
        .unwrap_or(StakerRewardAssetInfo {
            keys: vec![],
            reward_debt: Uint128::zero(),
            reward_claimed: Uint128::zero(),
        });

    // if limit per staker > 0 then check amount nft staked
    // if limit_per_staker = 0, then no limit nft stake
    if campaign_info.limit_per_staker > 0 {
        // the length of token_ids + length nft staked should be smaller than limit per staker
        if stake_info.token_ids.len() + staker_info.keys.len()
            > campaign_info.limit_per_staker as usize
        {
            return Err(ContractError::LimitPerStake {});
        }
    }

    // prepare response
    let mut res = Response::new();

    let mut nft_key = NUMBER_OF_NFTS.load(deps.storage, stake_info.lockup_term)?;

    // load TERM_REWARD_RATES, TOTAL_STAKING_BY_TERM, TERM_EXPIRATION_TIMES
    let mut term_reward_rates = TERM_REWARD_RATES.load(deps.storage, stake_info.lockup_term)?;
    let mut term_expiration_times =
        TERM_EXPIRATION_TIMES.load(deps.storage, stake_info.lockup_term)?;
    let mut total_staking_by_term =
        TOTAL_STAKING_BY_TERM.load(deps.storage, stake_info.lockup_term)?;

    // check the owner of token_ids, all token_ids should be owned by info.sender
    for token_id in &stake_info.token_ids {
        nft_key = nft_key.saturating_add(1u64); // if nft_key max throw err

        // check owner of nft
        let query_owner_msg = Cw721QueryMsg::OwnerOf {
            token_id: token_id.clone(),
            include_expired: Some(false),
        };
        let owner_response: StdResult<cw721::OwnerOfResponse> =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: campaign_info.allowed_collection.clone().to_string(),
                msg: to_binary(&query_owner_msg)?,
            }));
        match owner_response {
            Ok(owner) => {
                if owner.owner != info.sender {
                    return Err(ContractError::NotOwner {
                        token_id: token_id.to_string(),
                    });
                }
            }
            Err(_) => {
                return Err(ContractError::NotOwner {
                    token_id: token_id.to_string(),
                });
            }
        }

        // prepare message to transfer nft to contract
        let transfer_nft_msg = WasmMsg::Execute {
            contract_addr: campaign_info.allowed_collection.clone().to_string(),
            msg: to_binary(&Cw721ExecuteMsg::TransferNft {
                recipient: env.contract.address.clone().to_string(),
                token_id: token_id.clone(),
            })?,
            funds: vec![],
        };

        let nft_info = NftInfo {
            key: nft_key,
            token_id: token_id.clone(),
            owner: info.sender.clone(),
            pending_reward: Uint128::zero(),
            lockup_term: lockup_term.clone(),
            is_end_reward: false,
            start_time: current_time,
            time_calc: current_time,
            end_time: (current_time + lockup_term.value),
        };
        // save info nft
        NFTS.save(deps.storage, (nft_key, lockup_term.value), &nft_info)?;

        let (new_term_expiration_times, new_term_reward_rates, new_total_staking_by_term) =
            stake_nft(
                term_expiration_times.clone(),
                term_reward_rates.clone(),
                total_staking_by_term,
                nft_info.clone(),
                current_time,
            );

        // update rates, expire_times, total_staking after stake nft
        term_reward_rates = new_term_reward_rates;
        term_expiration_times = new_term_expiration_times;
        total_staking_by_term = new_total_staking_by_term;

        // update nft into staker_info
        staker_info.keys.push(NftKey {
            key: nft_key,
            token_id: token_id.to_string(),
            lockup_term: lockup_term.value,
        });

        total_nfts = total_nfts.saturating_add(1u64); // count total nft

        res = res.add_message(transfer_nft_msg);
    }

    // save TERM_REWARD_RATES, TOTAL_STAKING_BY_TERM, TERM_EXPIRATION_TIMES
    TERM_REWARD_RATES.save(deps.storage, stake_info.lockup_term, &term_reward_rates)?;
    TERM_EXPIRATION_TIMES.save(deps.storage, stake_info.lockup_term, &term_expiration_times)?;
    TOTAL_STAKING_BY_TERM.save(deps.storage, stake_info.lockup_term, &total_staking_by_term)?;

    // save NUMBER_OF_NFTS
    NUMBER_OF_NFTS.save(deps.storage, stake_info.lockup_term, &nft_key)?;

    // save STAKER_INFO
    STAKERS_INFO.save(deps.storage, info.sender.clone(), &staker_info)?;

    // save TOTAL_NFTS
    TOTAL_NFTS.save(deps.storage, &total_nfts)?;

    Ok(res.add_attributes([
        ("action", "stake_nft"),
        ("owner", info.sender.as_ref()),
        (
            "allowed_collection",
            campaign_info.allowed_collection.as_ref(),
        ),
        ("stake_info", &format!("{:?}", &stake_info)),
    ]))
}

pub fn execute_unstake_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    unstake_info: NftKey,
) -> Result<Response, ContractError> {
    // load campaign info
    let campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;
    // prepare response
    let mut res = Response::new();

    if NFTS
        .may_load(deps.storage, (unstake_info.key, unstake_info.lockup_term))?
        .is_none()
    {
        return Err(ContractError::EmptyNft {
            key: unstake_info.key,
        });
    }

    // max time calc pending reward is campaign_info.end_time
    let mut current_time = env.block.time.seconds();
    if campaign_info.end_time < env.block.time.seconds() {
        current_time = campaign_info.end_time;
    }

    // load nft info
    let nft_info = NFTS.load(deps.storage, (unstake_info.key, unstake_info.lockup_term))?;

    if nft_info.owner != info.sender {
        return Err(ContractError::NotOwner {
            token_id: nft_info.token_id,
        });
    }

    // load TERM_REWARD_RATES
    let term_reward_rates = TERM_REWARD_RATES.load(deps.storage, nft_info.lockup_term.value)?;
    let total_staking = TOTAL_STAKING_BY_TERM.load(deps.storage, nft_info.lockup_term.value)?;
    let expiration_times = TERM_EXPIRATION_TIMES.load(deps.storage, nft_info.lockup_term.value)?;

    let (new_nft_info, _, _, _) = calculate_reward(
        nft_info.clone(),
        term_reward_rates,
        expiration_times,
        total_staking,
        current_time,
        campaign_info.end_time,
        campaign_info.reward_per_second,
    );

    // check time unstake and owner nft
    if !new_nft_info.is_end_reward {
        return Err(ContractError::InvalidTimeToUnStake {});
    }

    // prepare message to transfer nft back to the owner
    let transfer_nft_msg = WasmMsg::Execute {
        contract_addr: campaign_info.allowed_collection.to_string(),
        msg: to_binary(&Cw721ExecuteMsg::TransferNft {
            recipient: nft_info.owner.to_string(),
            token_id: nft_info.token_id.clone(),
        })?,
        funds: vec![],
    };
    res = res.add_message(transfer_nft_msg);

    // update reward for staker
    let mut staker = STAKERS_INFO.load(deps.storage, info.sender.clone())?;

    staker.reward_debt = staker
        .reward_debt
        .saturating_add(new_nft_info.pending_reward);

    staker
        .keys
        .retain(|k| !(k.key == unstake_info.key && k.lockup_term == unstake_info.lockup_term)); // remove nft for staker

    STAKERS_INFO.save(deps.storage, info.sender.clone(), &staker)?;

    // remove nft in NFTS
    NFTS.remove(deps.storage, (unstake_info.key, unstake_info.lockup_term));

    Ok(res.add_attributes([
        ("action", "unstake_nft"),
        ("owner", info.sender.as_ref()),
        (
            "allowed_collection",
            campaign_info.allowed_collection.as_ref(),
        ),
        ("token_id", &nft_info.token_id),
    ]))
}

pub fn execute_unstake_nft_and_claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    un_stakes: Vec<NftKey>,
) -> Result<Response, ContractError> {
    // load campaign info
    let mut campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;

    // prepare response
    let mut res: Response = Response::new();

    // max time calc pending reward is campaign_info.end_time
    let mut current_time = env.block.time.seconds();
    if campaign_info.end_time < env.block.time.seconds() {
        current_time = campaign_info.end_time;
    }

    let mut staker: StakerRewardAssetInfo = STAKERS_INFO.load(deps.storage, info.sender.clone())?;
    let mut pending_reward_staker: Uint128 = Uint128::zero();

    for nft in &un_stakes {
        if NFTS
            .may_load(deps.storage, (nft.key, nft.lockup_term))?
            .is_none()
        {
            return Err(ContractError::EmptyNft { key: nft.key });
        }

        // load nft info
        let nft_info = NFTS.load(deps.storage, (nft.key, nft.lockup_term))?;

        if nft_info.owner != info.sender {
            return Err(ContractError::NotOwner {
                token_id: nft_info.token_id,
            });
        }

        // load TERM_REWARD_RATES
        let term_reward_rates = TERM_REWARD_RATES.load(deps.storage, nft_info.lockup_term.value)?;
        let total_staking = TOTAL_STAKING_BY_TERM.load(deps.storage, nft_info.lockup_term.value)?;
        let expiration_times =
            TERM_EXPIRATION_TIMES.load(deps.storage, nft_info.lockup_term.value)?;

        let (new_nft_info, _, _, _) = calculate_reward(
            nft_info.clone(),
            term_reward_rates,
            expiration_times,
            total_staking,
            current_time,
            campaign_info.end_time,
            campaign_info.reward_per_second,
        );

        // check time unstake and owner nft
        if !new_nft_info.is_end_reward {
            return Err(ContractError::InvalidTimeToUnStake {});
        }

        // prepare message to transfer nft back to the owner
        let transfer_nft_msg = WasmMsg::Execute {
            contract_addr: campaign_info.allowed_collection.to_string(),
            msg: to_binary(&Cw721ExecuteMsg::TransferNft {
                recipient: nft_info.owner.to_string(),
                token_id: nft_info.token_id.clone(),
            })?,
            funds: vec![],
        };

        pending_reward_staker = pending_reward_staker.saturating_add(new_nft_info.pending_reward);

        staker
            .keys
            .retain(|k| !(k.key == nft.key && k.lockup_term == nft.lockup_term)); // remove nft for staker

        // remove nft in NFTS
        NFTS.remove(deps.storage, (nft.key, nft.lockup_term));

        res = res.add_message(transfer_nft_msg);
    }

    match campaign_info.reward_token.info.clone() {
        TokenInfo::Token { contract_addr } => {
            // execute cw20 transfer msg from info.sender to contract
            res = res.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: info.sender.to_string(),
                    amount: pending_reward_staker,
                })?,
                funds: vec![],
            }));

            res = res.add_attributes([
                ("reward_token_info", contract_addr),
                ("reward_claim_amount", pending_reward_staker.to_string()),
            ]);

            // update staker info
            staker.reward_claimed = staker.reward_claimed.saturating_add(pending_reward_staker);

            // update reward total and reward claimed for campaign
            campaign_info.reward_token.amount = campaign_info
                .reward_token
                .amount
                .saturating_sub(pending_reward_staker);

            campaign_info.total_reward_claimed = campaign_info
                .total_reward_claimed
                .saturating_add(pending_reward_staker);
        }
        TokenInfo::NativeToken { denom: _ } => {}
    }

    STAKERS_INFO.save(deps.storage, info.sender.clone(), &staker)?;

    // save campaign info
    CAMPAIGN_INFO.save(deps.storage, &campaign_info)?;

    Ok(res.add_attributes([
        ("action", "unstake_nft"),
        ("owner", info.sender.as_ref()),
        (
            "allowed_collection",
            campaign_info.allowed_collection.as_ref(),
        ),
        ("un_stakes", &format!("{:?}", &un_stakes)),
    ]))
}

pub fn execute_claim_reward(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // load campaign info
    let mut campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;

    // Only stakers could claim rewards in this campaign
    if STAKERS_INFO
        .may_load(deps.storage, info.sender.clone())?
        .is_none()
    {
        return Err(ContractError::InvalidClaim {});
    }

    // load staker_info
    let mut staker_info = STAKERS_INFO.load(deps.storage, info.sender.clone())?;

    let mut pending_reward_staker: Uint128 = Uint128::zero();

    // max time calc pending reward is campaign_info.end_time
    let mut current_time = env.block.time.seconds();
    if campaign_info.end_time < env.block.time.seconds() {
        current_time = campaign_info.end_time;
    }

    // transfer pending reward nft into staker
    for key in staker_info.keys.iter() {
        let nft_info = NFTS.load(deps.storage, (key.key, key.lockup_term))?;

        // load TERM_REWARD_RATES
        let term_reward_rates = TERM_REWARD_RATES.load(deps.storage, nft_info.lockup_term.value)?;

        let total_staking = TOTAL_STAKING_BY_TERM.load(deps.storage, nft_info.lockup_term.value)?;

        let expiration_times =
            TERM_EXPIRATION_TIMES.load(deps.storage, nft_info.lockup_term.value)?;

        let (mut new_nft_info, new_term_reward_rates, new_total_staking, new_expiration_times) =
            calculate_reward(
                nft_info,
                term_reward_rates.clone(),
                expiration_times,
                total_staking,
                current_time,
                campaign_info.end_time,
                campaign_info.reward_per_second,
            );

        pending_reward_staker = pending_reward_staker.saturating_add(new_nft_info.pending_reward);

        //update pending reward for nft = 0 because pending reward in nft are transferred to staker
        new_nft_info.pending_reward = Uint128::zero();
        NFTS.save(deps.storage, (key.key, key.lockup_term), &new_nft_info)?;

        // update term reward rates
        TERM_REWARD_RATES.save(
            deps.storage,
            new_nft_info.lockup_term.value,
            &new_term_reward_rates,
        )?;
        TOTAL_STAKING_BY_TERM.save(
            deps.storage,
            new_nft_info.lockup_term.value,
            &new_total_staking,
        )?;
        TERM_EXPIRATION_TIMES.save(
            deps.storage,
            new_nft_info.lockup_term.value,
            &new_expiration_times,
        )?;
    }

    // amount reward claim must be less than or equal reward in staker
    if amount > staker_info.reward_debt + pending_reward_staker {
        return Err(ContractError::InsufficientBalance {});
    }

    let mut res = Response::new();

    match campaign_info.reward_token.info.clone() {
        TokenInfo::Token { contract_addr } => {
            // execute cw20 transfer msg from info.sender to contract
            res = res.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: info.sender.to_string(),
                    amount,
                })?,
                funds: vec![],
            }));

            res = res.add_attributes([
                ("reward_token_info", contract_addr),
                ("reward_claim_amount", amount.to_string()),
            ]);

            // update staker info
            staker_info.reward_claimed = staker_info.reward_claimed.saturating_add(amount);

            staker_info.reward_debt = staker_info
                .reward_debt
                .saturating_add(pending_reward_staker)
                .saturating_sub(amount);

            STAKERS_INFO.save(deps.storage, info.sender, &staker_info)?;

            // update reward total and reward claimed for campaign
            campaign_info.reward_token.amount =
                campaign_info.reward_token.amount.saturating_sub(amount);

            campaign_info.total_reward_claimed =
                campaign_info.total_reward_claimed.saturating_add(amount);

            // save campaign info
            CAMPAIGN_INFO.save(deps.storage, &campaign_info)?;
        }
        TokenInfo::NativeToken { denom: _ } => {}
    }
    Ok(res.add_attributes([
        ("action", "claim_reward"),
        ("owner", campaign_info.owner.as_ref()),
    ]))
}

pub fn execute_withdraw_reward(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // load campaign info
    let mut campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;

    // permission check
    if info.sender != campaign_info.owner {
        return Err(ContractError::Unauthorized {});
    }

    // campaign must be ended then withdraw remaining reward
    if campaign_info.end_time > env.block.time.seconds() {
        return Err(ContractError::InvalidTimeToWithdrawReward {});
    }

    // total_pending_reward = previous total reward + total in rates - reward claimed
    let mut total_pending_reward = PREVIOUS_TOTAL_REWARD.load(deps.storage)?;

    // time to calc pending reward
    let current_time = campaign_info.end_time;

    // load TERM_REWARD_RATES
    for term in campaign_info.lockup_term.iter() {
        let mut term_reward_rates = TERM_REWARD_RATES.load(deps.storage, term.value)?;
        let expiration_times = TERM_EXPIRATION_TIMES.load(deps.storage, term.value)?;
        let mut total_staking = TOTAL_STAKING_BY_TERM.load(deps.storage, term.value)?;
        for &end_time in &expiration_times {
            if end_time <= current_time {
                let (updated_reward_rate, t) =
                    update_reward_rate(term_reward_rates, total_staking, end_time, -1);
                term_reward_rates = updated_reward_rate;
                total_staking = t;
            }
        }

        let (final_reward_rate, _) =
            update_reward_rate(term_reward_rates, total_staking, current_time, 0);

        if final_reward_rate.len() > 1 {
            for i in 0..(final_reward_rate.len() - 1) {
                let current = &final_reward_rate[i];
                let next = &final_reward_rate[i + 1];

                if current.rate != 0 && next.timestamp > current.timestamp {
                    let duration = (next.timestamp - current.timestamp) as u128;
                    let reward_per_second = campaign_info.reward_per_second.u128();
                    let term_percent = term.percent.u128();

                    let product = duration
                        .saturating_mul(reward_per_second)
                        .saturating_mul(term_percent)
                        .saturating_div(100u128);

                    total_pending_reward =
                        Uint128::from(total_pending_reward.u128().saturating_add(product));
                }
            }
        }
    }

    total_pending_reward = total_pending_reward.saturating_sub(campaign_info.total_reward_claimed);

    // reward remaining = current total reward - total pending reward
    let withdraw_reward = campaign_info
        .reward_token
        .amount
        .saturating_sub(total_pending_reward);

    let mut res = Response::new();
    match campaign_info.reward_token.info.clone() {
        TokenInfo::Token { contract_addr } => {
            // execute cw20 transfer msg from info.sender to contract

            res = res.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: info.sender.to_string(),
                    amount: withdraw_reward,
                })?,
                funds: vec![],
            }));

            res = res.add_attributes([
                ("reward_token_info", contract_addr),
                ("withdraw_reward_amount", withdraw_reward.to_string()),
            ]);

            // update reward total and reward claimed for campaign
            campaign_info.reward_token.amount = campaign_info
                .reward_token
                .amount
                .saturating_sub(withdraw_reward);

            CAMPAIGN_INFO.save(deps.storage, &campaign_info)?;
        }
        TokenInfo::NativeToken { denom: _ } => {}
    }
    Ok(res.add_attributes([
        ("action", "withdraw_reward"),
        ("owner", campaign_info.owner.as_ref()),
    ]))
}

pub fn execute_reset_pool(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;
    // load campaign info
    let campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;

    // permission check
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    // max time calc pending reward is campaign_info.end_time
    let mut current_time = env.block.time.seconds();
    if campaign_info.end_time < env.block.time.seconds() {
        current_time = campaign_info.end_time;
    }

    let mut current_total_reward = PREVIOUS_TOTAL_REWARD.load(deps.storage)?;

    // load TERM_REWARD_RATES
    for term in campaign_info.lockup_term.iter() {
        let mut term_reward_rates = TERM_REWARD_RATES.load(deps.storage, term.value)?;
        let mut expiration_times = TERM_EXPIRATION_TIMES.load(deps.storage, term.value)?;
        let mut total_staking = TOTAL_STAKING_BY_TERM.load(deps.storage, term.value)?;

        let nft_count_by_term = NUMBER_OF_NFTS.load(deps.storage, term.value)?;

        let nfts_by_term = (0..nft_count_by_term)
            .map(|key| NFTS.load(deps.storage, (key + 1, term.value)))
            .filter_map(Result::ok)
            .collect::<Vec<_>>();

        for nft_info in nfts_by_term.iter() {
            let (new_nft_info, new_term_reward_rates, new_total_staking, new_expiration_times) =
                calculate_reward(
                    nft_info.clone(),
                    term_reward_rates.clone(),
                    expiration_times.clone(),
                    total_staking,
                    current_time,
                    campaign_info.end_time,
                    campaign_info.reward_per_second,
                );
            term_reward_rates = new_term_reward_rates;
            total_staking = new_total_staking;
            expiration_times = new_expiration_times;
            NFTS.save(deps.storage, (nft_info.key, term.value), &new_nft_info)?;
        }

        // calculate total pending reward in current reward_rates
        if term_reward_rates.len() > 1 {
            for i in 0..(term_reward_rates.len() - 1) {
                let current = &term_reward_rates[i];
                let next = &term_reward_rates[i + 1];

                if current.rate != 0 && next.timestamp > current.timestamp {
                    let duration = (next.timestamp - current.timestamp) as u128;
                    let reward_per_second = campaign_info.reward_per_second.u128();
                    let term_percent = term.percent.u128();

                    let product = duration
                        .saturating_mul(reward_per_second)
                        .saturating_mul(term_percent)
                        .saturating_div(100u128);

                    current_total_reward =
                        Uint128::from(current_total_reward.u128().saturating_add(product));
                }
            }
        }

        // update reward rates for future
        let updated_term_reward_rates = term_reward_rates
            .last()
            .map_or(Vec::new(), |last_element| vec![last_element.clone()]);

        TERM_REWARD_RATES.save(deps.storage, term.value, &updated_term_reward_rates)?;
        TOTAL_STAKING_BY_TERM.save(deps.storage, term.value, &total_staking)?;
        TERM_EXPIRATION_TIMES.save(deps.storage, term.value, &expiration_times)?;
    }

    PREVIOUS_TOTAL_REWARD.save(deps.storage, &current_total_reward)?;

    Ok(
        Response::new()
            .add_attributes([("action", "reset_pool"), ("admin", config.admin.as_ref())]),
    )
}

pub fn execute_update_admin(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admin: String,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;

    // permission check
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    let new_config = Config {
        admin: deps.api.addr_validate(&admin).unwrap(),
    };

    // save config admin
    CONFIG.save(deps.storage, &new_config)?;

    Ok(Response::new().add_attributes([("action", "update_admin"), ("admin", &admin)]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::CampaignInfo {} => Ok(to_binary(&query_campaign_info(deps)?)?),
        QueryMsg::NftInfo { nft_key } => Ok(to_binary(&query_nft_info(deps, env, nft_key)?)?),
        QueryMsg::Nfts {
            lockup_term,
            start_after,
            limit,
        } => Ok(to_binary(&query_nfts(
            deps,
            env,
            lockup_term,
            start_after,
            limit,
        )?)?),
        QueryMsg::TotalNfts {} => Ok(to_binary(&query_total_nfts(deps)?)?),
        QueryMsg::NftStaked { owner } => Ok(to_binary(&query_staker_info(deps, env, owner)?)?),
        QueryMsg::TotalPendingReward {} => Ok(to_binary(&query_total_pending_reward(deps, env)?)?),
        QueryMsg::TermRewardRates { term_value } => {
            Ok(to_binary(&query_term_reward_rates(deps, term_value)?)?)
        }
        QueryMsg::ExpirationTimes { term_value } => {
            Ok(to_binary(&query_expiration_times(deps, term_value)?)?)
        }
        QueryMsg::TotalStaking { term_value } => {
            Ok(to_binary(&query_total_staking(deps, term_value)?)?)
        }
        QueryMsg::VersionContract {} => Ok(to_binary(&query_version_contract()?)?),
    }
}

fn query_campaign_info(deps: Deps) -> Result<CampaignInfo, ContractError> {
    let campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;

    Ok(campaign_info)
}

fn query_total_nfts(deps: Deps) -> Result<u64, ContractError> {
    let total_nfts: u64 = TOTAL_NFTS.load(deps.storage)?;

    Ok(total_nfts)
}

fn query_nft_info(deps: Deps, env: Env, nft_key: NftKey) -> Result<NftInfo, ContractError> {
    let campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;
    let nft_info: NftInfo = NFTS.load(deps.storage, (nft_key.key, nft_key.lockup_term))?;

    let mut current_time = env.block.time.seconds();
    if campaign_info.end_time < env.block.time.seconds() {
        current_time = campaign_info.end_time;
    }

    let term_reward_rates = TERM_REWARD_RATES.load(deps.storage, nft_info.lockup_term.value)?;

    let total_staking = TOTAL_STAKING_BY_TERM.load(deps.storage, nft_info.lockup_term.value)?;

    let expiration_times = TERM_EXPIRATION_TIMES.load(deps.storage, nft_info.lockup_term.value)?;

    let (new_nft_info, _, _, _) = calculate_reward(
        nft_info,
        term_reward_rates,
        expiration_times,
        total_staking,
        current_time,
        campaign_info.end_time,
        campaign_info.reward_per_second,
    );

    Ok(new_nft_info)
}

fn query_nfts(
    deps: Deps,
    env: Env,
    lockup_term: u64,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> Result<Vec<NftInfo>, ContractError> {
    let start_after = start_after.unwrap_or(0);
    let limit = limit.unwrap_or(30) as usize;
    let nft_count_by_term = NUMBER_OF_NFTS.load(deps.storage, lockup_term)?;

    let campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;

    let mut current_time = env.block.time.seconds();
    if campaign_info.end_time < env.block.time.seconds() {
        current_time = campaign_info.end_time;
    }

    let mut nfts = (start_after..nft_count_by_term)
        .map(|key| NFTS.load(deps.storage, (key + 1, lockup_term)))
        .filter_map(Result::ok)
        .take(limit)
        .collect::<Vec<_>>();

    let term_reward_rates = TERM_REWARD_RATES.load(deps.storage, lockup_term)?;

    let total_staking = TOTAL_STAKING_BY_TERM.load(deps.storage, lockup_term)?;

    let expiration_times = TERM_EXPIRATION_TIMES.load(deps.storage, lockup_term)?;

    for nft_info in nfts.iter_mut() {
        let (new_nft_info, _, _, _) = calculate_reward(
            nft_info.clone(),
            term_reward_rates.clone(),
            expiration_times.clone(),
            total_staking,
            current_time,
            campaign_info.end_time,
            campaign_info.reward_per_second,
        );
        *nft_info = new_nft_info;
    }

    Ok(nfts)
}

fn query_staker_info(
    deps: Deps,
    _env: Env,
    owner: Addr,
) -> Result<StakerRewardAssetInfo, ContractError> {
    let staker_asset: StakerRewardAssetInfo =
        STAKERS_INFO
            .load(deps.storage, owner)
            .unwrap_or(StakerRewardAssetInfo {
                keys: vec![],
                reward_debt: Uint128::zero(),
                reward_claimed: Uint128::zero(),
            });

    Ok(staker_asset)
}

fn query_total_pending_reward(deps: Deps, env: Env) -> Result<Uint128, ContractError> {
    let campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;

    // total pending reward = previous total reward + total reward in rates - total reward claimed
    let mut total_pending_reward: Uint128 = PREVIOUS_TOTAL_REWARD.load(deps.storage)?;

    // max time to calc = campaign_info.end_time
    let mut current_time = env.block.time.seconds();
    if campaign_info.end_time < env.block.time.seconds() {
        current_time = campaign_info.end_time;
    }

    // load TERM_REWARD_RATES
    for term in campaign_info.lockup_term.iter() {
        let mut term_reward_rates = TERM_REWARD_RATES.load(deps.storage, term.value)?;
        let expiration_times = TERM_EXPIRATION_TIMES.load(deps.storage, term.value)?;
        let mut total_staking = TOTAL_STAKING_BY_TERM.load(deps.storage, term.value)?;

        for &end_time in &expiration_times {
            if end_time <= current_time {
                let (updated_reward_rate, t) =
                    update_reward_rate(term_reward_rates, total_staking, end_time, -1);
                term_reward_rates = updated_reward_rate;
                total_staking = t;
            }
        }

        let (final_reward_rate, _) =
            update_reward_rate(term_reward_rates, total_staking, current_time, 0);

        if final_reward_rate.len() > 1 {
            for i in 0..(final_reward_rate.len() - 1) {
                let current = &final_reward_rate[i];
                let next = &final_reward_rate[i + 1];

                if current.rate != 0 && next.timestamp > current.timestamp {
                    let duration = (next.timestamp - current.timestamp) as u128;
                    let reward_per_second = campaign_info.reward_per_second.u128();
                    let term_percent = term.percent.u128();

                    let product = duration
                        .saturating_mul(reward_per_second)
                        .saturating_mul(term_percent)
                        .saturating_div(100u128);

                    total_pending_reward =
                        Uint128::from(total_pending_reward.u128().saturating_add(product));
                }
            }
        }
    }

    total_pending_reward = total_pending_reward.saturating_sub(campaign_info.total_reward_claimed);

    Ok(total_pending_reward)
}

fn query_term_reward_rates(deps: Deps, term_value: u64) -> Result<Vec<RewardRate>, ContractError> {
    let reward_rates = TERM_REWARD_RATES.load(deps.storage, term_value)?;

    Ok(reward_rates)
}

fn query_expiration_times(deps: Deps, term_value: u64) -> Result<Vec<u64>, ContractError> {
    let expiration_times = TERM_EXPIRATION_TIMES.load(deps.storage, term_value)?;

    Ok(expiration_times)
}

fn query_total_staking(deps: Deps, term_value: u64) -> Result<u64, ContractError> {
    let total_staking = TOTAL_STAKING_BY_TERM.load(deps.storage, term_value)?;

    Ok(total_staking)
}

fn query_version_contract() -> Result<String, ContractError> {
    Ok("version_2".to_string())
}
