require('dotenv').config();

const { ApiPromise, WsProvider } = require('@polkadot/api');

const typedefs = require('../../e2e/typedefs.json');

async function main () {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider, types: typedefs });

    const workerState = {};
    const entries = await api.query.phalaModule.workerState.entries();
    for (let [k, v] of entries) {
        console.log(`${k.args.map(k => k.toHuman())} =>`, v.toHuman());
        workerState[k.args[0].toHuman()] = v.toHuman();
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
        numWorkerWithScore:
            Object.entries(workerState).filter(([_, v]) => !!v.score).length,
        numStartingWorkers:
            Object.entries(workerState).filter(([_, v]) => 'MiningPending' in v.state).length,
        numStoppingWorkers:
            Object.entries(workerState).filter(([_, v]) => 'MiningStopping' in v.state).length,
        totalPower: Object.entries(workerState)
            .filter(([_, v]) => !!v.state.Mining)
            .map(([_, v]) => v.score && parseInt(v.score.overallScore) || 0)
            .reduce((a, x) => a + x, 0),
        maxCores: Object.entries(workerState)
            .map(([_, v]) => v.score && parseInt(v.score.features[0]) || 0)
            .reduce((a, x) => Math.max(a, x), 0),
    }
    console.log({statsFromWorkerState});
}

main().catch(console.error).finally(() => process.exit());

