#![cfg(test)]
mod tests {
    const MOCK_1000_TOKEN_AMOUNT: u128 = 1_000_000;
   
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

            // Approve cw20 token to contract
            let approve_msg: Cw20ExecuteMsg = Cw20ExecuteMsg::IncreaseAllowance {
                spender: "contract1".to_string(), // Contract
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

            // check reward token in contract
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

            // query balance of contract in cw20 base token contract
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
