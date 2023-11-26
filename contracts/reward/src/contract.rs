#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
      Uint128, WasmMsg, 
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{
    AssetToken, RewardInfo, StakerRewardAssetInfo, TokenInfo, WatchListReward, REWARD_INFO,
    STAKERS_INFO, WATCH_LIST,
};
use cw20::Cw20ExecuteMsg;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:reward";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

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

    // reward info
    let reward = RewardInfo {
        owner: deps.api.addr_validate(&msg.owner).unwrap(),
        reward_token: AssetToken {
            info: msg.reward_token_info.info.clone(),
            amount: Uint128::zero(),
        },
        total_reward: Uint128::zero(),
    };

    // save reward info
    REWARD_INFO.save(deps.storage, &reward)?;

    // create watch list
    WATCH_LIST.save(deps.storage, &vec![])?;

    // we need emit the information of reward token to response
    let reward_token_info_str = match msg.reward_token_info.info {
        TokenInfo::Token { contract_addr } => contract_addr,
        TokenInfo::NativeToken { denom } => denom,
    };

    // emit the information of instantiated reward
    Ok(Response::new().add_attributes([
        ("action", "instantiate"),
        ("owner", &msg.owner),
        ("reward_token_info", &reward_token_info_str),
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
        ExecuteMsg::SetWatchListReward { watch_list } => {
            execute_set_watch_list(deps, env, info, watch_list)
        }
        ExecuteMsg::TransferReward { watch_list } => {
            execute_transfer_reward(deps, env, info, watch_list)
        }
    }
}

pub fn execute_add_reward_token(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // load reward info
    let mut reward_info: RewardInfo = REWARD_INFO.load(deps.storage)?;

    // only owner can add reward token to reward
    if reward_info.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let mut res = Response::new();

    // we need determine the reward token is native token or cw20 token
    match reward_info.reward_token.info.clone() {
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

            // update amount, reward_per_second token in reward
            reward_info.reward_token.amount =
                reward_info.reward_token.amount.saturating_add(amount);

            reward_info.total_reward = reward_info.total_reward.saturating_add(amount);

            // save reward
            REWARD_INFO.save(deps.storage, &reward_info)?;
        }
        TokenInfo::NativeToken { denom: _ } => {}
    }
    Ok(res.add_attributes([
        ("action", "add_reward_token"),
        ("owner", reward_info.owner.as_ref()),
        ("reward_token_amount", &amount.to_string()),
    ]))
}

pub fn execute_set_watch_list(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    watch_list: Vec<WatchListReward>,
) -> Result<Response, ContractError> {
    // load reward info
    let reward_info: RewardInfo = REWARD_INFO.load(deps.storage)?;
    let mut watch_list_info: Vec<Addr> = WATCH_LIST.load(deps.storage)?;

    // only owner can add reward token to reward
    if reward_info.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    for staker in &watch_list {
        let staker_address = deps.api.addr_validate(&staker.address).unwrap();

        let mut staker_info = STAKERS_INFO
            .load(deps.storage, staker_address.clone())
            .unwrap_or(StakerRewardAssetInfo {
                reward_debt: Uint128::zero(),
                reward_claimed: Uint128::zero(),
            });
        staker_info.reward_debt = staker.reward_debt;
        staker_info.reward_claimed = staker.reward_claimed;

        watch_list_info.push(staker_address.clone());
        STAKERS_INFO.save(deps.storage, staker_address, &staker_info)?;
    }

    WATCH_LIST.save(deps.storage, &watch_list_info)?;

    Ok(Response::new().add_attributes([
        ("action", "set_watch_list"),
        ("owner", &reward_info.owner.to_string()),
        ("watch_list", &format!("{:?}", &watch_list)),
    ]))
}

pub fn execute_transfer_reward(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    watch_list: Vec<Addr>,
) -> Result<Response, ContractError> {
    // load reward info
    let mut reward_info: RewardInfo = REWARD_INFO.load(deps.storage)?;

    // only owner can add reward token to reward
    if reward_info.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let mut res = Response::new();

    let mut total_reward_transfer: Uint128 = Uint128::zero();

    for addr in watch_list.clone() {
        if STAKERS_INFO.may_load(deps.storage, addr.clone())?.is_none() {
            return Err(ContractError::InvalidWatchList {});
        }

        let mut staker_info = STAKERS_INFO.load(deps.storage, addr.clone())?;

        match reward_info.reward_token.info.clone() {
            TokenInfo::Token { contract_addr } => {
                // execute cw20 transfer msg from info.sender to contract
                res = res.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: contract_addr.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Transfer {
                        recipient: addr.clone().to_string(),
                        amount: staker_info.reward_debt.clone(),
                    })?,
                    funds: vec![],
                }));

                // add token info to response
                res = res.add_attributes([
                    ("address", &addr.to_string()),
                    ("reward", &staker_info.reward_debt.to_string()),
                ]);

                total_reward_transfer = total_reward_transfer.saturating_add(staker_info.reward_debt);

                staker_info.reward_claimed = staker_info
                    .reward_claimed
                    .saturating_add(staker_info.reward_debt);
                staker_info.reward_debt = Uint128::zero();
            }
            TokenInfo::NativeToken { denom: _ } => {}
        }
        STAKERS_INFO.save(deps.storage, addr, &staker_info)?;
    }

    reward_info.reward_token.amount = reward_info.reward_token.amount.saturating_sub(total_reward_transfer);

    REWARD_INFO.save(deps.storage, &reward_info)?;

    Ok(res.add_attributes([
        ("action", "transfer_reward"),
        ("owner", &reward_info.owner.to_string()),
    ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::RewardInfo {} => Ok(to_binary(&query_reward_info(deps)?)?),
        QueryMsg::WatchList {} => Ok(to_binary(&query_watch_list(deps)?)?),
        QueryMsg::StakerInfo { address } => Ok(to_binary(&query_staker_info(deps, address)?)?),
    }
}

fn query_reward_info(deps: Deps) -> Result<RewardInfo, ContractError> {
    let reward_info: RewardInfo = REWARD_INFO.load(deps.storage)?;

    Ok(reward_info)
}

fn query_watch_list(deps: Deps) -> Result<Vec<Addr>, ContractError> {
    let watch_list = WATCH_LIST.load(deps.storage)?;

    Ok(watch_list)
}

fn query_staker_info(deps: Deps, address: Addr) -> Result<StakerRewardAssetInfo, ContractError> {
    let staker_info = STAKERS_INFO.load(deps.storage, address)?;

    Ok(staker_info)
}
