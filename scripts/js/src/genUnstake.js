require('dotenv').config();

const { ApiPromise, WsProvider } = require('@polkadot/api');
const BN = require('bn.js');

const bn1e12 = new BN(10).pow(new BN(12));

const pools = [
    // 426, 427, 428, 429, 430, 431, 432, 434, 435, 436, 437, 438, 439, 440, 441, 442, 443, 444, 445, 446, 447, 448, 449, 450, 451, 452, 453, 454, 455, 456, 457, 458, 459, 460, 461, 462, 463, 467, 468, 469, 470, 471, 472, 473, 474, 475, 476,
    // 413, 414, 415, 416, 417, 418, 419, 420, 421, 422, 423, 424, 425,
]
for (let i = 519; i <= 532; i++)
    pools.push(i);

async function main() {
    const beneficiary = '5He5biKTYbb5MkMTtm8NqBs7VgTJ1pSYZFmL8LUbYAuYqmej';
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider });

    const user = await Promise.all(
        pools.map(async pid =>
            [pid, await api.query.phalaStakePool.poolStakers([pid, beneficiary])]
        )
    );
    const poolShares = user.map(([pid, u]) => [pid, u.unwrap().shares]);

    const tx = api.tx.utility.batchAll(
        poolShares.map(([pid, shares]) =>
            api.tx.phalaStakePool.withdraw(pid, shares),
        ),
    );

    console.log(tx.toHex());
}

main().catch(console.error).finally(() => process.exit());
