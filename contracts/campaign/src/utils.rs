use cosmwasm_std::{DivideByZeroError, OverflowError, Uint128};

use crate::state::{NftInfo, RewardRate};

/// Calculates the reward amount
pub fn add_reward(current_reward: Uint128, calc_reward: Uint128) -> Result<Uint128, OverflowError> {
    current_reward.checked_add(calc_reward)
}

pub fn sub_reward(current_reward: Uint128, calc_reward: Uint128) -> Result<Uint128, OverflowError> {
    current_reward.checked_sub(calc_reward)
}

pub fn calc_reward_in_time(
    start_time: u64,
    end_time: u64,
    reward_per_second: Uint128,
    percent: Uint128,
    nft_count: u128,
) -> Result<Uint128, DivideByZeroError> {
    let diff_time = end_time.checked_sub(start_time).unwrap();

    let mul_reward = Uint128::from(diff_time)
        .checked_mul(reward_per_second)
        .and_then(|res| res.checked_mul(percent))
        .unwrap();

    let divisor = Uint128::from(100u128)
        .checked_mul(Uint128::from(nft_count))
        .unwrap();

    mul_reward.checked_div(divisor)
}

pub fn update_reward_rate(
    mut arr_reward_rate: Vec<RewardRate>,
    total: u64,
    timestamp: u64,
    change: i64,
) -> (Vec<RewardRate>, u64) {
    // Ensure the total doesn't go below zero
    let result = (total as i64).saturating_add(change);
    let total_nft = if result <= 0 { 0 } else { result } as u64;

    if let Some(reward) = arr_reward_rate
        .iter_mut()
        .find(|item| item.timestamp == timestamp)
    {
        reward.rate = total_nft;
    } else {
        arr_reward_rate.push(RewardRate {
            timestamp,
            rate: total_nft,
        });
    }

    // Sort arrRewardRate by time in ascending order
    arr_reward_rate.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    (arr_reward_rate, total_nft)
}

pub fn stake_nft(
    expiration_times: Vec<u64>,
    mut arr_reward_rate: Vec<RewardRate>,
    total: u64,
    nft: NftInfo,
    timestamp: u64,
) -> (Vec<u64>, Vec<RewardRate>, u64) {
    let mut new_expiration_times: Vec<u64> = vec![];
    let mut total_nft = total;

    for &end_time in &expiration_times {
        if end_time <= timestamp {
            let (updated_reward_rate, t) =
                update_reward_rate(arr_reward_rate, total_nft, end_time, -1);
            arr_reward_rate = updated_reward_rate;
            total_nft = t;
        } else {
            new_expiration_times.push(end_time);
        }
    }

    new_expiration_times.push(nft.end_time);

    let (final_reward_rate, new_total) =
        update_reward_rate(arr_reward_rate, total_nft, timestamp, 1);

    (new_expiration_times, final_reward_rate, new_total)
}

pub fn calculate_reward(
    mut nft: NftInfo,
    mut term_reward_rates: Vec<RewardRate>,
    expiration_times: Vec<u64>,
    total: u64,
    current_time: u64,
    end_time_campaign: u64,
    reward_per_second: Uint128,
) -> (NftInfo, Vec<RewardRate>, u64, Vec<u64>) {
    if nft.is_end_reward {
        return (nft, term_reward_rates, total, expiration_times);
    }
    let mut new_expiration_times: Vec<u64> = vec![];
    let mut total_nft = total;

    for &end_time in &expiration_times {
        if end_time <= current_time {
            let (updated_reward_rate, t) =
                update_reward_rate(term_reward_rates, total_nft, end_time, -1);
            term_reward_rates = updated_reward_rate;
            total_nft = t;
        } else {
            new_expiration_times.push(end_time);
        }
    }

    let (final_reward_rate, _) = update_reward_rate(term_reward_rates, total_nft, current_time, 0);
    term_reward_rates = final_reward_rate;

    let mut reward: u128 = 0;

    // Sort arrRewardRate by time in ascending order
    term_reward_rates.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    // If the required timeline is before the NFT is staked, return 0
    if current_time < nft.time_calc {
        return (nft, term_reward_rates, total_nft, new_expiration_times);
    }

    let nft_start = nft.time_calc;
    let nft_end = current_time.min(nft.end_time);

    // Browse each timeline with reward rate changes
    for i in 0..term_reward_rates.len() {
        let rate_obj = &term_reward_rates[i];

        if rate_obj.timestamp >= nft_start && rate_obj.timestamp <= nft_end {
            let next_timestamp = if i + 1 < term_reward_rates.len() {
                term_reward_rates[i + 1].timestamp
            } else {
                nft_end
            };
            let duration = (next_timestamp.min(nft_end) - rate_obj.timestamp) as u128;
            if rate_obj.rate != 0 {
                let additional_reward = duration
                    .saturating_mul(reward_per_second.u128())
                    .saturating_mul(nft.lockup_term.percent.u128())
                    .saturating_div(rate_obj.rate as u128)
                    .saturating_div(100u128);
                reward = reward.saturating_add(additional_reward);
            }
        }
    }

    // update pending reward
    nft.pending_reward = add_reward(nft.pending_reward, Uint128::from(reward)).unwrap();

    // update time calc
    nft.time_calc = nft_end;

    // if nft is end reward then update status nft
    if nft.time_calc == nft.end_time || current_time >= end_time_campaign {
        nft.is_end_reward = true;
    }

    (nft, term_reward_rates, total_nft, new_expiration_times)
}
