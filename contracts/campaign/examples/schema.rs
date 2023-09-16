use cosmwasm_schema::write_api;

use campaign::{
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{LockupTerm, NftInfo, NftStake, RewardRate},
    utils::{calculate_reward, stake_nft},
};
use cosmwasm_std::{Addr, Uint128};

fn main() {
    // write_api! {
    //     instantiate: InstantiateMsg,
    //     execute: ExecuteMsg,
    //     query: QueryMsg
    // }

    let expiration_times = vec![];

    let arr_reward_rate = vec![];

    let total = 0;

    // Gọi hàm stake_nft
    let (new_expiration_times, new_arr_reward_rate, new_total) = stake_nft(
        expiration_times,
        arr_reward_rate,
        total,
        NftStake {
            token_id: '2'.to_string(),
            lockup_term: 10,
        },
        10u64,
    );
    //1

    // Gọi hàm stake_nft
    let (new_expiration_times, new_arr_reward_rate, new_total) = stake_nft(
        new_expiration_times,
        new_arr_reward_rate,
        new_total,
        NftStake {
            token_id: '3'.to_string(),
            lockup_term: 10,
        },
        15u64,
    );
    //2

    // Gọi hàm stake_nft
    let (new_expiration_times, new_arr_reward_rate, new_total) = stake_nft(
        new_expiration_times,
        new_arr_reward_rate,
        new_total,
        NftStake {
            token_id: '4'.to_string(),
            lockup_term: 10,
        },
        20u64,
    );
    //2

    // Gọi hàm stake_nft
    let (new_expiration_times, new_arr_reward_rate, new_total) = stake_nft(
        new_expiration_times,
        new_arr_reward_rate,
        new_total,
        NftStake {
            token_id: '5'.to_string(),
            lockup_term: 10,
        },
        25u64,
    );
    //2

    // Gọi hàm stake_nft
    let (new_expiration_times, new_arr_reward_rate, new_total) = stake_nft(
        new_expiration_times,
        new_arr_reward_rate,
        new_total,
        NftStake {
            token_id: '6'.to_string(),
            lockup_term: 10,
        },
        40u64,
    );
    //35:0, 40:1

    // Gọi hàm stake_nft
    let (new_expiration_times, new_arr_reward_rate, new_total) = stake_nft(
        new_expiration_times,
        new_arr_reward_rate,
        new_total,
        NftStake {
            token_id: '7'.to_string(),
            lockup_term: 10,
        },
        42u64,
    );
    // Gọi hàm stake_nft
    let (new_expiration_times, new_arr_reward_rate, new_total) = stake_nft(
        new_expiration_times,
        new_arr_reward_rate,
        new_total,
        NftStake {
            token_id: '8'.to_string(),
            lockup_term: 10,
        },
        55u64,
    );

    // Gọi hàm stake_nft
    let (new_expiration_times, new_arr_reward_rate, new_total) = stake_nft(
        new_expiration_times,
        new_arr_reward_rate,
        new_total,
        NftStake {
            token_id: '9'.to_string(),
            lockup_term: 10,
        },
        55u64,
    );

    // Gọi hàm stake_nft
    let (new_expiration_times, new_arr_reward_rate, new_total) = stake_nft(
        new_expiration_times,
        new_arr_reward_rate,
        new_total,
        NftStake {
            token_id: "10".to_string(),
            lockup_term: 10,
        },
        55u64,
    );

    // Gọi hàm stake_nft
    let (new_expiration_times, new_arr_reward_rate, new_total) = stake_nft(
        new_expiration_times,
        new_arr_reward_rate,
        new_total,
        NftStake {
            token_id: "11".to_string(),
            lockup_term: 10,
        },
        56u64,
    );

    // Gọi hàm stake_nft
    let (new_expiration_times, new_arr_reward_rate, new_total) = stake_nft(
        new_expiration_times,
        new_arr_reward_rate,
        new_total,
        NftStake {
            token_id: "12".to_string(),
            lockup_term: 10,
        },
        70u64,
    );

    let nft = calculate_reward(
        NftInfo {
            token_id: "7".to_string(),
            owner: Addr::unchecked("input"),
            lockup_term: LockupTerm {
                value: 10u64,
                percent: Uint128::from(10u128),
            },
            end_time: 52u64,
            start_time: 42u64,
            is_end_reward: false,
            pending_reward: Uint128::from(0u128),
        },
        new_arr_reward_rate.clone(),
        50u64,
        Uint128::from(10u128),
    );
    // In ra kết quả
    println!("New Expiration Times: {:?}", new_expiration_times);
    println!("New Reward Rates: {:?}", new_arr_reward_rate);
    println!("New Total: {}", new_total);
    println!("Reward: {}", nft.pending_reward);
}
