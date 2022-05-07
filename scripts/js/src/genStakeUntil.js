require('dotenv').config();

const { ApiPromise, WsProvider } = require('@polkadot/api');
const BN = require('bn.js');

const bn1e12 = new BN(10).pow(new BN(12));

const target = [
    [484, 1000000],
    [485, 1000000],
    [486, 1000000],
    [487, 1000000],
];


async function main() {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider });

    const pools = (await Promise.all(
        target.map(([pid]) => api.query.phalaStakePool.stakePools(pid))
    )).map(x => x.unwrap());
    console.log(pools.map(x => x.totalStake.toString()))

    const tx = api.tx.utility.batchAll(
        target.map(([pid, amount], idx) =>
            api.tx.phalaStakePool.contribute(
                pid,
                new BN(amount).mul(bn1e12).sub(pools[idx].totalStake)
            )
        ),
    );

    console.log(tx.toHex());
}

main().catch(console.error).finally(() => process.exit());
