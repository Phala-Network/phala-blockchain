require('dotenv').config();

const { ApiPromise, WsProvider } = require('@polkadot/api');
const { decodeAddress } = require('@polkadot/keyring');
const { u8aToHex } = require('@polkadot/util');


const typedefs = require('../../e2e/typedefs.json');

async function main () {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider, types: typedefs });
    const shouldDumpFullWorkers = (process.env.FULL_DUMP === '1');

    const workerState = {};
    const entries = await api.query.phalaModule.workerState.entries();
    for (let [k, v] of entries) {
        console.log(`${k.args.map(k => k.toHuman())} =>`, v.toJSON());
        workerState[k.args[0].toHuman()] = v.toJSON();
    }

    const [onlineWorkers, totalPower] = await Promise.all([
        api.query.phalaModule.onlineWorkers(),
        api.query.phalaModule.totalPower(),
    ]);

    console.log({onchainData: {
        onlineWorkers: onlineWorkers.toNumber(),
        totalPower: totalPower.toNumber(),
    }});

    const statsFromWorkerState = {
        numWorkers: Object.keys(workerState).length,
        numOnlineWorkers: Object.entries(workerState).filter(([_, v]) => !!v.state.Mining).length,
        numStartingWorkers:
            Object.entries(workerState).filter(([_, v]) => 'MiningPending' in v.state).length,
        numStoppingWorkers:
            Object.entries(workerState).filter(([_, v]) => 'MiningStopping' in v.state).length,
        numWorkerWithScore:
            Object.entries(workerState).filter(([_, v]) => !!v.score).length,
        numWorkerWithMachineId:
            Object.entries(workerState).filter(([_, v]) => !!v.machineId).length,
        numWorkerWithoutMachineIdAndMining:
            Object.entries(workerState).filter(([_, v]) => !v.machineId && !!v.state.Mining).length,
        totalPower: Object.entries(workerState)
            .filter(([_, v]) => (!!v.state.Mining || 'MiningStopping' in v.state) && !!v.machineId)
            .map(([_, v]) => v.score && parseInt(v.score.overallScore) || 0)
            .reduce((a, x) => a + x, 0),
        maxCores: Object.entries(workerState)
            .map(([_, v]) => v.score && parseInt(v.score.features[0]) || 0)
            .reduce((a, x) => Math.max(a, x), 0),
    }
    console.log({statsFromWorkerState});

    if (shouldDumpFullWorkers) {
        const fs = require('fs');
        let tsv = Object.entries(workerState)
            .map(([k, v]) => [
                k, u8aToHex(decodeAddress(k)),
                JSON.stringify(v.state), v.score.overallScore,
                JSON.stringify(v.score.features)].join('\t'))
            .join('\n');
        tsv = ['ss58', 'hex', 'state', 'score', 'feature'].join('\t') + '\n' + tsv;
        fs.writeFileSync('./tmp/full_workers.tsv', tsv, {encoding: 'utf-8'});
    }
}

main().catch(console.error).finally(() => process.exit());

