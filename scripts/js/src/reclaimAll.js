require('dotenv').config();

const { ApiPromise, WsProvider } = require('@polkadot/api');
const BN = require('bn.js');

const { loadJson, writeJson } = require('../src/utils/common');

const typedefs = require('@phala/typedefs').khalaDev;

function chunk(arr, chunkSize) {
    if (chunkSize <= 0) throw "Invalid chunk size";
    var R = [];
    for (var i=0,len=arr.length; i<len; i+=chunkSize)
      R.push(arr.slice(i,i+chunkSize));
    return R;
  }

async function main() {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider, types: typedefs });

    const reconciling = loadJson('./tmp/issue500Reconciling.json');
    const minerPreimages = reconciling.preimage;

    const h = await api.rpc.chain.getBlockHash();
    const apiAt = await api.at(h);

    const nowS = new BN((Date.now() / 1000) | 0);
    const coolDownPeriod = await apiAt.query.phalaMining.coolDownPeriod();
    const latestCoolDown = nowS.sub(coolDownPeriod);

    let miners = await apiAt.query.phalaMining.miners.entries();
    miners = miners.map(([key, value]) => [key.args[0].toString(), value.unwrap()]);

    const minerCD = miners
        .filter(([_, info]) => info.state.toString() == 'MiningCoolingDown');
    console.log(`Found ${minerCD.length} miners in CD`);

    const minerCanReclaim = minerCD
        .filter(([_, info]) => info.coolDownStart.lte(latestCoolDown));
    console.log(`Found ${minerCanReclaim.length} miners ready to reclaim`);

    const minerToReclaim = minerCanReclaim
        .filter(([miner, _]) => miner in minerPreimages)
        .map(([miner, _]) => [miner, minerPreimages[miner].pid, minerPreimages[miner].worker]);
    console.log(`Found ${minerToReclaim.length} miners with preimage to claim`);

    const knownBadPools = [726, 386];
    const minerToReclaimSkipBadOnes = minerToReclaim
        .filter(x => !knownBadPools.includes(x[1]));
    console.log(`After applied post filter: ${minerToReclaimSkipBadOnes.length}`);

    const reclaimChunks = chunk(minerToReclaimSkipBadOnes, 100);
    const txs = reclaimChunks.map(reclaim =>
        api.tx.utility.batchAll(
            reclaim.map(([_miner, pid, worker]) =>
                api.tx.phalaStakePool.reclaimPoolWorker(pid, worker)
            )
        ).toHex()
    );

    console.log(minerToReclaim);
    console.log(txs);

    writeJson('./tmp/reclaimAll-miners.json', {
        minerCD: minerCD.map(([miner, info]) => ({
            miner,
            ready: info.coolDownStart.lte(latestCoolDown),
            preimage: minerPreimages[miner],
        })),
        reclaim: minerToReclaimSkipBadOnes,
        txs,
    });
}

main().catch(console.error).finally(() => process.exit());
