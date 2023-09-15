const { exec } = require("child_process");

// for (let i = 221; i < 2200; i++) {
//     setInterval(() => {
//         exec(
//             `beaker wasm execute cw721_metadata_onchain --raw '{"mint":{"token_id":"${i}", "owner": "aura1y9ts00xzxywqx3dxcyyg2xsqd8edackqcr9glm", "token_uri":"ipfs://QmNjiRJ4UUMWjZ7cBQus7GgaNYLAbvN1vvjsqSrbXEwUVE/100.json",  "extension": {"image": null, "image_data": null,"external_url": null, "description": null,"name": null,"attributes": null,"background_color": null,"animation_url": null, "youtube_url": null}}}' --signer-account signer --network euphoria`,
//             (error, stdout, stderr) => {
//                 // if (error) {
//                 //     console.error(`exec error: ${error}`);
//                 //     return;
//                 // }
//                 console.log(`stdout: ${stdout}`);
//                 // console.error(`stderr: ${stderr}`);
//             }
//         );
//     }, 1000);
// }
// let i = 100000;
// while (true) {
//     console.log(i);

//     exec(
//         `beaker wasm execute cw721_metadata_onchain --raw '{"mint":{"token_id":"${i.toString()}", "owner": "aura1y9ts00xzxywqx3dxcyyg2xsqd8edackqcr9glm", "token_uri":"ipfs://QmNjiRJ4UUMWjZ7cBQus7GgaNYLAbvN1vvjsqSrbXEwUVE/100.json",  "extension": {"image": null, "image_data": null,"external_url": null, "description": null,"name": null,"attributes": null,"background_color": null,"animation_url": null, "youtube_url": null}}}' --signer-account signer --network euphoria`,
//         (error, stdout, stderr) => {
//             // if (error) {
//             //     console.error(`exec error: ${error}`);
//             //     return;
//             // }
//             console.log(`stdout: ${stdout}`);
//             // console.error(`stderr: ${stderr}`);
//         }
//     );
//     i = i + 1;
// }

// exec(
//     `beaker wasm execute cw721_metadata_onchain --raw '{"mint":{"token_id":"111114", "owner": "aura1qtd5f6ssp730e9mgy8rdhu58xvxp3hpd9ld5th", "token_uri":"ipfs://QmNjiRJ4UUMWjZ7cBQus7GgaNYLAbvN1vvjsqSrbXEwUVE/100.json",  "extension": {"image": null, "image_data": null,"external_url": null, "description": null,"name": null,"attributes": null,"background_color": null,"animation_url": null, "youtube_url": null}}}' --signer-account signer --network euphoria`,
//     (error, stdout, stderr) => {
//         // if (error) {
//         //     console.error(`exec error: ${error}`);
//         //     return;
//         // }
//         console.log(`stdout: ${stdout}`);
//         // console.error(`stderr: ${stderr}`);
//     }
// );

// let count = 120000;
// setInterval(() => {
//     console.log(count);

//     exec(
//         `beaker wasm execute cw721_metadata_onchain --raw '{"mint":{"token_id":"1${count.toString()}", "owner": "aura1y9ts00xzxywqx3dxcyyg2xsqd8edackqcr9glm", "token_uri":"ipfs://QmNjiRJ4UUMWjZ7cBQus7GgaNYLAbvN1vvjsqSrbXEwUVE/100.json",  "extension": {"image": null, "image_data": null,"external_url": null, "description": null,"name": null,"attributes": null,"background_color": null,"animation_url": null, "youtube_url": null}}}' --signer-account signer --network euphoria`,
//         (error, stdout, stderr) => {
//             // if (error) {
//             //     console.error(`exec error: ${error}`);
//             //     return;
//             // }
//             console.log(`stdout: ${stdout}`);
//             // console.error(`stderr: ${stderr}`);
//         }
//     );
//     count++;
// }, 1000);

let arrRewardRate = [];
let arrEndTime = [];
let reward_per_second = 10;
let term = 10;
let total_nft = 0;

// stake nft 1
let stake1 = {
    start: 10,
    end: 20,
};

total_nft += 1;
let temp = arrEndTime.filter((item) => item > stake1.start);
total_nft -= arrEndTime.length - temp.length;
arrEndTime = temp;

arrRewardRate.push({ timestamp: stake1.start, rate: reward_per_second / total_nft });
arrEndTime.push(stake1.end);

// stake nft 2
let stake2 = {
    start: 15,
    end: 25,
};

total_nft += 1;
let temp2 = arrEndTime.filter((item) => item > stake2.start);
total_nft -= arrEndTime.length - temp2.length;
arrEndTime = temp2;

arrRewardRate.push({ timestamp: stake2.start, rate: reward_per_second / total_nft });
arrEndTime.push(stake2.end);

// stake nft 3
let stake3 = {
    start: 20,
    end: 30,
};

total_nft += 1;
let temp3 = arrEndTime.filter((item) => item > stake3.start);
total_nft -= arrEndTime.length - temp3.length;
arrEndTime = temp3;

arrRewardRate.push({ timestamp: stake3.start, rate: reward_per_second / total_nft });
arrEndTime.push(stake3.end);

// stake nft 4
let stake4 = {
    start: 25,
    end: 35,
};

total_nft += 1;
let temp4 = arrEndTime.filter((item) => item > stake4.start);
total_nft -= arrEndTime.length - temp4.length;
arrEndTime = temp4;

arrRewardRate.push({ timestamp: stake4.start, rate: reward_per_second / total_nft });
arrEndTime.push(stake4.end);

// stake nft 5
let stake5 = {
    start: 40,
    end: 50,
};

total_nft += 1;

let temp5 = arrEndTime.filter((item) => item > stake5.start);
total_nft -= arrEndTime.length - temp5.length;
arrEndTime = temp5;

arrRewardRate.push({ timestamp: stake5.start, rate: reward_per_second / total_nft });
arrEndTime.push(stake5.end);

function calculateReward(timestamp, stake, arrRewardRate) {
    let reward = 0;

    // Sắp xếp arrRewardRate theo thời gian tăng dần
    arrRewardRate.sort((a, b) => a.timestamp - b.timestamp);

    // Tìm khoảng thời gian đặt cọc của NFT
    let stakeStart = stake.start;
    let stakeEnd = Math.min(stake.end, timestamp);

    // Nếu mốc thời gian yêu cầu trước khi NFT được đặt cọc, trả về 0
    if (timestamp < stakeStart) return 0;

    let lastRateTimestamp = stakeStart;

    // Duyệt qua mỗi mốc thời gian có sự thay đổi về tỷ lệ phần thưởng
    for (let i = 0; i < arrRewardRate.length; i++) {
        let rateObj = arrRewardRate[i];

        if (rateObj.timestamp >= stakeStart && rateObj.timestamp <= stakeEnd) {
            let nextTimestamp = i + 1 < arrRewardRate.length ? arrRewardRate[i + 1].timestamp : stakeEnd;
            let duration = Math.min(nextTimestamp, stakeEnd) - rateObj.timestamp;
            reward += duration * rateObj.rate;
        }
    }

    return reward;
}

// Sử dụng function
let rewardForNFT4 = calculateReward(500, stake1, arrRewardRate);
console.log(rewardForNFT4);

console.log({ arrEndTime });
console.log({ arrRewardRate });
console.log({ total_nft });
