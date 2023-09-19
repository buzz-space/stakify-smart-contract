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
    AssetToken, CampaignInfo, CampaignInfoResult, Config, NftInfo, NftStake, RewardRate,
    StakerRewardAssetInfo, TokenInfo, CAMPAIGN_INFO, CONFIG, NFTS, PREVIOUS_TOTAL_REWARD,
    STAKERS_INFO, TERM_EXPIRATION_TIMES, TERM_REWARD_RATES, TOKEN_IDS, TOTAL_STAKING_BY_TERM,
};
use crate::utils::{add_reward, calculate_reward, stake_nft, sub_reward, update_reward_rate};
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
        owner: deps.api.addr_validate(&msg.admin).unwrap(),
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

    // init TERM_REWARD_RATES, TOTAL_STAKING_BY_TERM, EXPIRATION_TIMES, TOKEN_IDS
    for term in msg.lockup_term.iter() {
        TERM_REWARD_RATES.save(deps.storage, term.value.to_string(), &vec![])?;
        TOTAL_STAKING_BY_TERM.save(deps.storage, term.value.to_string(), &0u64)?;
        TERM_EXPIRATION_TIMES.save(deps.storage, term.value.to_string(), &vec![])?;
        TOKEN_IDS.save(deps.storage, term.value.to_string(), &vec![])?;
    }

    PREVIOUS_TOTAL_REWARD.save(deps.storage, &Uint128::zero())?;

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
        ExecuteMsg::StakeNfts { nfts } => execute_stake_nft(deps, env, info, nfts),
        ExecuteMsg::UnStakeNft { token_id } => execute_unstake_nft(deps, env, info, token_id),
        ExecuteMsg::ClaimReward { amount } => execute_claim_reward(deps, env, info, amount),
        ExecuteMsg::WithdrawReward {} => execute_withdraw_reward(deps, env, info),
        ExecuteMsg::ResetPool {} => execute_reset_pool(deps, env, info),
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
            campaign_info.reward_token.amount = campaign_info
                .reward_token
                .amount
                .checked_add(amount)
                .unwrap();
            campaign_info.reward_per_second = campaign_info
                .reward_token
                .amount
                .checked_div(Uint128::from(
                    campaign_info.end_time - campaign_info.start_time,
                ))
                .unwrap();
            campaign_info.total_reward = campaign_info.total_reward.checked_add(amount).unwrap();

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
    nfts: Vec<NftStake>,
) -> Result<Response, ContractError> {
    // load campaign info
    let campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;

    let current_time = env.block.time.seconds();

    // the reward token must be added to campaign before staking nft
    if campaign_info.reward_token.amount == Uint128::zero() {
        return Err(ContractError::EmptyReward {});
    }

    // only start_time < current_time && current_time < end_time && amount != 0 can stake nft
    if campaign_info.start_time >= current_time {
        return Err(ContractError::InvalidTimeToStakeNft {});
    }
    if campaign_info.end_time <= current_time {
        return Err(ContractError::InvalidTimeToStakeNft {});
    }

    // load staker_info or default if staker has not staked nft
    let mut staker_info = STAKERS_INFO
        .may_load(deps.storage, info.sender.clone())?
        .unwrap_or(StakerRewardAssetInfo {
            token_ids: vec![],
            reward_debt: Uint128::zero(),
            reward_claimed: Uint128::zero(),
        });

    // if limit per staker > 0 then check amount nft staked
    // if limit_per_staker = 0, then no limit nft stake
    if campaign_info.limit_per_staker > 0 {
        // the length of token_ids + length nft staked should be smaller than limit per staker
        if nfts.len() + staker_info.token_ids.len() > campaign_info.limit_per_staker as usize {
            return Err(ContractError::LimitPerStake {});
        }
    }

    // prepare response
    let mut res = Response::new();

    // check the owner of token_ids, all token_ids should be owned by info.sender
    for nft in &nfts {
        // let campaign_info = CAMPAIGN_INFO.load(deps.storage)?;
        // check invalid lockup_term
        if !campaign_info
            .clone()
            .lockup_term
            .iter()
            .any(|t| t.value == nft.lockup_term)
        {
            return Err(ContractError::InvalidLockupTerm {});
        }

        // check owner of nft
        let query_owner_msg = Cw721QueryMsg::OwnerOf {
            token_id: nft.token_id.clone(),
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
                        token_id: nft.token_id.to_string(),
                    });
                }
            }
            Err(_) => {
                return Err(ContractError::NotOwner {
                    token_id: nft.token_id.to_string(),
                });
            }
        }

        // prepare message to transfer nft to contract
        let transfer_nft_msg = WasmMsg::Execute {
            contract_addr: campaign_info.allowed_collection.clone().to_string(),
            msg: to_binary(&Cw721ExecuteMsg::TransferNft {
                recipient: env.contract.address.clone().to_string(),
                token_id: nft.token_id.clone(),
            })?,
            funds: vec![],
        };

        // load lockup_term in campaign info
        let lockup_term = campaign_info
            .lockup_term
            .iter()
            .find(|&term| term.value == nft.lockup_term)
            .cloned()
            .unwrap();

        let nft_info = NftInfo {
            token_id: nft.token_id.clone(),
            owner: info.sender.clone(),
            pending_reward: Uint128::zero(),
            lockup_term: lockup_term.clone(),
            is_end_reward: false,
            start_time: current_time,
            time_calc: current_time,
            end_time: (current_time + lockup_term.value),
        };
        // save info nft
        NFTS.save(deps.storage, nft.token_id.clone(), &nft_info)?;

        // load TERM_REWARD_RATES, TOTAL_STAKING_BY_TERM, TERM_EXPIRATION_TIMES
        let term_reward_rates =
            TERM_REWARD_RATES.load(deps.storage, nft.lockup_term.to_string())?;
        let term_expiration_times =
            TERM_EXPIRATION_TIMES.load(deps.storage, nft.lockup_term.to_string())?;
        let total_staking_by_term =
            TOTAL_STAKING_BY_TERM.load(deps.storage, nft.lockup_term.to_string())?;

        let (new_term_expiration_times, new_term_reward_rates, new_total_staking_by_term) =
            stake_nft(
                term_expiration_times,
                term_reward_rates,
                total_staking_by_term,
                nft.clone(),
                current_time,
            );

        // load TOKEN_IDS
        let mut token_ids: Vec<String> =
            TOKEN_IDS.load(deps.storage, nft.lockup_term.to_string())?;

        // save TERM_REWARD_RATES, TOTAL_STAKING_BY_TERM, TERM_EXPIRATION_TIMES
        TERM_REWARD_RATES.save(
            deps.storage,
            nft.lockup_term.to_string(),
            &new_term_reward_rates,
        )?;
        TERM_EXPIRATION_TIMES.save(
            deps.storage,
            nft.lockup_term.to_string(),
            &new_term_expiration_times,
        )?;
        TOTAL_STAKING_BY_TERM.save(
            deps.storage,
            nft.lockup_term.to_string(),
            &new_total_staking_by_term,
        )?;

        // update token_id in TOKEN_IDS
        token_ids.push(nft.token_id.clone());
        TOKEN_IDS.save(deps.storage, nft.lockup_term.to_string(), &token_ids)?;

        // save staker_info
        staker_info.token_ids.push(nft.token_id.clone());

        res = res.add_message(transfer_nft_msg);
    }

    // save STAKER_INFO
    STAKERS_INFO.save(deps.storage, info.sender.clone(), &staker_info)?;

    Ok(res.add_attributes([
        ("action", "stake_nft"),
        ("owner", info.sender.as_ref()),
        (
            "allowed_collection",
            campaign_info.allowed_collection.as_ref(),
        ),
        ("nfts", &format!("{:?}", &nfts)),
    ]))
}

