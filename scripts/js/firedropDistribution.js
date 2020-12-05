require('dotenv').config();

const { ApiPromise, Keyring, WsProvider } = require('@polkadot/api');
const BN = require('bn.js');

const typedefs = require('../../e2e/typedefs.json');
const bn1e12 = new BN(10).pow(new BN(12));

const poc3Prize = [{
    // Assume amounts are integer
    data: [
        ['44AHCQtUiEzf1aE9QDQKP6wLAERhDpYdLTLsDNxnqUC4gK4G', 8499],
        ['41dfUfy5JexS1L9CAf1gHmKws8GHbXBDXeJvpfdpVxRvA894', 98546],
        ['44Uxw91hz88zLytWwWwTbH1c1SvTset4cvAwxree2oVdAyQk', 27692],
        ['45FKTTyMTVqGFysvv2JtDRodWMNuYWbwARnyjkD6bZXTuJn5', 51688],
        ['46EzvipMtzQ7z6YscCXcZjhobaLLxjuicookgf78keSrBfMY', 15851],
        ['42t19Y26GGZbZyp4FPcJKEdKmyZt7kYdfoovf6DFjgB1BVKi', 15716],
        ['41UoKhSWFt2ShPYvbphs9RaGynGU2Nxbij2kgAc673B4p2X8', 107010],
        ['43qMqcraek34k1Ass6xTJhySBSuD76LxHHwUQLDAdumLdunZ', 53371],
        ['444reqUqZst1dPxJP8e3ACNaXR41CdViiWWtEkzu8eUk4wPx', 26061],
        ['43t4PmV9GqCcG72ns5A76bLU4524w4uxiNUaV6oveKUWvJbG', 186892],
        ['44jhFBS1H9xmYW7qPKZMVREa8KnQqPey5N5Ec9PmKTHRKGo7', 12802],
        ['43UKqmgoGJzJctvAhNbnUaf5J8D7CzrgXB9oxdTFrfUKJqLm', 72876],
        ['44khVN5Gs4VgRh33189seEqwQoMvA74CiG1Vt6qAEYKM7jdd', 10196],
        ['45RaHh6wCR1uU1DN2P2LK76hYASuMY9HKvsKsW4NhM58thPm', 32800],
    ],
    context: {
        "blockNum": 110277,
        "blockHash": "0x50294f31df05190e696241e786c21476e91772847de485b58644fcfefdea880e"
    }
}];

async function getPayoutAddress (api, hash, stash) {
    const stashInfo = await api.query.phalaModule.stashState.at(hash, stash);
    return stashInfo.payoutPrefs.target;
}

async function main () {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider, types: typedefs });

    const keyring = new Keyring({ type: 'sr25519' });
    const root = keyring.addFromUri(process.env.PRIVKEY);

    const {data, context} = poc3Prize[0];

    const argAddrs = [];
    const argAmounts = [];
    for (let [addr, amount] of data) {
        const payoutAddr = await getPayoutAddress(api, context.blockHash, addr);
        argAddrs.push(payoutAddr);
        argAmounts.push(new BN(amount).mul(bn1e12));
    }

    await new Promise(resolve => {
        const unsub = api.tx.sudo.sudo(api.tx.phalaModule.forceAddFire(argAddrs, argAmounts))
            .signAndSend(root, (result) => {
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
