const fs = require('fs');

const { ApiPromise, WsProvider } = require('@polkadot/api');
const { poolSubAccount } = require('../src/utils/palletUtils');

const typedefs = require('@phala/typedefs').khalaDev;

const PoolWorkerAdded = 'phalaStakePool.PoolWorkerAdded';   // (pid, worker)
const MinerStarted = 'phalaMining.MinerStarted';    // (miner)
const MinerStopped = 'phalaMining.MinerStopped';    // (miner)
const MinerReclaimed = 'phalaMining.MinerReclaimed';    // (miner, returned, slashed)

async function main() {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider, types: typedefs });

    const filein = fs.readFileSync('./tmp/issue500events.json', {encoding: 'utf-8'});
    const events = filein.split('\n').filter(x => !!x).map(JSON.parse).reverse();

    const preimage = {};
    const minerEvents = {};
    for (const ev of events) {
        if (ev.event == PoolWorkerAdded) {
            const [pid, worker] = ev.data;
            const miner = poolSubAccount(api, pid, worker).toString();
            preimage[miner] = {pid, worker};
            if (!minerEvents[miner]) {
                minerEvents[miner] = {
                    pid, worker,
                    events: [],
                };
            }
            minerEvents[miner].events.push({
                blockNumber: ev.blockNumber,
                event: ev.event,
            });
        } else if (ev.event == MinerStarted || ev.event == MinerStopped || ev.event == MinerReclaimed) {
            const miner = ev.data[0];
            minerEvents[miner].events.push({
                blockNumber: ev.blockNumber,
                event: ev.event,
            })
        }
    }

    const outJson = JSON.stringify(minerEvents, undefined, 2);
    fs.writeFileSync('./tmp/issue500minerEvents.json', outJson, {encoding: 'utf-8'});
}

main().catch(console.error).finally(() => process.exit());
