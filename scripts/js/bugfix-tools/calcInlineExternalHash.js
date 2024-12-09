require('dotenv').config();
const fs = require('fs');
const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');

process.env.ENDPOINT = 'wss://phala-rpc.dwellir.com'

const { blake2AsHex } = require('@polkadot/util-crypto');

async function main() {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider });

    const nextExt = (await api.query.democracy.nextExternal()).unwrap();
    console.log([nextExt[0].toHuman(), nextExt[1].toHuman()])
    const rawExt = nextExt[0].toHex();
    const hash = blake2AsHex(nextExt[0].asInline);  // special hash - only hash the content
    console.log({rawExt, hash})
}

main().catch(console.error).finally(() => process.exit());
