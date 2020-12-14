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
const dashboard = require('../../tmp/dashboard.json');
const bn1e9 = new BN(10).pow(new BN(9));
const bn1e12 = new BN(10).pow(new BN(12));
const kDryRun = parseInt(process.env.DRYRUN || '0') === 1;

async function getAllControllers (api, stashes) {
    return await new Promise(async resolve => {
        const unsub = await api.query.phalaModule.stashState.multi(stashes, data => {
            unsub();
            resolve(data.map(x => x.controller.toHuman()));
        });
    })
}

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

    const miners = dashboard.dashboard.find(x => x.targetAddress == payoutTarget);
    if (!miners) {
        console.log('Payout address not found; qed.');
        return;
    }

    const stashes = miners.targetState.map(x => x.stashAddress);
    console.log('Getting controllers');
    const controllers = await getAllControllers(api, stashes);
    console.log('Getting balance');
    const balances = await getBalances(api, controllers);
    const balancesFloat = balances.map(b => b.div(bn1e9).toNumber() / 1e3);

    const topUpTargets = controllers.filter((_, idx) => balancesFloat[idx] <= minAmount);
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
