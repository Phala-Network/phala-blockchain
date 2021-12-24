require('dotenv').config();
const { ApiPromise, WsProvider } = require('@polkadot/api');

async function main() {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider });

    const finalizedHash = await api.rpc.chain.getFinalizedHead();
    const finalizedBlock = await api.rpc.chain.getBlock(finalizedHash);

    let hash = finalizedHash;
    let block = finalizedBlock;
    while (true) {
        const jlen = block.justification.length;
        const num = block.block.header.number.toNumber();
        console.log(num, hash.toHex(), jlen);
        if (num == 0) {
            break;
        }

        hash = block.block.header.parentHash;
        block = await api.rpc.chain.getBlock(hash);
    }

    process.exit(0);
}

try { main(); }
catch(err) { console.error(err); }
