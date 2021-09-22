const fs = require('fs');
const Papa = require('papaparse');

const { program } = require('commander');


program
    .option('--snapshots <path>', 'The snapshots to analyze', './tmp/snapshots.json')
    .option('--pool-workers <path>', 'When sepcified the dumped pool worker json file, the analysis will have pool level break-down', './tmp/pool-workers.json')
    .option('--output <path>', 'The path of the output csv file', './tmp/analysis.csv')
    .option('--sample-worker', 'If enabled, it will sample "v, totalReward, pPerc" of the first worker in a pool in the output', false)
    .action(main);

function sampleWorker(dataset, N=1) {
    const miners = dataset[0].frame.slice(0, N).map(m => m.miner);
    const sampled = dataset.map(({frame}) =>
        frame
            .filter(m => miners.includes(m.miner))
            .sort((a, b) => a.miner.localeCompare(b.miner))
    );

    // series: v, totalReward, pInstant/pInit
    const series = {};
    for (let i = 0; i < N; i++) {
        series[`v-${i}`] = sampled.map(m => m[i] ? m[i].v : 0);
        series[`totalReward-${i}`] = sampled.map(m => m[i] ? m[i].totalReward : 0);
        series[`pPerc-${i}`] = sampled.map(m => m[i] ? m[i].pInstant / m[i].pInit : 0);
    }
    return series;
}

function stats(dataset) {
    // series: sum(totalReward)
    const totalRewards = dataset
        .map(({frame}) => frame.reduce((acc, {totalReward}) => acc + totalReward, 0));

    // series: sum(state == Unresponsive)
    const unresponsive = dataset
        .map(({frame}) => frame.filter(m => m.state == 'MiningUnresponsive').length);

    // series: typical V and totalReward
    const sampledMetrics = program.opts().sampleWorker ? sampleWorker(dataset) : {};

    return {
        totalRewards,
        unresponsive,
        ...sampledMetrics,
    };
}

function extractPoolWorkers(dataset, workerSet) {
    return dataset.map(({blocknum, frame}) => ({
        blocknum,
        frame: frame.filter(m => workerSet.has(m.worker)),
    }));
}

function loadJson(path) {
    const rawJson = fs.readFileSync(path);
    return JSON.parse(rawJson);
}

function addToSheet(sheet, columns, prefix) {
    for (const k in columns) {
        sheet[`${prefix}-${k}`] = columns[k];
    }
}

function sheetToCsv(sheet) {
    // Assume same rows
    const n = Object.values(sheet)[0].length;

    const fields = Object.keys(sheet);
    const data = [];
    for (let i = 0; i < n; i++) {
        const row = [];
        for (const k in sheet) {
            row.push(sheet[k][i]);
        }
        data.push(row);
    }
    return { fields, data };
}

function main() {
    const { snapshots, poolWorkers, output } = program.opts();

    const dataset = loadJson(snapshots);
    const poolWorkersData = poolWorkers ? loadJson(poolWorkers) : {};

    const sheet = {
        blocknum: dataset.map(row => row.blocknum),
    };

    addToSheet(sheet, stats(dataset), 'full');
    for (const pid in poolWorkersData) {
        const workers = poolWorkersData[pid];
        const slice = extractPoolWorkers(dataset, new Set(workers));
        addToSheet(sheet, stats(slice), `p${pid}`);
    }

    const csvObj = sheetToCsv(sheet);
    const rawCsv = Papa.unparse(csvObj);
    fs.writeFileSync(output, rawCsv, {encoding: 'utf-8'});
}

program.parse(process.argv);
