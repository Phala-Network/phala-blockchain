require('dotenv').config();

const { ApiPromise, WsProvider } = require('@polkadot/api');
const BN = require('bn.js');
const fs = require('fs');

const typedefs = require('@phala/typedefs').phalaDev;

const bn64b = new BN(2).pow(new BN(64));
const bn1e12 = new BN(10).pow(new BN(12));
const bn1e10 = new BN(10).pow(new BN(10));

function writeJson(path, obj) {
    const jsonData = JSON.stringify(obj, undefined, 2);
    fs.writeFileSync(path, jsonData, {encoding: 'utf-8'});
}

async function main() {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({
        provider: wsProvider,
        types: typedefs,
    });

    const tip = await api.rpc.chain.getHeader();
    const tipNum = tip.number.toNumber();
    const GAP = 50; // 10 mins
    const startNum = tipNum - GAP * 144;  // 1 day

    // Dump pool workers
    const pids = [0];
    const pools = await api.query.phalaStakePool.stakePools.multi(pids);
    const poolWorkers = pools
        .map(p => p.unwrap())
        .map(p => [p.pid.toNumber(), p.workers.toJSON()])
        .reduce((map, [k, v]) => {
            map[k] = v;
            return map;
        }, {});
    writeJson('./tmp/poolWorkers.json', poolWorkers);

    // Dump miner-to-worker map (instant snapshot)
    const minerBindings = await api.query.phalaMining.minerBindings.entries();
    const minerWorkerMap = minerBindings
        .map(([key, v]) => [key.args[0].toHuman(), v.unwrap().toJSON()])
        .reduce((map, [key, v]) => {
            map[key] = v;
            return map;
        }, {});

    // Dump miner status
    const dataset = [];
    for (let n = tipNum; n > startNum; n -= GAP) {
        console.log('Remaining to dump:', (n - startNum) / GAP);
        const h = await api.rpc.chain.getBlockHash(n);
        const entries = await api.query.phalaMining.miners.entriesAt(h);

        const frame = entries.map(([key, v]) => {
            const m = v.unwrap();
            const miner = key.args[0].toHuman();
            return {
                miner,
                worker: minerWorkerMap[miner] || '',
                state: m.state.toString(),
                v: m.v.div(bn64b).toNumber(),
                pInit: m.benchmark.pInit.toNumber(),
                pInstant: m.benchmark.pInstant.toNumber(),
                updatedAt: m.benchmark.challengeTimeLast.toNumber(),
                totalReward: m.stats.totalReward.div(bn1e10).toNumber() / 100,
            }
        });
        dataset.push({
            blocknum: n,
            frame,
        });
    }

    writeJson('./tmp/snapshot.json', dataset);
}

main().catch(console.error).finally(() => process.exit());
