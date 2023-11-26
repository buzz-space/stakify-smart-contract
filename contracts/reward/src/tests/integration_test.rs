#![cfg(test)]
mod tests {
    const MOCK_1000_TOKEN_AMOUNT: u128 = 1_000_000;
    // 1. create token contract, collection contract
    // 2. create factory contract
    // 3. create campaign by factory contract
    // 4. add reward by token contract
    // 5. stake nft by collection contract
    // 6. claim reward
    // 7. unstake nft
    // 8. withdraw remaining reward
    mod execute_proper_operation {
        use crate::{
            msg::{ExecuteMsg, QueryMsg},
            state::{AssetToken, RewardInfo, StakerRewardAssetInfo, TokenInfo, WatchListReward},
            tests::{
                env_setup::env::{instantiate_contracts, ADMIN, USER_1, USER_2},
                integration_test::tests::MOCK_1000_TOKEN_AMOUNT,
            },
        };

        use cosmwasm_std::{Addr, Uint128};
        use cw20::{BalanceResponse, Cw20ExecuteMsg};
        use cw_multi_test::Executor;

        //         -------------- proper operation ------------------
        // - ADMIN create campaign contract by factory contract
        // - add 1000.000 reward token to campaign by ADMIN
        // - with end time 100s -> reward_per_second = 10.000 token
        // - increase 20s to make active campaign
        // - stake nft token_id 1 with lockup_term = 10s, percent = 30% to campaign by USER_1
        // - token_id 1 -> has time stake: s20 -> s30
        // - increase simulation time more 1s
        // - stake token_id 2 with lockup_term = 10s, percent = 30% -> has time staking: s21 -> s31 -> calculate pending reward token_id 1
        // 	- token_id 1 pending_reward = 1(s) * 10.000(reward_per_second) * 30 / 100 (percent_lockup_term) / 1 (nft_count) = 3.000 token
        // - increase simulation time more 6s
        // 	- token_id 1 pending_reward = 3.000 + 6(s) * 10.000(reward_per_second) * 30 / 100 (percent_lockup_term) / 2 (nft_count) = 12000
        // 	- token_id 2 pending_reward = 6(s) * 10.000(reward_per_second) * 30 / 100 (percent_lockup_term) / 2 (nft_count) = 9000
        // - USER_1 reward_debt = 12.000 + 9.000 = 21.000, reward_claimed = 0
        // - USER_1 claim reward: 21.000
        // 	- token_id 1 pending_reward = 0
        // 	- token_id 2 pending_reward = 0
        // 	- USER_1 reward_debt = 0, reward_claimed = 21.000
        // 	- total_reward = 1000.000 - 21.000 = 979.000
        // - increase simulation time more 3s
        // 	- USER_1 un_stake nft 1
        // 		- calc token_id 1: pending_reward = 0 + 3(s) * 10.000(reward_per_second) * 30 / 100 (percent_lockup_term) / 2 (nft_count) = 4.500	move reward to USER_1, remove token_id 1 from USER_1
        // 		- token_id 2:  pending_reward = 0 + 3(s) * 10.000(reward_per_second) * 30 / 100 (percent_lockup_term) / 2 (nft_count) = 4.500
        // 		- USER_1 reward_debt = 4.500(token_id 1 transerfered) + 4.500 (token_id 2) = 9.000, reward_claimed = 21.000
        // - total_pending_reward = 4.500(token_id 2) + 4.500(reward_debt USER_1) = 9.000
        // - increase simulation time more 80s -> ended campaign
        // - USER_1:
        // 	- token_id 2: pending_reward = 4.500 + 1(s) * 10.000(reward_per_second) * 30 / 100 (percent_lockup_term) / 1 (nft_count) = 7.500
        // 	- USER_1 reward_debt = 4.500(token_id 1 transerfered) + 7.500(token_id 2) = 12.000, reward_claimed = 21.000
        // 	- total_pending_reward = 7.500(token_id 2) + 4.500(reward_debt USER_1) = 12.000
        // - withdraw remaining reward by ADMIN
        // 	- total_pending_reward = 12.000
        // 	- withdraw_reward = 979.000 - 12.000(total_pending_reward) = 967.000
        //  - ADMIN token = 967.000
        // 	- Campaign token = 12.000
        #[test]
        fn proper_operation() {
            // get integration test app and contracts
            let (mut app, contracts) = instantiate_contracts();

            // get lp token contract
            let token_contract = &contracts[0].contract_addr;
            // get collection contract
            let reward_contract = &contracts[1].contract_addr;

            assert_eq!(reward_contract, "contract1");

            // token info
            let token_info = TokenInfo::Token {
                contract_addr: token_contract.to_string(),
            };

            // Mint 1000 tokens to ADMIN
            let mint_msg: Cw20ExecuteMsg = Cw20ExecuteMsg::Mint {
                recipient: ADMIN.to_string(),
                amount: Uint128::from(MOCK_1000_TOKEN_AMOUNT),
            };

            // Execute minting
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(token_contract.clone()),
                &mint_msg,
                &[],
            );

