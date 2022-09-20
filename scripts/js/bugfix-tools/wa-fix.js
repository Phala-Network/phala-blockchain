require('dotenv').config();
const fs = require('fs');
const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');
const typedefs = require('@phala/typedefs').khalaDev;
const { numberToHex, hexToU8a, u8aConcat, u8aToHex } = require('@polkadot/util');
const { xxhashAsHex, xxhashAsU8a } = require('@polkadot/util-crypto');

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


const newWorkers = [
    '0x265eaf0eadf24f9c52cc8d017033ecb75e73f4d14d0a507d30653778b842062b',
    '0x4e2f7ff1f3e06eb6ec1a87fabef0f2f0bede415c454c41bb4022bd6256404038',
    '0xd40bd63aef9254d813f32fd7a9e63a1b1afbab238b1fbaf1586df65034ef1e26',
    '0x9236b5efe662018307e9af37658e39c9434d842b7ac25fcfc4a7b144869e9c44',
    '0xf89ea4732efe6880baa83ff16fc22ec778a735f8700984a87ee5244f26acd34f',
    '0x5a4a126d65ea7722f9554a56a89e6fc211f975cd5951315242117041d53f9066',
]

async function main() {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api0 = await ApiPromise.create({ provider: wsProvider, types: typedefs });

    // 1548358
    // 0x947c83167968b82f6e0312debe9486933440b32611153078cc9315623bf1a98c

    // 1548357
    // 0x945619ac9c557836a0d2620d9e2231e0c0fd58f3ec6695851363e4261172dbeb

    // 1548356
    // 0x0390ce9307007fa3a62149e5b8ce1fda40d281ce5d61251762735b7f677f8e2e

    const prefix = api0.query.phalaStakePool.workerAssignments.keyPrefix();

    function waStorageKey(worker) {
        const hash = xxhashAsHex(worker);
        return u8aToHex(u8aConcat(hexToU8a(prefix), hexToU8a(hash), worker));
    }

    const api = await api0.at('0x945619ac9c557836a0d2620d9e2231e0c0fd58f3ec6695851363e4261172dbeb');
    const data = await api.query.phalaStakePool.workerAssignments.entries();
    // api.query.
    const wa = data
        .map(([key, value]) => [key.args[0].toString(), value.unwrap()]);

    const changes = wa.map(([key, value]) => {
        return [
            waStorageKey(key),
            value.toU8a(),
            // api0.createType('u64', value).toU8a(),
        ]
    })
    // const conflict = wa.filter(([k,v]) => newWorkers.includes(k));
    console.log(changes);

    // FOR TEST
    // const call = api0.tx.system.setStorage(
    //     changes.slice(0, 1).map(([kHex, vU8a]) => [kHex, u8aToHex(vU8a)])
    // );
    // console.log(wa[0]);
    // console.log(call.toHex());

    const call = api0.tx.system.setStorage(
        changes.map(([kHex, vU8a]) => [kHex, u8aToHex(vU8a)])
    );
    console.log('preimage');
    const hexPreimage = call.method.toHex();
    console.log('hex len', hexPreimage.length);
    fs.writeFileSync('tmp/wa-fix-call-preimage.hex', hexPreimage)

    // const sudoCall = api0.tx.sudo.sudo(call);
    // console.log(sudoCall.toHex());

    // const k = api.query.phalaStakePool.workerAssignments.keyPrefix();
    // const hash = xxhashAsHex('0x4e2f7ff1f3e06eb6ec1a87fabef0f2f0bede415c454c41bb4022bd6256404038');
    // console.log(k + hash + '0x4e2f7ff1f3e06eb6ec1a87fabef0f2f0bede415c454c41bb4022bd6256404038');


    // 0x9708ddcf89326bf4f4428dd135287d5119607b9c9a8737bdc9a8097e5575f8df5959e67afaaedd694e2f7ff1f3e06eb6ec1a87fabef0f2f0bede415c454c41bb4022bd6256404038

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