pub fn execute_unstake_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
) -> Result<Response, ContractError> {
    // load campaign info
    let campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;
    // prepare response
    let mut res = Response::new();

    if NFTS.may_load(deps.storage, token_id.clone())?.is_none() {
        return Err(ContractError::EmptyNft { token_id });
    }

    // max time calc pending reward is campaign_info.end_time
    let mut current_time = env.block.time.seconds();
    if campaign_info.end_time < env.block.time.seconds() {
        current_time = campaign_info.end_time;
    }

    // load nft info
    let nft_info = NFTS.load(deps.storage, token_id.clone())?;

    // load TERM_REWARD_RATES
    let term_reward_rates =
        TERM_REWARD_RATES.load(deps.storage, nft_info.lockup_term.value.to_string())?;

    let total_staking =
        TOTAL_STAKING_BY_TERM.load(deps.storage, nft_info.lockup_term.value.to_string())?;

    let expiration_times =
        TERM_EXPIRATION_TIMES.load(deps.storage, nft_info.lockup_term.value.to_string())?;

    let (new_nft_info, _, _, _) = calculate_reward(
        nft_info,
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
            recipient: info.sender.to_string(),
            token_id: token_id.clone(),
        })?,
        funds: vec![],
    };
    res = res.add_message(transfer_nft_msg);

    // remove token_id in TOKEN_IDS
    let mut token_ids = TOKEN_IDS.load(deps.storage, new_nft_info.lockup_term.value.to_string())?;
    token_ids.retain(|id| *id != token_id.clone());
    TOKEN_IDS.save(
        deps.storage,
        new_nft_info.lockup_term.value.to_string(),
        &token_ids,
    )?;

    // update reward for staker
    let mut staker = STAKERS_INFO.load(deps.storage, info.sender.clone())?;
    staker.reward_debt = add_reward(staker.reward_debt, new_nft_info.pending_reward).unwrap();
    staker.token_ids.retain(|key| *key != token_id.clone()); // remove nft for staker
    STAKERS_INFO.save(deps.storage, info.sender.clone(), &staker)?;

    // remove nft in NFTS
    NFTS.remove(deps.storage, token_id.clone());

    Ok(res.add_attributes([
        ("action", "unstake_nft"),
        ("owner", info.sender.as_ref()),
        (
            "allowed_collection",
            campaign_info.allowed_collection.as_ref(),
        ),
        ("token_id", &token_id),
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

    // transfer pending reward in nft to staker
    for id in staker_info.token_ids.iter() {
        let nft_info = NFTS.load(deps.storage, id.clone())?;

        // load TERM_REWARD_RATES
        let term_reward_rates =
            TERM_REWARD_RATES.load(deps.storage, nft_info.lockup_term.value.to_string())?;

        let total_staking =
            TOTAL_STAKING_BY_TERM.load(deps.storage, nft_info.lockup_term.value.to_string())?;

        let expiration_times =
            TERM_EXPIRATION_TIMES.load(deps.storage, nft_info.lockup_term.value.to_string())?;

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

        pending_reward_staker =
            add_reward(pending_reward_staker, new_nft_info.pending_reward).unwrap();

        //update pending reward for nft = 0 because pending reward in nft are transferred to staker
        new_nft_info.pending_reward = Uint128::zero();
        NFTS.save(deps.storage, id.clone(), &new_nft_info)?;

        // update term reward rates
        TERM_REWARD_RATES.save(
            deps.storage,
            new_nft_info.lockup_term.value.to_string(),
            &new_term_reward_rates,
        )?;
        TOTAL_STAKING_BY_TERM.save(
            deps.storage,
            new_nft_info.lockup_term.value.to_string(),
            &new_total_staking,
        )?;
        TERM_EXPIRATION_TIMES.save(
            deps.storage,
            new_nft_info.lockup_term.value.to_string(),
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
            staker_info.reward_claimed = add_reward(staker_info.reward_claimed, amount).unwrap();
            // staker_info.reward_debt = sub_reward(staker_info.reward_debt, amount).unwrap();
            // staker_info.reward_debt = staker_info.reward_debt + pending_reward_staker - amount;

            STAKERS_INFO.save(deps.storage, info.sender, &staker_info)?;

            // update reward total and reward claimed for campaign
            campaign_info.reward_token.amount =
                sub_reward(campaign_info.reward_token.amount, amount).unwrap();
            campaign_info.total_reward_claimed =
                add_reward(campaign_info.total_reward_claimed, amount).unwrap();

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

    // campaing must be ended then withdraw remaining reward
    if campaign_info.end_time > env.block.time.seconds() {
        return Err(ContractError::InvalidTimeToWithdrawReward {});
    }

    // total_pending_reward = previous total reward + total in rates - reward claimed
    let mut total_pending_reward = PREVIOUS_TOTAL_REWARD.load(deps.storage)?;

    // time to calc pending reward
    let current_time = campaign_info.end_time;

    // load TERM_REWARD_RATES
    for term in campaign_info.lockup_term.iter() {
        let mut term_reward_rates = TERM_REWARD_RATES.load(deps.storage, term.value.to_string())?;
        let expiration_times = TERM_EXPIRATION_TIMES.load(deps.storage, term.value.to_string())?;
        let mut total_staking = TOTAL_STAKING_BY_TERM.load(deps.storage, term.value.to_string())?;
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
        .checked_sub(total_pending_reward)
        .unwrap();

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
            campaign_info.reward_token.amount =
                sub_reward(campaign_info.reward_token.amount, withdraw_reward).unwrap();
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
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // max time calc pending reward is campaign_info.end_time
    let mut current_time = env.block.time.seconds();
    if campaign_info.end_time < env.block.time.seconds() {
        current_time = campaign_info.end_time;
    }

    // load TERM_REWARD_RATES
    for term in campaign_info.lockup_term.iter() {
        let mut term_reward_rates = TERM_REWARD_RATES.load(deps.storage, term.value.to_string())?;
        let mut expiration_times =
            TERM_EXPIRATION_TIMES.load(deps.storage, term.value.to_string())?;
        let mut total_staking = TOTAL_STAKING_BY_TERM.load(deps.storage, term.value.to_string())?;

        let token_ids = TOKEN_IDS.load(deps.storage, term.value.to_string())?;

        for token_id in token_ids.iter() {
            let nft_info = NFTS.load(deps.storage, token_id.clone())?;

            let (new_nft_info, new_term_reward_rates, new_total_staking, new_expiration_times) =
                calculate_reward(
                    nft_info,
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
            NFTS.save(deps.storage, token_id.clone(), &new_nft_info)?;
        }
        let updated_term_reward_rates = term_reward_rates
            .last()
            .map_or(Vec::new(), |last_element| vec![last_element.clone()]);

        TERM_REWARD_RATES.save(
            deps.storage,
            term.value.to_string(),
            &updated_term_reward_rates,
        )?;
        TOTAL_STAKING_BY_TERM.save(deps.storage, term.value.to_string(), &total_staking)?;
        TERM_EXPIRATION_TIMES.save(deps.storage, term.value.to_string(), &expiration_times)?;
    }

    Ok(
        Response::new()
            .add_attributes([("action", "reset_pool"), ("admin", config.owner.as_ref())]),
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::CampaignInfo {} => Ok(to_binary(&query_campaign_info(deps)?)?),
        QueryMsg::NftInfo { token_id } => Ok(to_binary(&query_nft_info(deps, env, token_id)?)?),
        QueryMsg::NftStaked { owner } => Ok(to_binary(&query_staker_info(deps, env, owner)?)?),
        QueryMsg::TotalPendingReward {} => Ok(to_binary(&query_total_pending_reward(deps, env)?)?),
        QueryMsg::TokenIds { term_value } => Ok(to_binary(&query_token_ids(deps, term_value)?)?),
        QueryMsg::TermRewardRates { term_value } => {
            Ok(to_binary(&query_term_reward_rates(deps, term_value)?)?)
        }
    }
}

fn query_campaign_info(deps: Deps) -> Result<CampaignInfoResult, ContractError> {
    let campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;
    let mut total_nft_staked: u64 = 0;

    for term in campaign_info.lockup_term.iter() {
        total_nft_staked = total_nft_staked
            .saturating_add(TOKEN_IDS.load(deps.storage, term.value.to_string())?.len() as u64);
    }

    let campaign_result = CampaignInfoResult {
        owner: campaign_info.owner,
        campaign_name: campaign_info.campaign_name,
        campaign_image: campaign_info.campaign_image,
        campaign_description: campaign_info.campaign_description,
        start_time: campaign_info.start_time,
        end_time: campaign_info.end_time,
        total_reward_claimed: campaign_info.total_reward_claimed,
        total_reward: campaign_info.total_reward,
        limit_per_staker: campaign_info.limit_per_staker,
        reward_token_info: campaign_info.reward_token,
        allowed_collection: campaign_info.allowed_collection,
        lockup_term: campaign_info.lockup_term,
        reward_per_second: campaign_info.reward_per_second,
        total_nft_staked,
    };
    Ok(campaign_result)
}

fn query_nft_info(deps: Deps, env: Env, token_id: String) -> Result<NftInfo, ContractError> {
    let campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;
    let nft_info: NftInfo = NFTS.load(deps.storage, token_id)?;

    let mut current_time = env.block.time.seconds();
    if campaign_info.end_time < env.block.time.seconds() {
        current_time = campaign_info.end_time;
    }

    let term_reward_rates =
        TERM_REWARD_RATES.load(deps.storage, nft_info.lockup_term.value.to_string())?;

    let total_staking =
        TOTAL_STAKING_BY_TERM.load(deps.storage, nft_info.lockup_term.value.to_string())?;

    let expiration_times =
        TERM_EXPIRATION_TIMES.load(deps.storage, nft_info.lockup_term.value.to_string())?;

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

fn query_staker_info(
    deps: Deps,
    _env: Env,
    owner: Addr,
) -> Result<StakerRewardAssetInfo, ContractError> {
    let staker_asset: StakerRewardAssetInfo = STAKERS_INFO.may_load(deps.storage, owner)?.unwrap();

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
        let mut term_reward_rates = TERM_REWARD_RATES.load(deps.storage, term.value.to_string())?;
        let expiration_times = TERM_EXPIRATION_TIMES.load(deps.storage, term.value.to_string())?;
        let mut total_staking = TOTAL_STAKING_BY_TERM.load(deps.storage, term.value.to_string())?;
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

fn query_token_ids(deps: Deps, term_value: u64) -> Result<Vec<String>, ContractError> {
    let token_ids = TOKEN_IDS.load(deps.storage, term_value.to_string())?;

    Ok(token_ids)
}

fn query_term_reward_rates(deps: Deps, term_value: u64) -> Result<Vec<RewardRate>, ContractError> {
    let reward_rates = TERM_REWARD_RATES.load(deps.storage, term_value.to_string())?;

    Ok(reward_rates)
}
