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

let count = 1;
setInterval(() => {
    console.log(count);

    exec(
        `beaker wasm execute cw721_metadata_onchain --raw '{"mint":{"token_id":"Buzz${count.toString()}", "owner": "aura1y9ts00xzxywqx3dxcyyg2xsqd8edackqcr9glm", "token_uri":"ipfs://QmNjiRJ4UUMWjZ7cBQus7GgaNYLAbvN1vvjsqSrbXEwUVE/100.json",  "extension": {"image": null, "image_data": null,"external_url": null, "description": null,"name": null,"attributes": null,"background_color": null,"animation_url": null, "youtube_url": null}}}' --signer-account signer --network euphoria`,
        (error, stdout, stderr) => {
            // if (error) {
            //     console.error(`exec error: ${error}`);
            //     return;
            // }
            console.log(`stdout: ${stdout}`);
            // console.error(`stderr: ${stderr}`);
        }
    );
    count++;
}, 1000);

// let arrRewardRate = [];
// let arrEndTime = [];
// let reward_per_second = 10;
// let term = 10;
// let total_nft = 0;

// // stake nft 1
// let stake1 = {
//     start: 10,
//     end: 20,
// };
// arrEndTime.forEach((time) => {
//     if (time <= stake1.start) {
//         total_nft -= 1;
//         arrRewardRate.push({ timestamp: time, rate: total_nft == 0 ? 0 : reward_per_second / total_nft });
//     }
// });

// total_nft += 1;
// let temp = arrEndTime.filter((item) => item > stake1.start);
// arrEndTime = temp;

// arrRewardRate.push({ timestamp: stake1.start, rate: reward_per_second / total_nft });
// arrEndTime.push(stake1.end);

// // stake nft 2
// let stake2 = {
//     start: 15,
//     end: 25,
// };

// let isHad = false;
// // 20
// arrEndTime.forEach((time) => {
//     if (time <= stake2.start) {
//         total_nft -= 1;
//         arrRewardRate.push({ timestamp: time, rate: total_nft == 0 ? 0 : reward_per_second / total_nft });
//     }
//     if (time == stake2.start) {
//         isHad = true;
//     }
// });

// total_nft += 1;
// arrEndTime = arrEndTime.filter((item) => item > stake2.start);
// arrEndTime.push(stake2.end);
// if (!isHad) arrRewardRate.push({ timestamp: stake2.start, rate: reward_per_second / total_nft });

// // stake nft 3
// let stake3 = {
//     start: 20,
//     end: 30,
// };

// isHad = false;
// // arrRewardRate =[10: 10, 15: 5]
// // arrEndTime = [20, 25]
// // total_nft = 2

// arrEndTime.forEach((time) => {
//     // 20
//     if (time <= stake3.start) {
//         arrRewardRate.push({ timestamp: time, rate: total_nft == 0 ? 0 : reward_per_second / total_nft });
//     }
//     if (time == stake3.start) {
//         isHad = true;
//     }
// });

// arrEndTime = arrEndTime.filter((item) => item > stake3.start);
// arrEndTime.push(stake3.end);
// if (!isHad) {
//     total_nft += 1;
//     arrRewardRate.push({ timestamp: stake3.start, rate: reward_per_second / total_nft });
// }

// // stake nft 4
// let stake4 = {
//     start: 25,
//     end: 35,
// };

// isHad = false;
// arrEndTime.forEach((time) => {
//     if (time <= stake4.start) {
//         arrRewardRate.push({ timestamp: time, rate: total_nft == 0 ? 0 : reward_per_second / total_nft });
//     }
//     if (time == stake4.start) {
//         isHad = true;
//     }
// });

// arrEndTime = arrEndTime.filter((item) => item > stake4.start);
// arrEndTime.push(stake4.end);
// if (!isHad) {
//     total_nft += 1;
//     arrRewardRate.push({ timestamp: stake4.start, rate: reward_per_second / total_nft });
// }

// // stake nft 5
// let stake5 = {
//     start: 40,
//     end: 50,
// };

// isHad = false;
// arrEndTime.forEach((time) => {
//     if (time <= stake5.start) {
//         arrRewardRate.push({ timestamp: time, rate: total_nft == 0 ? 0 : reward_per_second / total_nft });
//     }
//     if (time == stake5.start) {
//         isHad = true;
//     }
// });

// arrEndTime = arrEndTime.filter((item) => item > stake5.start);
// arrEndTime.push(stake5.end);
// if (!isHad) {
//     total_nft += 1;
//     arrRewardRate.push({ timestamp: stake5.start, rate: reward_per_second / total_nft });
// }

// // stake nft 6
// let stake6 = {
//     start: 42,
//     end: 52,
// };

// isHad = false;
// arrEndTime.forEach((time) => {
//     if (time <= stake6.start) {
//         arrRewardRate.push({ timestamp: time, rate: total_nft == 0 ? 0 : reward_per_second / total_nft });
//     }
//     if (time == stake6.start) {
//         isHad = true;
//     }
// });

// arrEndTime = arrEndTime.filter((item) => item > stake6.start);
// arrEndTime.push(stake6.end);
// if (!isHad) {
//     total_nft += 1;
//     arrRewardRate.push({ timestamp: stake6.start, rate: reward_per_second / total_nft });
// }

// // stake nft 7
// let stake7 = {
//     start: 55,
//     end: 65,
// };

// isHad = false;
// arrEndTime.forEach((time) => {
//     if (time <= stake7.start) {
//         arrRewardRate.push({ timestamp: time, rate: total_nft == 0 ? 0 : reward_per_second / total_nft });
//     }
//     if (time == stake7.start) {
//         isHad = true;
//     }
// });

// arrEndTime = arrEndTime.filter((item) => item > stake7.start);
// arrEndTime.push(stake7.end);
// if (!isHad) {
//     total_nft += 1;
//     arrRewardRate.push({ timestamp: stake7.start, rate: reward_per_second / total_nft });
// }

// console.log(arrRewardRate);

// function calculateReward(timestamp, stake, arrRewardRate) {
//     let reward = 0;

//     // Sắp xếp arrRewardRate theo thời gian tăng dần
//     arrRewardRate.sort((a, b) => a.timestamp - b.timestamp);

//     // Tìm khoảng thời gian đặt cọc của NFT
//     let stakeStart = stake.start;
//     let stakeEnd = Math.min(stake.end, timestamp);

//     // Nếu mốc thời gian yêu cầu trước khi NFT được đặt cọc, trả về 0
//     if (timestamp < stakeStart) return 0;

//     let lastRateTimestamp = stakeStart;

//     // Duyệt qua mỗi mốc thời gian có sự thay đổi về tỷ lệ phần thưởng
//     for (let i = 0; i < arrRewardRate.length; i++) {
//         let rateObj = arrRewardRate[i];

//         if (rateObj.timestamp >= stakeStart && rateObj.timestamp <= stakeEnd) {
//             let nextTimestamp = i + 1 < arrRewardRate.length ? arrRewardRate[i + 1].timestamp : stakeEnd;
//             let duration = Math.min(nextTimestamp, stakeEnd) - rateObj.timestamp;
//             reward += duration * rateObj.rate;
//         }
//     }

//     return reward;
// }

// // Sử dụng function
// let rewardForNFT4 = calculateReward(500, stake1, arrRewardRate);
// console.log(rewardForNFT4);

// console.log({ arrEndTime });
// console.log({ arrRewardRate });
// console.log({ total_nft });
