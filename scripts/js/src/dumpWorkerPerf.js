require('dotenv').config();

const { ApiPromise, WsProvider } = require('@polkadot/api');
const BN = require('bn.js');
const fs = require('fs');

const Papa = require('papaparse');

const bn64b = new BN(2).pow(new BN(64));
const bn1e12 = new BN(10).pow(new BN(12));


async function main() {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider });

    let out = process.env.OUT;
    let workers = process.env.WORKERS.split(',').map(x => x.trim());

    const tip = await api.rpc.chain.getHeader();
    const tipNum = tip.number.toNumber();
    const GAP = 50; // 10 mins
    const startNum = tipNum - GAP * 144/2;  // 2 days

    let miners = await Promise.all(workers.map(w => api.query.phalaMining.workerBindings(w)));
    miners = miners.map(m => m.unwrap());

    const rows = [];
    for (let n = tipNum; n > startNum; n -= GAP) {
        const h = await api.rpc.chain.getBlockHash(n);
        let minerData = await Promise.all(miners.map(m => api.query.phalaMining.miners.at(h, m)));
        minerData = minerData.map(m => m.unwrap());

        const row = minerData
            .map(m => ({
                v: m.v.div(bn64b).toString(),
                pInstant: m.benchmark.pInstant.toString(),
                totalRward: m.stats.totalReward.div(bn1e12).toString(),
            }))
            .reduce((row, m, idx) => {
                for (let k in m) {
                    row[`m${idx}_${k}`] = m[k];
                }
                return row;
            }, {});
        rows.push(row);
    }

    const csvData = Papa.unparse(rows);
    if (out) {
        fs.writeFileSync(out, csvData, {encoding: 'utf-8'});
    } else {
        console.log(csvData);
    }
}

main().catch(console.error).finally(() => process.exit());
