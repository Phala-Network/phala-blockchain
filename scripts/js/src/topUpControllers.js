/**
 * USAGE: node topUpControllers.js
 *
 * ENVs:
 *  - PRIVKEY: sr25519 privkey sURI
 *  - ENDPOINT: full node WS (WSS) RPC endpoint
 *  - CONTROLLER_ADDRESS_FILE: a text file with an address in each line to top up; if this is used the next option will be ignored
 *  - TOPUP_PAYOUT_TARGET: the mining payout target account that owns the controllers to top up
 *  - TARGET_AMOUNT: top up to the specified amount. no-op if the account has more token
 *  - AMOUNT_MIN: skip the controller if it has this min amount, float accepted
 *  - DRYRUN: 1 means dry-run mode (optional)
 *
 * Example:
 *  TOPUP_PAYOUT_TARGET=<some-payout-address> TARGET_AMOUNT=5 node topUpControllers.js
 */

require('dotenv').config();

const { ApiPromise, Keyring, WsProvider } = require('@polkadot/api');
const BN = require('bn.js');
const fs = require('fs');

const typedefs = require('@phala/typedefs').phalaDev;
const bn1e9 = new BN(10).pow(new BN(9));
const bn1e12 = new BN(10).pow(new BN(12));
const kDryRun = parseInt(process.env.DRYRUN || '0') === 1;
const kVerbose = parseInt(process.env.VERBOSE || '0') === 1;

function readAddress(path) {
    try {
        const data = fs.readFileSync(path, {encoding: 'utf-8'});
        return data
            .split('\n')
            .map(x => x.trim())
            .filter(x => !!x);
    } catch (err) {
        return [];
    }
}

async function controllersFromPayout(api, payoutTarget) {
    const entries = await api.query.phala.stashState.entries();
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
    return controllers;
}

async function getBalances (api, addresses) {
    return await new Promise(async resolve => {
        const unsub = await api.query.system.account.multi(addresses, data => {
            unsub();
            resolve(data.map(data => data.data.free));
        })
    });
}

function realToBn(f) {
    return new BN(f * 1e3 | 0).mul(bn1e9);
}

function bnToReal(b) {
    return b.div(bn1e9).toNumber() / 1e3;
}

async function main () {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider, types: typedefs });

    const keyring = new Keyring({ type: 'sr25519' });
    const sender = keyring.addFromUri(process.env.PRIVKEY);

    console.log(`Sender account:`, {
        address: sender.address,
        balance: bnToReal((await api.query.system.account(sender.address)).data.free),
    });

    const targetAmount = parseFloat(process.env.TARGET_AMOUNT)
    let controllers = readAddress(process.env.CONTROLLER_ADDRESS_FILE);
    if (!controllers) {
        console.log('Getting controllers');
        const payoutTarget = process.env.TOPUP_PAYOUT_TARGET;
        controllers = controllersFromPayout(api, payoutTarget)
    }
    console.log(`Found ${controllers.length}`);

    console.log('Getting balance');
    const balances = await getBalances(api, controllers);
    const balancesFloat = balances.map(bnToReal);
    let topUpPlan = controllers
        .map((address, idx) => ({
            address,
            amount: targetAmount - balancesFloat[idx]
        }))
        .filter(({amount}) => amount > 0);
    const total = topUpPlan.reduce((a, x) => a + x.amount, 0);
    console.log({topUpPlan});
    console.log(`Found ${topUpPlan.length}, total: ${total}`);

    topUpPlan = topUpPlan.map(({address, amount}) => ({
        address,
        amount: realToBn(amount),
    }));

    if (kDryRun) {
        console.log('Exiting because of dryrun');
        return;
    }

    await new Promise(async resolve => {
        const unsub = await api.tx.utility.batch(
            topUpPlan.map(({address, amount}) =>
                api.tx.balances.transfer(address, amount))
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
