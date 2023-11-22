const chainConfig = require("./config/chain").defaultChain;

const fs = require("fs");

const { SigningCosmWasmClient } = require("@cosmjs/cosmwasm-stargate");
const { DirectSecp256k1HdWallet, coin } = require("@cosmjs/proto-signing");
const { calculateFee, GasPrice } = require("@cosmjs/stargate");

// wasm folder
const wasmFolder = `${__dirname}/../artifacts`;

// gas price
const gasPrice = GasPrice.fromString(`0.025${chainConfig.denom}`);
// tester and deployer info
let testerWallet, testerClient, testerAccount;
let deployerWallet, deployerClient, deployerAccount;

/// @dev Store the contract source code on chain
/// @param `wasm_name` - The name of the wasm file
/// @return `storeCodeResponse` - The response of the store code transaction
async function store_contract(wasm_name) {
    const uploadFee = calculateFee(3000000, gasPrice);
    const contractCode = fs.readFileSync(`${wasmFolder}/${wasm_name}.wasm`);

    console.log("Uploading contract code...");
    const storeCodeResponse = await deployerClient.upload(
        deployerAccount.address,
        contractCode,
        uploadFee,
        "Upload campaign contract code"
    );

    console.log("  transactionHash: ", storeCodeResponse.transactionHash);
    console.log("  codeId: ", storeCodeResponse.codeId);
    console.log("  gasWanted / gasUsed: ", storeCodeResponse.gasWanted, " / ", storeCodeResponse.gasUsed);

    return storeCodeResponse;
}

/// @dev Instantiate contract base on the code id and instantiate message of contract
/// @param `_codeID` - The code id of the contract
/// @param `instantiateMsg` - The instantiate message of the contract
/// @return `instantiateResponse` - The response of the instantiate transaction
async function instantiate(contract_code_id, instantiateMsg) {
    console.log("Instantiating contract...");

    //Instantiate the contract
    const instantiateResponse = await deployerClient.instantiate(
        deployerAccount.address,
        Number(contract_code_id),
        instantiateMsg,
        "instantiation contract",
        "auto"
    );
    console.log("  transactionHash: ", instantiateResponse.transactionHash);
    console.log("  contractAddress: ", instantiateResponse.contractAddress);
    console.log("  gasWanted / gasUsed: ", instantiateResponse.gasWanted, " / ", instantiateResponse.gasUsed);

    return instantiateResponse;
}

/// @dev Execute a message to the contract
/// @param `userClient` - The client of the user who execute the message
/// @param `userAccount` -  The account of the user who execute the message
/// @param `contract` - The address of the contract
/// @param `executeMsg` - The message that will be executed
/// @return `executeResponse` - The response of the execute transaction
async function execute(
    userClient,
    userAccount,
    contract,
    executeMsg,
    native_amount = 0,
    native_denom = chainConfig.denom
) {
    console.log("Executing message to contract...");

    const memo = "execute a message";

    let executeResponse;

    // if the native amount is not 0, then send the native token to the contract
    if (native_amount != 0) {
        executeResponse = await userClient.execute(userAccount.address, contract, executeMsg, "auto", memo, [
            coin(native_amount, native_denom),
        ]);
    } else {
        executeResponse = await userClient.execute(userAccount.address, contract, executeMsg, "auto", memo);
    }

    console.log("  transactionHash: ", executeResponse.transactionHash);
    console.log("  gasWanted / gasUsed: ", executeResponse.gasWanted, " / ", executeResponse.gasUsed);

    return executeResponse;
}

/// @dev Query information from the contract
/// @param `userClient` - The client of the user who execute the message
/// @param `contract` - The address of the contract
/// @param `queryMsg` - The message that will be executed
/// @return `queryResponse` - The response of the query
async function query(userClient, contract, queryMsg) {
    console.log("Querying contract...");

    const queryResponse = await userClient.queryContractSmart(contract, queryMsg);

    console.log("  Querying successful");

    return queryResponse;
}

async function main(contract_name) {
    // ***************************
    // SETUP INFORMATION FOR USERS
    // ***************************
    // connect deployer wallet to chain and get deployer account
    deployerWallet = await DirectSecp256k1HdWallet.fromMnemonic(chainConfig.deployer_mnemonic, {
        prefix: chainConfig.prefix,
    });
    // console.log({ deployerWallet });
    deployerClient = await SigningCosmWasmClient.connectWithSigner(chainConfig.rpcEndpoint, deployerWallet, {
        gasPrice,
    });

    // console.log({ deployerClient });
    deployerAccount = (await deployerWallet.getAccounts())[0];
    // console.log({ deployerAccount });

    // connect tester wallet to chain and get tester account
    testerWallet = await DirectSecp256k1HdWallet.fromMnemonic(chainConfig.tester_mnemonic, {
        prefix: chainConfig.prefix,
    });
    testerClient = await SigningCosmWasmClient.connectWithSigner(chainConfig.rpcEndpoint, testerWallet, { gasPrice });
    testerAccount = (await testerWallet.getAccounts())[0];

    // ****************
    // EXECUTE CONTRACT
    // ****************
    // store contract
    console.log("1. Storing source code...");
    let storeCodeResponse = await store_contract(contract_name);

    // prepare instantiate message
    const instantiateMsg = {
        owner: "aura1y9ts00xzxywqx3dxcyyg2xsqd8edackqcr9glm",
        campaign_code_id: 1555,
        allow_create_for_all: false,
    };

    // prepare instantiate message
    // const instantiateMsg = {
    //     name: "GolfBall",
    //     symbol: "GBALL",
    //     decimals: 18,
    //     initial_balances: [
    //         {
    //             amount: "500000000000000000000000",
    //             address: "aura19he3exuu8kz5kcshaw74uhn4s9ppx3dykrrwda",
    //         },
    //     ],
    //     mint: {
    //         minter: "aura19he3exuu8kz5kcshaw74uhn4s9ppx3dykrrwda",
    //     },
    //     marketing: {
    //         logo: {
    //             url: "https://bafybeifkrodjtle4xgh3nk6saigy7cms4rx7e24p3ydswbieavkmx3f5vm.ipfs.nftstorage.link/photo_2023-09-27_19-44-07.jpg",
    //         },
    //     },
    // };

    // // instantiate contract
    // console.log("2. Instantiating contract...");
    // let instantiateResponse = await instantiate(17, instantiateMsg);

    console.log("Contract setup completed!");
}

const myArgs = process.argv.slice(2);
if (myArgs.length != 1) {
    console.log("Usage: node contract_setup.js <wasm_contract_name>");
    process.exit(1);
}
main(myArgs[0]);
