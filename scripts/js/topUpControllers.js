/**
 * USAGE: node topUpControllers.js
 *
 * ENVs:
 *  - PRIVKEY: sr25519 privkey sURI
 *  - ENDPOINT: full node WS (WSS) RPC endpoint
 *  - TOPUP_PAYOUT_TARGET: the mining payout target account that owns the controllers to top up
 *  - AMOUNT: the amount to top up, integer only
 *  - AMOUNT_MIN: skip the controller if it has this min amount, float accepted
 *  - DRYRUN: 1 means dry-run mode (optional)
 *
 * Example:
 *  TOPUP_PAYOUT_TARGET=<some-payout-address> AMOUNT=10 MIN_AMOUNT=5 node topUpControllers.js
 */

require('dotenv').config();

const { ApiPromise, Keyring, WsProvider } = require('@polkadot/api');
const BN = require('bn.js');

const typedefs = require('../../e2e/typedefs.json');
const bn1e9 = new BN(10).pow(new BN(9));
const bn1e12 = new BN(10).pow(new BN(12));
const kDryRun = parseInt(process.env.DRYRUN || '0') === 1;
const kVerbose = parseInt(process.env.VERBOSE || '0') === 1;

async function getBalances (api, addresses) {
    return await new Promise(async resolve => {
        const unsub = await api.query.system.account.multi(addresses, data => {
            unsub();
            resolve(data.map(data => data.data.free));
        })
    });
}

async function main () {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider, types: typedefs });

    const keyring = new Keyring({ type: 'sr25519' });
    const sender = keyring.addFromUri(process.env.PRIVKEY);

    const payoutTarget = process.env.TOPUP_PAYOUT_TARGET;
    const amount = new BN(process.env.AMOUNT).mul(bn1e12);
    const minAmount = parseFloat(process.env.MIN_AMOUNT);

    console.log('Getting controllers');
    const entries = await api.query.phalaModule.stashState.entries();
    let controllers = [];
    for (let [k, v] of entries) {
        const jsonValue = v.toJSON();
        const controller = jsonValue.controller;

        if (jsonValue.payoutPrefs.target === payoutTarget) {
            if (kVerbose) {
                console.log(`${k.args.map(k => k.toHuman())} =>`, jsonValue);
            }

            controllers.push(controller);
        }
    }
    console.log(`Found ${controllers.length}`);

    console.log('Getting balance');
    const balances = await getBalances(api, controllers);
    const balancesFloat = balances.map(b => b.div(bn1e9).toNumber() / 1e3);
    const topUpTargets = controllers.filter(
        (_, idx) => {
            const shouldTopUp = balancesFloat[idx] <= minAmount;
            if (shouldTopUp && kVerbose) {
                console.log(`${idx}: ${balancesFloat[idx]}`)
            }
            return shouldTopUp;
        });
    console.log(`Found ${topUpTargets.length}`);
    console.log({topUpTargets});

    if (kDryRun) {
        console.log('Exiting because of dryrun');
        return;
    }

    await new Promise(async resolve => {
        const unsub = await api.tx.utility.batch(
            topUpTargets.map(controller =>
                api.tx.balances.transfer(controller, amount))
        ).signAndSend(sender, (result) => {
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
