require('dotenv').config();

const { ApiPromise, WsProvider } = require('@polkadot/api');
const BN = require('bn.js');

const bn1e12 = new BN(10).pow(new BN(12));

const pools = [
    [480, 135000-1],
    [515, 135000-1],
    // [504, 131749],
    // [501, 120115],
    // [489, 104060],
    // [508, 99655],
    // [482, 99571],
    // [518, 70540],
    // [514, 64066],
    // [516, 46479],
    // [507, 45686],
    // [511, 37829],
    // [499, 36619],
    // [517, 35968],
    // [500, 34818],
    // [510, 28622],
    // [495, 27328],
    // [492, 24726],
    // [498, 24726],
    // [509, 22548],
    // [490, 19375],
    // [497, 16133],
    // [526, 13970],
    // [496, 12240],
    // [471, 10000],
    // [502, 9321],
    // [485, 8655],
    // [1890, 7927],
    // [491, 6942],
    // [494, 4927],
    // [481, 4389],
    // [512, 3473],
    // [493, 3355],
    // [506, 3255],
    // [483, 2906],
    // [505, 2271],
    // [488, 1274],
    // [503, 1203],
    // [513,500],
]

async function main() {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider });

    const tx = api.tx.utility.batchAll(
        pools.map(([pid, shares]) =>
            api.tx.phalaStakePool.withdraw(pid, new BN(shares).mul(bn1e12)),
        ),
    );

    console.log(tx.toHex());
}

main().catch(console.error).finally(() => process.exit());
