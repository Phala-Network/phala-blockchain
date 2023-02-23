const fs = require('fs');
const Papa = require('papaparse');
const { program } = require('commander');
const { BN } = require('bn.js');
const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');

const bn1e10 = new BN('10000000000');

program
    .argument('<input csv>')
    .argument('<address>')
    .argument('<output csv')
    .option('--endpoint <endpoint>', 'WS endpoint', 'wss://khala-api.phala.network/ws')
    .action((csv, addr, outCsvPath) =>
        main(csv, addr, outCsvPath)
            .then(process.exit)
            .catch(console.error)
            .finally(() => process.exit(-1))
    )
    .parse(process.argv);

async function main(csvPath, addr, outCsvPath) {
    const { endpoint } = program.opts();
    const csvContent = fs.readFileSync(csvPath, { encoding: 'utf-8' });
    const csv = Papa.parse(csvContent, { header: true })
        .data
        .slice(0, 200);

    const entries = csv.map(row => ({
        address: addr,
        block: parseInt(row['Block-number'])
    }));

    const wsProvider = new WsProvider(endpoint);
    const api = await ApiPromise.create({ provider: wsProvider });

    const out = fs.openSync(outCsvPath, 'a');

    const balances = [];
    let i = 0;
    let lastBlock;
    let lastBalance;
    for (const {address, block} of entries) {
        console.log('Row', i++);
        let balance;
        if (block == lastBlock && lastBalance) {
            balance = lastBalance;
        } else {
            const apiAt = await api.at(await api.rpc.chain.getBlockHash(block));
            const account = await apiAt.query.system.account(address);
            balance = account.data.free.div(bn1e10).toNumber() / 100;
        }
        balances.push({block, balance});
        fs.writeSync(out, `${block},${balance.toFixed(2)}\n`);
        lastBlock = block;
        lastBalance = balance;
    }
}
