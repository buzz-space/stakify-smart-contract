const { exec } = require("child_process");

let arr = [
    "Buzz6811z",
    "Buzz6973z",
    "Buzz7058z",
    "Buzz7063z",
    "Buzz7075z",
    "Buzz709",
    "Buzz7097z",
    "Buzz7115z",
    "Buzz711z",
    "Buzz7126z",
    "Buzz7137z",
    "Buzz7143z",
    "Buzz715",
    "Buzz7154z",
    "Buzz7165z",
    "Buzz7176z",
    "Buzz7182z",
    "Buzz7188z",
    "Buzz7204z",
    "Buzz721",
    "Buzz7210z",
    "Buzz7216z",
    "Buzz7221z",
    "Buzz7227z",
    "Buzz722z",
    "Buzz7232z",
    "Buzz7238z",
    "Buzz7244z",
    "Buzz7250z",
    "Buzz7256z",
    "Buzz726",
    "Buzz7261z",
    "Buzz7267z",
    "Buzz7278z",
    "Buzz7284z",
    "Buzz7289z",
    "Buzz728z",
    "Buzz7295z",
    "Buzz7300z",
    "Buzz732",
    "Buzz735z",
    "Buzz737",
    "Buzz743",
    "Buzz745z",
    "Buzz74z",
    "Buzz750z",
    "Buzz754",
    "Buzz756z",
    "Buzz76",
    "Buzz761",
    "Buzz761z",
    "Buzz766",
    "Buzz767z",
    "Buzz772",
    "Buzz772z",
    "Buzz777",
    "Buzz778z",
    "Buzz783",
    "Buzz784z",
    "Buzz788",
    "Buzz789z",
    "Buzz794",
    "Buzz795z",
    "Buzz799",
    "Buzz8",
    "Buzz800z",
    "Buzz805",
    "Buzz80z",
    "Buzz81",
    "Buzz811",
    "Buzz812z",
    "Buzz816",
    "Buzz817z",
    "Buzz822",
    "Buzz823z",
    "Buzz827",
    "Buzz829z",
    "Buzz833",
    "Buzz835z",
    "Buzz839",
    "Buzz840z",
    "Buzz846z",
    "Buzz850",
    "Buzz851z",
    "Buzz856",
    "Buzz857z",
    "Buzz85z",
    "Buzz862",
    "Buzz863z",
    "Buzz867",
];
let count = 50500;
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
//         `beaker wasm execute campaign --raw '{"stake_nfts":{"nfts":[{"token_id":"${arr[i]}","lockup_term":86400}]}}' --signer-account signer --network euphoria
//             `,
//         (error, stdout, stderr) => {
//             // if (error) {
//             //     console.error(`exec error: ${error}`);
//             //     return;
//             // }
//             console.log(`stdout: ${stdout}`);
//             // console.error(`stderr: ${stderr}`);
//         }
//     );
//     i++;
// }, 2000);
