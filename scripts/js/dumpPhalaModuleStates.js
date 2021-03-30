require('dotenv').config();

const { ApiPromise, WsProvider } = require('@polkadot/api');
const { decodeAddress } = require('@polkadot/keyring');
const { u8aToHex } = require('@polkadot/util');


const typedefs = require('@phala/typedefs/dist/phala-typedef').default;

async function getStatsAt(api, hash, logEntries=true, logStats=true) {
    const workerState = {};
    const entries = await api.query.phalaModule.workerState.entriesAt(hash);
    for (let [k, v] of entries) {
        if (logEntries) {
            console.log(`${k.args.map(k => k.toHuman())} =>`, v.toJSON());
        }
        workerState[k.args[0].toHuman()] = v.toJSON();
    }

    const [onlineWorkers, totalPower, delta] = await Promise.all([
        api.query.phalaModule.onlineWorkers.at(hash),
        api.query.phalaModule.totalPower.at(hash),
        api.query.phalaModule.pendingExitingDelta.at(hash),
    ]);
    const onchainData = {
        onlineWorkers: onlineWorkers.toNumber() + delta.numWorker.toNumber(),
        totalPower: totalPower.toNumber() + delta.numPower.toNumber(),
        delta: delta.toJSON(),
    };

    const statsFromWorkerState = {
        numWorkers: Object.keys(workerState).length,
        numOnlineWorkers: Object.entries(workerState)
            .filter(([_, v]) => !!v.state.Mining || 'MiningStopping' in v.state).length,
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
        totalCores: Object.entries(workerState)
            .map(([_, v]) => v.score && parseInt(v.score.features[0]) || 0)
            .reduce((a, x) => a + x, 0),
    }
    if (logStats) {
        console.log({onchainData, statsFromWorkerState});
    }

    return {
        workerState,
        onchainData,
        statsFromWorkerState,
    };
}

async function binarySearch(asyncPred, lo, hi) {
    let l = lo;
    let r = hi;
    while (l <= r) {
        const mid = ((l + r) / 2) | 0;
        const pred = await asyncPred(mid);
        if (pred == 'stop') {
            break;
        } else if (pred == 'left') {
            l = mid + 1;
        } else {
            r = mid - 1;
        }
    }
    return [l, r];
}

async function checkDiff (api, startHeight=378900) {
    const latestHeader = await api.rpc.chain.getHeader();
    const endHeight = latestHeader.number.toNumber();
    console.log(`Search diff between ${startHeight} and ${endHeight}`);

    for (let targetDiff = 0; targetDiff <= 5; targetDiff++) {
        const [l, r] = await binarySearch(async (height) => {
            const hash = await api.rpc.chain.getBlockHash(height);
            const { onchainData, statsFromWorkerState } = await getStatsAt(api, hash, false, false);
            const diff = onchainData.onlineWorkers - statsFromWorkerState.numOnlineWorkers;
            const result = diff <= targetDiff ? 'left' : 'right';
            console.log(`  . block [${height}]`, {
                onchain: onchainData.onlineWorkers,
                stats: statsFromWorkerState.numOnlineWorkers,
                diff,
                result,
            });
            return result;
        }, startHeight, endHeight);

        console.log(`Target online - stats == ${targetDiff}: [${l}, ${r}]`);
        if (l > endHeight) {
            break;
        }
    }
}

async function main () {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider, types: typedefs });
    const runBinarySearch = (process.env.BINARY_SEARCH === '1');
    const shouldDumpFullWorkers = (process.env.FULL_DUMP === '1');

    if (runBinarySearch) {
        await checkDiff(api);
        return;
    }

    const last = await api.rpc.chain.getBlockHash();
    const { workerState } = await getStatsAt(api, last);

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

