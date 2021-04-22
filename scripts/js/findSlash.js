require('dotenv').config();

const { ApiPromise, Keyring, WsProvider } = require('@polkadot/api');
const { decodeAddress } = require('@polkadot/keyring');
const { blake2AsU8a } = require('@polkadot/util-crypto');
const { u8aToHex } = require('@polkadot/util');
const BN = require('bn.js');

const typedefs = require('@phala/typedefs').latest;
const kDryRun = parseInt(process.env.DRYRUN || '0') === 1;

async function getWorkerSnapshotAt(api, hash) {
    // Get all worker state
    const workerState = {};
    const entries = await api.query.phala.workerState.entriesAt(hash);
    for (let [k, v] of entries) {
        workerState[k.args[0].toHuman()] = v.toJSON();
    }
    // Attach lastActivity (blocknum)
    const activityEntries = await api.query.phala.lastWorkerActivity.entriesAt(hash);
    for (let [k, v] of activityEntries) {
        const stash = k.args[0].toHuman();
        if (stash in workerState) {
            workerState[stash].lastActivity = v.toNumber();
        }
    }
    return {
        allWorkerMap: workerState,
        onlineWorkers: Object.entries(workerState)
            .filter(([_, v]) => !!v.state.Mining || 'MiningStopping' in v.state),
    };
}

async function getRewardSeedAt(api, hash, blocknum) {
    return await api.query.phala.blockRewardSeeds.at(hash, blocknum);
}

async function main () {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider, types: typedefs });

    const keyring = new Keyring({ type: 'sr25519' });
    const sender = keyring.addFromUri(process.env.PRIVKEY);

    const { hash, number } = await api.rpc.chain.getHeader();
    const blocknum = number.toNumber();

    const rewardWindow = (await api.query.phala.rewardWindow()).toNumber();
    const slashWindow = (await api.query.phala.slashWindow()).toNumber();

    const { onlineWorkers } = await getWorkerSnapshotAt(api, hash);
    onlineWorkers.forEach(([k, v]) => {
        const pubkey = v.pubkey;
        const pkh = blake2AsU8a(pubkey);
        // Weird enough but yeah Parity decided to serialize it in BE
        // See also: https://github.com/paritytech/parity-common/blob/3ad905d35ed5009547747ae9455f949a458123f2/uint/src/uint.rs#L1347
        v.bnPkh = new BN(pkh, undefined, 'be');
    })

    const allToSlash = new Map();
    const offlineAccounts = {};
    for (let i = blocknum - rewardWindow; i > blocknum - slashWindow; i--) {
        const seed = await getRewardSeedAt(api, hash, i);
        const toSlash = onlineWorkers.filter(([k, v]) => {
            if (v.lastActivity && v.lastActivity >= i) {
                // They have already claimed the reward.
                return false;
            }
            // And they hit the onlineTarget
            const x = seed.seed.xor(v.bnPkh);
            return x.lte(seed.onlineTarget);
        })
        toSlash.forEach(([k, _]) => {
            if (!(k in offlineAccounts)) {
                offlineAccounts[k] = true;
            }

            if (allToSlash.get(i) === undefined) {
                allToSlash.set(i, []);
            }
            allToSlash.get(i).push(k);
        })
        console.log(i, toSlash.map(([k, _]) => k));
    }
    const offlineAccountsLength = Object.keys(offlineAccounts).length;
    console.log('Workers we can slash:', offlineAccountsLength);

    if (kDryRun) {
        console.log('Exiting because of dryrun');
        return;
    }

    if (offlineAccountsLength === 0) {
        console.log('No worker can slash, exit.');
        return;
    }

    await new Promise(async resolve => {
        let calls = []
        for (const [blockNum, accounts] of allToSlash) {
            for (const account of accounts) {
                calls.push(api.tx.phala.reportOffline(account, blockNum))
            }
        }
        const unsub = await api.tx.utility
            .batch(calls)
            .signAndSend(sender, (result) => {
                console.log(`Current status is ${result.status}`);
                if (result.status.isInBlock) {
                    console.log(`Transaction included at blockHash ${result.status.asInBlock}`);
                } else if (result.status.isFinalized) {
                    console.log(`Transaction finalized at blockHash ${result.status.asFinalized}`);
                    unsub();
                    resolve();
                }
            });
    });
}

main().catch(console.error).finally(() => process.exit());

