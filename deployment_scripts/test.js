const { exec } = require("child_process");

let arr = [
    "Buzz39553z",
    "Buzz40126z",
    "Buzz40250z",
    "Buzz41792z",
    "Buzz50095z",
    "Buzz50545z",
    "Buzz50625z",
    "Buzz50636z",
    "Buzz50665z",
    "Buzz50667z",
    "Buzz50675z",
    "Buzz50715z",
    "Buzz50718z",
    "Buzz50758z",
    "Buzz7165z",
    "Buzz7261z",
    "Buzz732",
    "Buzz794",
    "Buzz80z",
    "Buzz85z",
    "Buzz878",
    "Buzz901z",
    "Buzz929",
    "Buzz947z",
];

let count = 60000;
setInterval(() => {
    console.log(count);

    exec(
        `beaker wasm execute cw721_metadata_onchain --raw '{"mint":{"token_id":"Buzz${count.toString()}z", "owner": "aura1y9ts00xzxywqx3dxcyyg2xsqd8edackqcr9glm", "token_uri":"ipfs://QmNjiRJ4UUMWjZ7cBQus7GgaNYLAbvN1vvjsqSrbXEwUVE/100.json",  "extension": {"image": null, "image_data": null,"external_url": null, "description": null,"name": null,"attributes": null,"background_color": null,"animation_url": null, "youtube_url": null}}}' --signer-account signer --network euphoria`,
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
}, 5000);

// let i = 0;
// setInterval(() => {
//     exec(
//         `beaker wasm execute campaign --raw '{"stake_nfts":{"stake_info":{"token_ids":["${arr[i]}"],"lockup_term":259200}}}' --signer-account signer --network euphoria
//             `,
//         (error, stdout, stderr) => {
//             // if (error) {
//             //     console.error(`exec error: ${error}`);
//             //     return;
//             // }
//             console.log(`stdout ${i}: ${stdout}`);
//             // console.error(`stderr: ${stderr}`);
//         }
//     );

//     i++;
// }, 5000);
