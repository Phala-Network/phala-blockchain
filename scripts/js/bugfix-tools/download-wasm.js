require('dotenv').config();
const fs = require('fs');
const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');
const typedefs = require('@phala/typedefs').khalaDev;

process.env.ENDPOINT = 'wss://polkadot.api.onfinality.io/public-ws'

const { blake2AsU8a } = require('@polkadot/util-crypto');

async function main() {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider, types: typedefs });

    const h = await api.query.paras.codeByHash('0xaa64df7b65965cf4772bdf4561126df1cb3411225c7f50e100c42b54d22bab49');
    const code = h.unwrap();
    const hex = code.toHex();

    fs.writeFileSync('./tmp/code.hex', hex);
}

main().catch(console.error).finally(() => process.exit());
