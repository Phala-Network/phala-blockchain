require('dotenv').config();
const { ApiPromise, WsProvider } = require('@polkadot/api');

const kInterval = 3000;

async function main() {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider });

    let blocknum = 1;
    let lastBlock = 1;
    setInterval(() => {
        const blockPerSec = (blocknum - lastBlock) / kInterval * 1000;
        console.log(`Benchmarking... ${blockPerSec.toFixed(4)} blocks/s`);
        lastBlock = blocknum;
    }, kInterval);

    while (true) {
        const h = await api.rpc.chain.getBlockHash(blocknum);
        const _block = await api.rpc.chain.getBlock(h);
        const _event = await api.query.system.events.at(h);
        const key = api.query.system.events.key();
        const _proof = await api.rpc.state.getReadProof([key], h);
        blocknum++;
    }
}

main().catch(console.error).finally(() => process.exit());
