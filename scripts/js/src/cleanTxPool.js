/* USAGE:

```bash
while true; do
  ENDPOINT=ws://127.0.0.1:9144 timeout 10 node cleanTxPool.js
  sleep 6
done
```
*/

require('dotenv').config();
const { ApiPromise, WsProvider } = require('@polkadot/api');

const typedefs = require('@phala/typedefs').khalaDev;

async function main() {
    const wsProvider = new WsProvider(process.env.ENDPOINT || 'ws://localhost:9944');
    const api = await ApiPromise.create({ provider: wsProvider, types: typedefs });

    const xt = await api.rpc.author.pendingExtrinsics();

    // Find problmatic pools
    const pools = await api.query.phalaStakePool.withdrawalQueuedPools.entries();
    const blacklist = pools
        .flatMap(([_key, value]) => value.unwrap())
        .map(i => i.toNumber());
    console.log({blacklist});

    // Filter pending extrinsics
    const contributeHash = xt
        .filter(x =>
            x.method.section == 'phalaStakePool'
            && x.method.method == 'contribute'
            && blacklist.includes(x.method.args[0].toNumber()))
        .map(x => ({Hash: x.hash.toHex()}));
    console.log({toRemove: contributeHash});

    // Remove pending extrinsics
    const result = await api.rpc.author.removeExtrinsic(contributeHash);
    console.dir(result.toHuman(), {depth: 5});
}

main().catch(console.error).finally(() => process.exit());
