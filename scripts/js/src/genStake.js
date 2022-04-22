require('dotenv').config();

const { ApiPromise, WsProvider } = require('@polkadot/api');
const BN = require('bn.js');

const bn1e12 = new BN(10).pow(new BN(12));

/*
[488, 0.27]
[495, 0.5]
[517, 0.1]
[491, 0.1]
[494, 0.075]
[514, 0.0001]
[500, 0.21]
[502, 0.0001]
 */

const target = [
    [487, 9990000],
    [484, 9990000],
];


async function main() {
    const beneficiary = '44HxFTkfcEyfAfkyxu73iftfjDZDz4zmyiodNvbonFUdkLhv';
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider });


    const pools = (await Promise.all(
        target.map(([pid]) => api.query.phalaStakePool.stakePools(pid))
    )).map(x => x.unwrap());

    const cappedTarget = target.map(([pid, amount], idx) => {
        const p = pools[idx];
        const a = new BN(amount).mul(bn1e12);
        const cappedAmount = p.cap.isSome ? BN.min(p.cap.unwrap().sub(p.totalStake), a) : a;
        return [pid, cappedAmount];
    });

    const tx = api.tx.utility.batchAll(
        cappedTarget.flatMap(([pid, amount]) => [
            // api.tx.phalaStakePool.withdraw(pid, new BN(amount).mul(bn1e12)),
            // api.tx.phalaStakePool.claimRewards(pid, beneficiary),
            api.tx.phalaStakePool.contribute(pid, amount)
        ]),
    );

    console.log(tx.toHex());
}

main().catch(console.error).finally(() => process.exit());
