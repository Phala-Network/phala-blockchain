require('dotenv').config();

const { ApiPromise, WsProvider } = require('@polkadot/api');

const { poolSubAccount } = require('../src/utils/palletUtils');

const typedefs = require('@phala/typedefs').khalaDev;

async function printMiner(api, pid, worker, checks) {
    const miner = poolSubAccount(api, pid, worker);

    async function print(n) {
        const h = await api.rpc.chain.getBlockHash(n);
        const apiAt = await api.at(h);
        const m = await apiAt.query.phalaMining.miners(miner);
        const worker = await apiAt.query.phalaMining.minerBindings(miner);
        const stake = await apiAt.query.phalaMining.stakes(miner);
        const pool = await apiAt.query.phalaStakePool.stakePools(pid);
        console.log({
            n,
            miner: m.toHuman(),
            worker: worker.toHuman(),
            stake: stake.toHuman(),
            pool: pool.toHuman(),
        });
    }
    for (const [tag, n] of checks) {
        console.log(tag);
        await print(n);
    }
}

async function main() {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider, types: typedefs });

    await printMiner(
        api, 311, '0x042fd1d8359700b729adc8a58f4208baac2d4d41a33cfb73bc557ec661fa211a', [
            ['added', 505552],
            ['started', 505622],
            ['stopped-1', 513876-1],
            ['stopped', 513876],
            ['wiped-1', 513903-1],
            ['wiped', 513903],
        ]
    );

    await printMiner(
        api, 311, '0xa4ca6a09e40d692707b5e34e52850399c650e338143a7fae6bc38b28e0cc8609', [
            ['added', 431747],
            ['started', 431853],
            ['stopped-1', 505505-1],
            ['stopped', 505505],
            ['stopped+50', 505505+50],
        ]
    );
}

main().catch(console.error).finally(() => process.exit());
