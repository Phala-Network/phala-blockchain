// USAGE:
//   ENDPOINT=wss://rococo-rpc.polkadot.io PARAID=30 DUMP=/tmp/phala-prticipants.json node dumpCrowdloan.js

require('dotenv').config();

const { ApiPromise, WsProvider } = require('@polkadot/api');
const { u8aConcat, u8aToHex } = require('@polkadot/util');
const { blake2AsU8a, encodeAddress } = require('@polkadot/util-crypto');
const BN = require('bn.js');
const fs = require('fs');

function createChildKey(trieIndex) {
    return u8aToHex(
        u8aConcat(
            ':child_storage:default:',
            blake2AsU8a(
                u8aConcat('crowdloan', trieIndex.toU8a())
            )
        )
      );
}

async function main () {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider });
    const paraId = parseInt(process.env.PARAID || 30);
    const dumpJson = process.env.DUMP || '';

    // const paraId = 30
    const fund = await api.query.crowdloan.funds(paraId);
    const trieIndex = fund.unwrap().trieIndex;
    const childKey = createChildKey(trieIndex);

    const keys = await api.rpc.childstate.getKeys(childKey, '0x');
    const ss58Keys = keys.map(k => encodeAddress(k));
    console.log(ss58Keys);

    const values = await Promise.all(keys.map(k => api.rpc.childstate.getStorage(childKey, k)));
    const contributions = values.map((v, idx) => ({
        from: ss58Keys[idx],
        data: api.createType('(Balance, Vec<u8>)', v.unwrap()).toJSON(),
    }));

    console.log(contributions);

    if (dumpJson) {
        const jsonStr = JSON.stringify(contributions, undefined, 2);
        fs.writeFileSync(dumpJson, jsonStr, {encoding: 'utf-8'});
    }
}

main().catch(console.error).finally(() => process.exit());
