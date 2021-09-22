require('dotenv').config();

const { ApiPromise, WsProvider } = require('@polkadot/api');
const BN = require('bn.js');
const fs = require('fs');

const typedefs = require('@phala/typedefs').phalaDev;

const { program } = require('commander');

program
    .option('--endpoint <url>', 'Substrate WS endpoint', process.env.ENDPOINT || 'wss://khala-api.phala.network/ws')

program
    .command('dump-pool-workers')
    .option('--pools <pid-list>', 'Dump a list of the pool id, separated by comma', '')
    .option('--output <path>', 'The path of the output json file', './tmp/pool-workers.json')
    .action(run(dumpPoolWorkers));

program
    .command('dump-snapshots')
    .option('--step <n>', 'Dump the snapshot every n blocks', 100)
    .option('--since <b>', 'Dump the snapshots since block b; a negative number to specify a relative block number to the best one', -7200)
    .option('--output <path>', 'The path of the output json file', './tmp/snapshot.json')
    .action(run(dumpSnapshots));

const bn64b = new BN(2).pow(new BN(64));
const bn1e10 = new BN(10).pow(new BN(10));

function run(afn) {
    function runner(...args) {
        afn(...args)
            .catch(console.error)
            .then(process.exit)
            .finally(() => process.exit(-1));
    };
    return runner;
}

async function createApi() {
    const {endpoint} = program.opts();
    const wsProvider = new WsProvider(endpoint);
    return await ApiPromise.create({
        provider: wsProvider,
        types: typedefs,
    });
}

function writeJson(path, obj) {
    const jsonData = JSON.stringify(obj, undefined, 2);
    fs.writeFileSync(path, jsonData, {encoding: 'utf-8'});
}

async function dumpPoolWorkers (opt) {
    const {output, pools} = opt;
    // Check file access
    fs.writeFileSync(output, '');

    const api = await createApi();

    // Dump pool workers
    const pids = pools
        .split(',')
        .map(s => parseInt(s.trim()))
        .filter(x => !!x);
    if (pids.length == 0) {
        console.log('No pool specified');
        return;
    }

    console.log('Dumping pool workers at:', pids);
    const poolInfo = await api.query.phalaStakePool.stakePools.multi(pids);
    const poolWorkers = poolInfo
        .map(p => p.unwrap())
        .map(p => [p.pid.toNumber(), p.workers.toJSON()])
        .reduce((map, [k, v]) => {
            map[k] = v;
            return map;
        }, {});
    writeJson(output, poolWorkers);
}

async function dumpSnapshots(opt) {
    let {output, step, since} = opt;
    step = parseInt(step);
    since = parseInt(since);

    // Check file access
    fs.writeFileSync(output, '');

    const api = await createApi();

    const tip = await api.rpc.chain.getHeader();
    const tipNum = tip.number.toNumber();
    const startNum = since > 0 ? since : tipNum + since;

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
    for (let n = startNum; n <= tipNum; n += step) {
        const percentage = ((n - startNum) / (tipNum - startNum) * 100).toFixed(2);
        console.log(`Dumping ${n} / ${tipNum} (${percentage}%)`);
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

    writeJson(output, dataset);
}

program.parse(process.argv);
