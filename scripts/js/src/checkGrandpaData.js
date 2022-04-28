// Scan the GRANDPA consensus data on the blockchain
//
// USAGE:
//   ENDPOINT=ws://<ip-to-relaychain>:<port> FROM=<block-num> TO=<block-num> node src/checkBlockJustifications.js 2>/dev/null
//
// `2>/dev/null` because we want to ignore some annoying warning from Polkadot.js

require('dotenv').config();
const { ApiPromise, WsProvider } = require('@polkadot/api');

async function main() {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api0 = await ApiPromise.create({ provider: wsProvider });

    const numFrom = parseInt(process.env.FROM) || 0;
    const numTo = parseInt(process.env.TO);

    let hash;
    if (numFrom && numFrom <= numTo) {
        hash = await api0.rpc.chain.getBlockHash(numTo);
    } else {
        hash = await api0.rpc.chain.getFinalizedHead();
    }

    // Scan backward
    while (true) {
        const block = await api0.rpc.chain.getBlock(hash);
        const jlen = block.justifications.toU8a().length;
        const num = block.block.header.number.toNumber();

        const api = await api0.at(hash);
        const sessId = (await api.query.session.currentIndex()).toNumber();
        const setId = (await api.query.grandpa.currentSetId()).toNumber();

        console.log(num, hash.toHex(), {jlen, sessId, setId});
        if (num <= numFrom) {
            break;
        }
        hash = block.block.header.parentHash;
    }
}

main().then(process.exit).catch(console.error).finally(() => process.exit(0));
