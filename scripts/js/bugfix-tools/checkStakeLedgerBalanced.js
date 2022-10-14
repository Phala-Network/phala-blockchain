/// Checks the state consistency before and after a register worker call

require('dotenv').config();

const { ApiPromise, WsProvider } = require('@polkadot/api');

async function main () {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider });

    const h = await api.rpc.chain.getBlockHash();

    const ledger = await api.query.phalaStakePool.stakeLedger.entriesAt(h);
    const locks = await api.query.balances.locks.entriesAt(h);

    let userLocked = ledger.reduce((map, [key, value]) => {
        const k = key.args[0].toHuman();
        map[k] = value.toString();
        return map;
    }, {});

    let actualLocked = locks.reduce((map, [key, value]) => {
        const k = key.args[0].toHuman();
        const lock = value.find(v => v.id.toString() == '0x7068616c612f7370');
        if (lock) {
            map[k] = lock.amount.toString();
        }
        return map;
    }, {});

    // We expted no output. The ledger must matches the actual lock.
    for (const [k, v] of Object.entries(userLocked)) {
        if (v != "0" && actualLocked[k] != v) {
            console.warn('diff', k, v, actualLocked[k]);
        }
    }
}

main().catch(console.error).finally(() => process.exit());