            assert!(response.is_ok());

            // query balance of ADMIN in cw20 base token contract
            let balance: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    token_contract.clone(),
                    &cw20::Cw20QueryMsg::Balance {
                        address: ADMIN.to_string(),
                    },
                )
                .unwrap();
            // It should be 1000 token as minting happened
            assert_eq!(balance.balance, Uint128::from(MOCK_1000_TOKEN_AMOUNT));

            // query contract address
            let reward_info: RewardInfo = app
                .wrap()
                .query_wasm_smart(Addr::unchecked("contract1"), &QueryMsg::RewardInfo {})
                .unwrap();

            assert_eq!(
                reward_info,
                RewardInfo {
                    owner: Addr::unchecked(ADMIN.to_string()),
                    reward_token: AssetToken {
                        info: token_info.clone(),
                        amount: Uint128::zero(),
                    },
                    total_reward: Uint128::zero()
                }
            );

            // Approve cw20 token to campaign contract
            let approve_msg: Cw20ExecuteMsg = Cw20ExecuteMsg::IncreaseAllowance {
                spender: "contract1".to_string(), // Campaign Contract
                amount: Uint128::from(MOCK_1000_TOKEN_AMOUNT),
                expires: None,
            };

            // Execute approve
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(token_contract.clone()),
                &approve_msg,
                &[],
            );

            assert!(response.is_ok());

            // add reward token
            let add_reward_balance_msg = ExecuteMsg::AddRewardToken {
                amount: Uint128::from(MOCK_1000_TOKEN_AMOUNT),
            };

            // Execute add reward balance
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked("contract1"),
                &add_reward_balance_msg,
                &[],
            );

            assert!(response.is_ok());

            // check reward token in campaign
            let reward_info: RewardInfo = app
                .wrap()
                .query_wasm_smart("contract1", &QueryMsg::RewardInfo {})
                .unwrap();

            assert_eq!(
                Uint128::from(MOCK_1000_TOKEN_AMOUNT),
                reward_info.reward_token.amount
            );

            // query balance of ADMIN in cw20 base token contract
            let balance: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    token_contract.clone(),
                    &cw20::Cw20QueryMsg::Balance {
                        address: ADMIN.to_string(),
                    },
                )
                .unwrap();

            // It should be 0 token as deposit happened
            assert_eq!(balance.balance, Uint128::zero());

            // query balance of campaign contract in cw20 base token contract
            let balance: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    token_contract.clone(),
                    &cw20::Cw20QueryMsg::Balance {
                        address: "contract1".to_string(),
                    },
                )
                .unwrap();

            // It should be MOCK_1000_TOKEN_AMOUNT token as deposit happened
            assert_eq!(balance.balance, Uint128::from(MOCK_1000_TOKEN_AMOUNT));

            // set watch list
            let watch_list_msg = ExecuteMsg::SetWatchListReward {
                watch_list: vec![
                    WatchListReward {
                        address: USER_1.to_string(),
                        reward_debt: Uint128::from(1000u128),
                        reward_claimed: Uint128::zero(),
                    },
                    WatchListReward {
                        address: USER_2.to_string(),
                        reward_debt: Uint128::from(2000u128),
                        reward_claimed: Uint128::zero(),
                    },
                ],
            };

            // Execute msg
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked("contract1"),
                &watch_list_msg,
                &[],
            );

            assert!(response.is_ok());

            let watch_list: Vec<Addr> = app
                .wrap()
                .query_wasm_smart("contract1", &QueryMsg::WatchList {})
                .unwrap();

            assert_eq!(
                watch_list,
                vec![
                    Addr::unchecked(USER_1.to_string()),
                    Addr::unchecked(USER_2.to_string())
                ]
            );

            // staker_info USER_1
            let staker_info: StakerRewardAssetInfo = app
                .wrap()
                .query_wasm_smart(
                    "contract1",
                    &QueryMsg::StakerInfo {
                        address: Addr::unchecked(USER_1.to_string()),
                    },
                )
                .unwrap();

            assert_eq!(staker_info, StakerRewardAssetInfo{
                reward_debt: Uint128::from(1000u128),
                reward_claimed: Uint128::zero()
            });

            // staker_info USER_2
            let staker_info: StakerRewardAssetInfo = app
                .wrap()
                .query_wasm_smart(
                    "contract1",
                    &QueryMsg::StakerInfo {
                        address: Addr::unchecked(USER_2.to_string()),
                    },
                )
                .unwrap();

            assert_eq!(staker_info, StakerRewardAssetInfo{
                reward_debt: Uint128::from(2000u128),
                reward_claimed: Uint128::zero()
            });

            // transer reward watch list
            let transfer_reward_msg = ExecuteMsg::TransferReward { watch_list: vec![
                Addr::unchecked(USER_1.to_string()),
                Addr::unchecked(USER_2.to_string())
            ] };

            // Execute msg
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked("contract1"),
                &transfer_reward_msg,
                &[],
            );

            assert!(response.is_ok());

            // staker_info USER_1
            let staker_info: StakerRewardAssetInfo = app
                .wrap()
                .query_wasm_smart(
                    "contract1",
                    &QueryMsg::StakerInfo {
                        address: Addr::unchecked(USER_1.to_string()),
                    },
                )
                .unwrap();

            assert_eq!(staker_info, StakerRewardAssetInfo{
                reward_debt: Uint128::zero(),
                reward_claimed: Uint128::from(1000u128)
            });

            // staker_info USER_2
            let staker_info: StakerRewardAssetInfo = app
                .wrap()
                .query_wasm_smart(
                    "contract1",
                    &QueryMsg::StakerInfo {
                        address: Addr::unchecked(USER_2.to_string()),
                    },
                )
                .unwrap();

            assert_eq!(staker_info, StakerRewardAssetInfo{
                reward_debt: Uint128::zero(),
                reward_claimed: Uint128::from(2000u128)
            });

            // get balance reward
            let balance: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    token_contract.clone(),
                    &cw20::Cw20QueryMsg::Balance {
                        address: USER_1.to_string(),
                    },
                )
                .unwrap();
            
            // reward USER_1
            assert_eq!(balance.balance, Uint128::from(1000u128));

            // get balance reward
            let balance: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    token_contract.clone(),
                    &cw20::Cw20QueryMsg::Balance {
                        address: USER_2.to_string(),
                    },
                )
                .unwrap();
            
            // reward USER_1
            assert_eq!(balance.balance, Uint128::from(2000u128));

            // get balance reward
            let balance: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    token_contract.clone(),
                    &cw20::Cw20QueryMsg::Balance {
                        address: "contract1".to_string(),
                    },
                )
                .unwrap();
            
            // reward contract
            assert_eq!(balance.balance, Uint128::from(997000u128));

            // query contract info
            let reward_info: RewardInfo = app
                .wrap()
                .query_wasm_smart(Addr::unchecked("contract1"), &QueryMsg::RewardInfo {})
                .unwrap();

            assert_eq!(
                reward_info,
                RewardInfo {
                    owner: Addr::unchecked(ADMIN.to_string()),
                    reward_token: AssetToken {
                        info: token_info.clone(),
                        amount: Uint128::from(997000u128),
                    },
                    total_reward: Uint128::from(MOCK_1000_TOKEN_AMOUNT)
                }
            );

        }
    }
}
