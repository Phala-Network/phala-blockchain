require('dotenv').config();
const fs = require('fs');
const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');
const typedefs = require('@phala/typedefs').khalaDev;

process.env.ENDPOINT = 'ws://127.0.0.1:9144'

const { blake2AsU8a } = require('@polkadot/util-crypto');
const { u8aToHex } = require('@polkadot/util');

async function main() {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider, types: typedefs });

    const tweakedVersion = api.createType('RuntimeVersion', {
        ...api.runtimeVersion,
        specVersion: 0,
    });
    api._runtimeVersion = tweakedVersion;
    api._rx.runtimeVersion = tweakedVersion;
    // Object.defineProperty(api, 'runtimeVersion', {
    //     get() {
    //         return tweakedVersion;
    //     }
    // });

    console.log(api.runtimeVersion.toHuman());

    const keyring = new Keyring({type: 'sr25519'});
    const pair = keyring.addFromUri('/khala');

    // const origSign = pair.sign;
    // pair.sign = (...args) => {
    //     console.log('sign called!', args);
    //     const r = origSign(...args);
    //     console.log('sign returned!');
    //     return r;
    // }

    // const call = api.tx.system.remark(`x`);

    const rawCode = fs.readFileSync(
        '/home/h4x/workspace/khala-parachain/tmp/wasm/522f571f5a8a0a655a97a204eceb5255062210f8/production/phala_parachain_runtime.compact.compressed.wasm',
    )
    const codeHex = u8aToHex(rawCode);
    console.log('Code:', codeHex.slice(0, 100));

    const call = api.tx.sudo.sudoUncheckedWeight(
        api.tx.system.setCode(codeHex),
        10000,
    );

    const r = await call.signAndSend(pair);
    console.log(r);
}

main().catch(console.error).finally(() => process.exit());
