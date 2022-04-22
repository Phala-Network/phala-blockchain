require('dotenv').config();

const fs = require('fs');

const { ApiPromise, WsProvider } = require('@polkadot/api');

const typedefs = require('@phala/typedefs').khalaDev;

const interested = [
    'phalaMining.MinerSettled',
];

async function main() {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider, types: typedefs });


    const since = parseInt(process.env.SINCE || '1467000');
    const until = parseInt(process.env.UNTIL || '1478000');  // 1478296 + 7200
    const outfile = fs.openSync(process.env.OUT || `./tmp/gkreconDumppedEvents-${since}-${until}.json`, 'a');

    const lastHash = await api.rpc.chain.getBlockHash(until);
    let header = await api.rpc.chain.getHeader(lastHash);
    while(header.number.toNumber() > since) {
        const h = header.hash;
        const blockNumber = header.number.toNumber();

        if (blockNumber % 100 == 0) {
            console.log(blockNumber);
        }

        const events = await api.query.system.events.at(h);
        const targetsRev = events
            .map(event => {
                event.display = `${event.event.section}.${event.event.method}`;
                return event;
            })
            .filter(event => interested.includes(event.display))
            .map(event => ({
                blockNumber,
                event: event.display,
                data: event.event.data.map(d => d.toString())
            }))
            .reverse();
        for (let record of targetsRev) {
            const d = JSON.stringify(record)
            fs.writeSync(outfile, d);
            fs.writeSync(outfile, '\n');
        }
        fs.fdatasyncSync(outfile);
        header = await api.rpc.chain.getHeader(header.parentHash);
    }
    fs.closeSync(outfile);
}

main().catch(console.error).finally(() => process.exit());
