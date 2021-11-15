require('dotenv').config();

const { ApiPromise, WsProvider } = require('@polkadot/api');
const { loadJson } = require('../src/utils/common');

const typedefs = require('@phala/typedefs').khalaDev;

function propose(api, call) {
    const lenthBound = call.toU8a().length + 10;
    return api.tx.council.propose(3, call, lenthBound);
}

async function main() {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider, types: typedefs });

    const reconciling = loadJson('./tmp/issue500Reconciling.json');

    const tx = api.tx.phalaStakePool.backfillIssue500Reclaim(
        reconciling.missingStake.map(
            ({pid, worker, origStake, slashed}) =>
            [pid, worker, origStake, slashed]
        )
    );
    const proposalTx = propose(api, tx);
    console.log(proposalTx.toHex());
}

main().catch(console.error).finally(() => process.exit());
