require('dotenv').config();
const fs = require('fs');
const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');
const typedefs = require('@phala/typedefs').khalaDev;

process.env.ENDPOINT = 'ws://127.0.0.1:9944'

const { blake2AsU8a } = require('@polkadot/util-crypto');

/// encodedExtrinsic must be derived with toHex({method: true})
function sign(encodedExtrinsic, pair, options={}) {
    const encoded = encodedExtrinsic.length > 256
        ? blake2AsU8a(encodedExtrinsic)
        : encodedExtrinsic;
    const sigData = pair.sign(encoded, options);
    return { address: pair.address, data: sigData };
}

async function createPayload(api, call, address, options) {
    const signingInfo = await api.derive.tx.signingInfo(address, undefined, undefined);


    const payloadOpts = {
        blockHash: api.genesisHash,
        genesisHash: api.genesisHash,
        runtimeVersion: api.runtimeVersion,
        signedExtensions: api.registry.signedExtensions,
        version: api.extrinsicType,
        ...options
    };

    const payload = call.inner.signature.createPayload(call, payloadOpts);
    return {
        payload,
        options: payloadOpts,
    }
}

function mergeDetachedSignature(call, payload, sig) {
    call.inner.addSignature(sig.address, sig.data, payload);
}

async function main() {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider, types: typedefs });


    let size = 0;
    const start = 14
    for (let i = 10000; i < 10100; i++) {
        const h = await api.rpc.chain.getHeader(
            await api.rpc.chain.getBlockHash(100)
        );
        size += h.toU8a().length;
    }
    console.log(size / 100);
    return;

    // const fHash = await api.rpc.chain.getFinalizedHead();
    // const fHead = await api.rpc.chain.getHeader(fHash);
    // console.log(fHead.toHuman());

    // const p = await api.rpc.state.getPairs('0xa37f719efab16103');
    // console.log(p.toHuman());

    const keyring = new Keyring({type: 'sr25519'});
    const alice = keyring.addFromUri('//Alice');

    const transfer = api.tx.balances.transfer(alice.address, '1000000');

    // // Detached sign
    // let signingMaterials;
    // {
    //     signingMaterials = createPayload(api, transfer, {nonce:})
    // }

    console.log('unsigned', transfer.toHex());
    await transfer.signAsync(alice);
    console.log('signed', transfer.toHex());
}

main().catch(console.error).finally(() => process.exit());
