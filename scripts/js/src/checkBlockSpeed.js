require('dotenv').config();
const { BN } = require('bn.js');
const { ApiPromise, WsProvider } = require('@polkadot/api');

const typedefs = require('@phala/typedefs').phalaDev;

async function main() {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider, types: typedefs });

    const blocksToCheck = [300, 3600, 7200, 50400];

    const latestBlock = await api.rpc.chain.getHeader();
    const tip = latestBlock.number;

    let h = latestBlock.hash;
    const tsTip = await api.query.timestamp.now.at(h);

    for (const d of blocksToCheck) {
        const n = tip - d;
        h = await api.rpc.chain.getBlockHash(n);
        const ts = await api.query.timestamp.now.at(h);
        const interval = tsTip.sub(ts).div(new BN(d)).toNumber() / 1000;
        console.log(`Last ${d} blocks: avg ${interval}s`);
    }

}

main().catch(console.error).finally(() => process.exit());
