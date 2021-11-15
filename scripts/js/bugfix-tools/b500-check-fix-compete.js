require('dotenv').config();

const { ApiPromise, WsProvider } = require('@polkadot/api');
const BN = require('bn.js');
const { Decimal } = require('decimal.js');

const { balanceBnToDec, calculateReturnSlash } = require('../src/utils/palletUtils');
const { loadJson } = require('../src/utils/common');
const { FixedPointConverter } = require('../src/utils/fixedUtils');
const fp = new FixedPointConverter();

const typedefs = require('@phala/typedefs').khalaDev;

const bn0 = new BN(0);
// const bn64b = new BN(2).pow(new BN(64));
const bn1e12 = new BN(10).pow(new BN(12));
const bnEps = new BN(100);

function bnCmp(a, b) {
    if (a.gt(b.add(bnEps))) {
        return 1;
    } else if (b.gt(a.add(bnEps))) {
        return -1;
    } else {
        return 0;
    }
}

async function main() {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider, types: typedefs });

    const reconciling = loadJson('./tmp/issue500Reconciling.json');
    const h = await api.rpc.chain.getBlockHash(reconciling.latestBlock);
    const apiAt = await api.at(h);

    let pools = await apiAt.query.phalaStakePool.stakePools.entries();
    pools = pools
        .map(([key, value]) => [key.args[0].toNumber(), value.unwrap()])
        .filter(([_, value]) => value.releasingStake.gt(bn0));

    const minerPreimages = reconciling.preimage;

    // Dump stakers & miners
    let stakes = await apiAt.query.phalaMining.stakes.entries();
    stakes = stakes.map(([key, value]) => [key.args[0].toString(), value.unwrap()]);
    let miners = await apiAt.query.phalaMining.miners.entries();
    miners = miners.map(([key, value]) => [key.args[0].toString(), value.unwrap()]);

    // All the CD miners => their releasing stake
    const stakesMap = stakes.reduce((map, [k, v]) => {
        map[k] = v;
        return map;
    }, {});
    const cdMinerReleasing = miners
        .filter(([_, info]) => info.state.toString() == 'MiningCoolingDown')
        .map(([miner, info]) => {
            const [returned, slashed] = calculateReturnSlash(info, stakesMap[miner]);
            return [miner, {returned, slashed}];
        }, {});

    const poolKnownReleasing = cdMinerReleasing.reduce((map, [miner, releasing]) => {
        if (!(miner in minerPreimages)) {
            console.warn('Miner not found', miner);
        } else {
            const { pid } = minerPreimages[miner];
            const n = map[pid] || bn0;
            map[pid] = n.add(releasing.returned);
        }
        return map;
    }, {});

    const diff = pools.flatMap(([pid, pool]) => {
        const knownReleasing = poolKnownReleasing[pid] || bn0;
        if (bnCmp(pool.releasingStake, knownReleasing) != 0) {
            return [{
                pid,
                cdSum: knownReleasing.toString(),
                poolReleasing: pool.releasingStake.toString(),
            }];
        } else {
            return [];
        }
    });

    console.log('---- Before reconcile ----')
    console.log(diff);
    console.log(`Diff: ${diff.length}`);

    // pool => stake(BN)

    console.log('---- Reconciled ----')

    const poolMissingStake = reconciling.missingStake
        .map(s => [s.pid, new BN(s.returned)])
        .reduce((map, [pid, returned]) => {
            map[pid] = (map[pid] || bn0).add(returned);
            return map;
        }, {});

    const diffReconciled = pools.flatMap(([pid, pool]) => {
        let missed, hasMissed;
        if (poolMissingStake[pid]) {
            missed = poolMissingStake[pid];
            hasMissed = true;
        } else {
            missed = bn0;
            hasMissed = false;
        }
        const present = poolKnownReleasing[pid] || bn0;
        const known = missed.add(present);
        if (bnCmp(pool.releasingStake, known) != 0) {
            const decCdSum = balanceBnToDec(known);
            const poolReleasing = balanceBnToDec(pool.releasingStake);

            return [{
                pid,
                cdSum: known.toString(),
                poolReleasing: pool.releasingStake.toString(),
                diff: poolReleasing.sub(decCdSum).toString(),
                hasMissed,
            }];
        } else {
            return [];
        }
    });

    console.log(diffReconciled);
    console.log(`Diff: ${diffReconciled.length}`);
}

main().catch(console.error).finally(() => process.exit());
