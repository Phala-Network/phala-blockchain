require('dotenv').config();

const fs = require('fs');

const { ApiPromise, WsProvider } = require('@polkadot/api');
const BN = require('bn.js');

const { poolSubAccount, calculateReturnSlash } = require('../src/utils/palletUtils');
const { FixedPointConverter } = require('../src/utils/fixedUtils');
const fp = new FixedPointConverter();

const typedefs = require('@phala/typedefs').khalaDev;
const Decimal = require('decimal.js').default;

const bn1e12 = new BN(10).pow(new BN(12));
const dec1 = new Decimal(1);
const dec1e12 = new Decimal(bn1e12.toString());

const PoolWorkerAdded = 'phalaStakePool.PoolWorkerAdded';   // (pid, worker)
const MinerStarted = 'phalaMining.MinerStarted';    // (miner)
const MinerStopped = 'phalaMining.MinerStopped';    // (miner)
const MinerReclaimed = 'phalaMining.MinerReclaimed';    // (miner, returned, slashed)

async function getApiAt(api, at) {
    const h = await api.rpc.chain.getBlockHash(at);
    const apiAt = await api.at(h);
    return apiAt;
}

async function minerStateAt(api, miner) {
    const r = await api.query.phalaMining.miners(miner);
    return r.unwrap();
}

function findAffectedWorkers(minerEvents) {
    // State machine:
    //                  |--------------[Relcaim]-------------|
    //                  v                                    |
    //  None -[Add]-> Ready -[Start]-> Mining -[Stop]-> CoolingDown
    //               /   ^                                   |
    //      [re-Add] \---|----------[Add (b500!)]------------|

    const b500Error = [];
    for (const [miner, info] of Object.entries(minerEvents)) {
        let state = 'None';
        let addedAt, stoppedAt;
        for (const ev of info.events) {
            const wrongState = () => {
                console.warn('wrong state', {
                    blockNumber: ev.blockNumber,
                    miner,
                    from: state,
                    event: ev.event,
                });
            }
            if (state == 'None') {
                if (ev.event == PoolWorkerAdded) {
                    state = 'Ready';
                    addedAt = ev.blockNumber;
                    stoppedAt = undefined;
                } else {
                    wrongState();
                }
            } else if (state == 'Ready') {
                if (ev.event == MinerStarted) {
                    state = 'Mining';
                } else if (ev.event == PoolWorkerAdded) {
                    // no-op: user just removed & added back
                    addedAt = ev.blockNumber;
                    stoppedAt = undefined;
                } else {
                    wrongState();
                }
            } else if (state == 'Mining') {
                if (ev.event == MinerStopped) {
                    state = 'CoolingDown';
                    stoppedAt = ev.blockNumber;
                } else {
                    wrongState();
                }
            } else if (state == 'CoolingDown') {
                if (ev.event == MinerReclaimed) {
                    state = 'Ready';
                } else if (ev.event == PoolWorkerAdded) {
                    // May trigger the bug!
                    b500Error.push({
                        blocknum: ev.blockNumber,
                        pid: info.pid,
                        worker: info.worker,
                        miner: miner,
                        addedAt,
                        stoppedAt,
                    });
                    console.log('b500 candidate', miner);
                    state = 'Ready';
                    addedAt =  ev.blockNumber;
                    stoppedAt = undefined;
                } else {
                    wrongState();
                }
            } else {
                wrongState();
            }
        }
    }
    return b500Error;
}

async function main() {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider, types: typedefs });

    const filein = fs.readFileSync('./tmp/issue500minerEvents.json', {encoding: 'utf-8'});
    const minerEvents = JSON.parse(filein);
    Object.entries(minerEvents).forEach(([_k, info]) => {
        info.pid = parseInt(info.pid);
    });

    // run the state machine to simulate the mining process and find out the workers affected by
    // b500
    const potentialMissingStake = findAffectedWorkers(minerEvents);

    // reconstruct real missing stake
    const missingStake = [];
    let skipped = 0;
    for (const candidate of potentialMissingStake) {
        const {stoppedAt, blocknum, pid, worker, miner} = candidate;  // blocknum

        const apiAtBefore = await getApiAt(api, stoppedAt);
        const apiAtAfter = await getApiAt(api, blocknum);

        const before = await minerStateAt(apiAtBefore , miner);
        const after = await minerStateAt(apiAtAfter, miner);

        if (before.state == 'Ready' && after.state == 'Ready') {
            skipped++;
            continue;
        } else if (before.state == 'MiningCoolingDown' && after.state == 'Ready') {
            const stake = await apiAtBefore.query.phalaMining.stakes(miner);

            const [returned, slashed] = calculateReturnSlash(before, stake.unwrap());
            const item = {
                blocknum, stoppedAt, pid, worker, miner,
                origStake: stake.toString(),
                returned: returned.toString(),
                slashed: slashed.toString(),
            };
            missingStake.push(item);
            console.log(`[${blocknum}] Missing stake pid=${pid} worker=${worker} returning=${returned} slashing=${slashed}`);
        } else {
            console.error('Unknown case', blocknum, pid, worker);
        }
        // console.log('------------------');
        // console.log(before.toJSON());
        // console.log(after.toJSON());
    }
    console.log(`${missingStake.length} found, ${skipped} entries skipped`);

    // apply #541 special adjustment
    const b541Hack = {
        '43E9fDbtnPZRudbFHxDoGiid8gxoMAodJJuBaHi9qLUb9zdn': '23592649396643'
    };
    let numHackApplied = 0;
    missingStake.forEach(item => {
        if (item.miner in b541Hack) {
            const slashed = new BN(item.slashed);
            const returned = new BN(item.returned);
            const toSlash = new BN(b541Hack[item.miner]);
            item.slashed = slashed.add(toSlash).toString();
            item.returned = returned.sub(toSlash).toString();
            console.log('b541 hack', {
                before: returned.toString(),
                after: item.returned.toString(),
            })
            numHackApplied++;
        }
    });
    console.log(`b541 hack applied: ${numHackApplied}`);

    const latestBlock = 570248;
    const preimage = Object.entries(minerEvents).reduce((map, [miner, {worker, pid}]) => {
        map[miner] = {worker, pid};
        return map;
    }, {});
    const reconciling = {
        latestBlock,
        preimage,
        missingStake,
        b541Hack,
    }
    fs.writeFileSync(
        './tmp/issue500Reconciling.json',
        JSON.stringify(reconciling, undefined, 2),
        {'encoding': 'utf-8'}
    );
}

main().catch(console.error).finally(() => process.exit());
