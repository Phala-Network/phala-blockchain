/** Force reset worker ingress utility (not ready for use)
 * not ready for use
 */

require('dotenv').config();

const { ApiPromise, Keyring, WsProvider } = require('@polkadot/api');
const BN = require('bn.js');

const bn1e12 = new BN(10).pow(new BN(12));
const data = require('./tmp/controllers.json');
const kDryRun = parseInt(process.env.DRYRUN || '0') === 1;  // TODO

async function main () {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider });

    const keyring = new Keyring({ type: 'sr25519' });
    const accountKeys = data.slice(1000 /*TODO*/).map(({account}) => {
        const obj = JSON.parse(account);
        return keyring.addFromUri(obj.secret_phrase);
    });

    let totalReset = 0;
    const bnMin = bn1e12.muln(2);
    const r = await Promise.all(accountKeys.map(async (p) => {
        const data = await api.query.system.account(p.address);
        if (data.data.free.lte(bnMin)) {
            return;
        }
        const seq = await api.query.phala.workerIngress(p.address);
        if (seq.toNumber() == 0) {
            return;
        }
        console.log('reset', p.address);
        await api.tx.phala.resetWorker().signAndSend(p);
        totalReset++;
    }));
    console.log(r);
    console.log({totalReset});
}

main().catch(console.error).finally(() => process.exit());
