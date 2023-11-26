#[cfg(test)]
pub mod env {
    use cosmwasm_std::{Addr, Empty, Uint128};
    use cw20::MinterResponse;
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    use cw20_base::contract::{
        execute as Cw20Execute, instantiate as Cw20Instantiate, query as Cw20Query,
    };

    use cw20_base::msg::InstantiateMsg as Cw20InstantiateMsg;

    use crate::contract::{execute as Execute, instantiate as Instantiate, query as Query};

    use crate::msg::InstantiateMsg;
    use crate::state::{AssetToken, TokenInfo};

    pub const ADMIN: &str = "aura1000000000000000000000000000000000admin";
    pub const USER_1: &str = "aura1000000000000000000000000000000000user1";
    pub const USER_2: &str = "aura1000000000000000000000000000000000user2";

    pub struct ContractInfo {
        pub contract_addr: String,
        pub contract_code_id: u64,
    }

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked(ADMIN), vec![])
                .unwrap();
        })
    }

    // reward contract
    fn contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(Execute, Instantiate, Query);
        Box::new(contract)
    }

    // token contract
    // create instantiate message for contract
    fn token_contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(Cw20Execute, Cw20Instantiate, Cw20Query);
        Box::new(contract)
    }

    pub fn instantiate_contracts() -> (App, Vec<ContractInfo>) {
        // Create a new app instance
        let mut app = mock_app();
        // Create a vector to store all contract info ([factory - [0])
        let mut contracts: Vec<ContractInfo> = Vec::new();

        // store code of all contracts to the app and get the code ids
        let token_contract_code_id = app.store_code(token_contract_template());
        let contract_code_id = app.store_code(contract_template());

        // token contract
        // create instantiate message for contract
        let lp_token_instantiate_msg = Cw20InstantiateMsg {
            name: "Token".to_string(),
            symbol: "TTT".to_string(),
            decimals: 3,
            initial_balances: vec![],
            mint: Some(MinterResponse {
                minter: ADMIN.to_string(),
                cap: None,
            }),
            marketing: None,
        };

        // token instantiate contract
        let token_contract_addr = app
            .instantiate_contract(
                token_contract_code_id,
                Addr::unchecked(ADMIN),
                &lp_token_instantiate_msg,
                &[],
                "test instantiate token",
                None,
            )
            .unwrap();

        // add contract info to the vector
        contracts.push(ContractInfo {
            contract_addr: token_contract_addr.to_string(),
            contract_code_id: token_contract_code_id,
        });

        // token info
        let token_info = TokenInfo::Token {
            contract_addr: token_contract_addr.to_string(),
        };

        let contract_instantiate_msg = InstantiateMsg {
            owner: ADMIN.to_string(),
            reward_token_info: AssetToken {
                info: token_info.clone(),
                amount: Uint128::zero(),
            },
        };

        // reward instantiate contract
        let contract_addr = app
            .instantiate_contract(
                contract_code_id,
                Addr::unchecked(ADMIN),
                &contract_instantiate_msg,
                &[],
                "test instantiate contract",
                None,
            )
            .unwrap();

        // add contract info to the vector
        contracts.push(ContractInfo {
            contract_addr: contract_addr.to_string(),
            contract_code_id,
        });

        (app, contracts)
    }
}
