let arrRewardRate = [];
let arrEndTime = [];
let reward_per_second = 10;
let total_nft = 0;

function updateRewardRate(timestamp, change) {
    total_nft += change;
    const rate = total_nft ? reward_per_second / total_nft : 0;

    const existingRate = arrRewardRate.find((item) => item.timestamp === timestamp);
    if (existingRate) {
        existingRate.rate = rate;
    } else {
        arrRewardRate.push({ timestamp, rate });
    }
}

function stakeNFT(nft) {
    // Xác định các NFT đã hết hạn trước thời điểm stake NFT hiện tại
    arrEndTime = arrEndTime.filter((endTime) => {
        if (endTime <= nft.start) {
            updateRewardRate(endTime, -1);
            return false;
        }
        return true;
    });

    updateRewardRate(nft.start, 1);
    arrEndTime.push(nft.start + nft.term);
}

const stakes = [
    { start: 10, term: 10 },
    { start: 15, term: 10 },
    { start: 20, term: 10 },
    { start: 25, term: 10 },
    { start: 40, term: 10 },
    { start: 42, term: 10 },
    { start: 55, term: 10 },
    { start: 55, term: 10 },
    { start: 55, term: 10 },
    { start: 56, term: 10 },
    { start: 70, term: 10 },
    { start: 71, term: 10 },
    { start: 90, term: 10 },
];

stakes.forEach((stake) => stakeNFT(stake));
console.log(arrRewardRate);

function calculateReward(timestamp, stake, arrRewardRate) {
    arrEndTime = arrEndTime.filter((endTime) => {
        if (endTime <= timestamp) {
            updateRewardRate(endTime, -1);
            return false;
        }
        return true;
    });

    updateRewardRate(timestamp, 0);

    let reward = 0;

    // Sắp xếp arrRewardRate theo thời gian tăng dần
    arrRewardRate.sort((a, b) => a.timestamp - b.timestamp);

    // Tìm khoảng thời gian đặt cọc của NFT
    let stakeStart = stake.start;
    let stakeEnd = Math.min(stake.end, timestamp);

    // Nếu mốc thời gian yêu cầu trước khi NFT được đặt cọc, trả về 0
    if (timestamp < stakeStart) return 0;

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

console.log(calculateReward(500, { start: 71, end: 81 }, arrRewardRate));
console.log(arrRewardRate);
let rates = [
    { timestamp: 10, rate: 10 },
    { timestamp: 15, rate: 5 },
    { timestamp: 20, rate: 5 },
    { timestamp: 25, rate: 5 },
    { timestamp: 30, rate: 10 },
    { timestamp: 35, rate: 0 },
    { timestamp: 40, rate: 10 },
    { timestamp: 42, rate: 5 },
    { timestamp: 52, rate: 0 },
    { timestamp: 55, rate: 10 },
];
