require('dotenv').config();
const { ApiPromise, WsProvider } = require('@polkadot/api');
const BN = require('bn.js');

const bn2e64 = new BN(1).shln(64);

async function main() {
    const wsProvider = new WsProvider("ws://127.0.0.1:9944");
    const api = await ApiPromise.create({ provider: wsProvider });

    // Take a snapshot of the pre-migration staking data
    let preHeight = 2916832;
    const hash0 = await api.rpc.chain.getBlockHash(preHeight);
    const api0 = await api.at(hash0);
    const stakerData = await api0.query.phalaStakePool.poolStakers.entries();
    const poolData = await api0.query.phalaStakePool.stakePools.entries();
    const stakerItems = stakerData.map(([k, v]) => [[k.args[0][0], k.args[0][1]], v.unwrap()]);
    const poolItems = poolData.map(([k, v]) => [k.args[0], v.unwrap()]);

    // Post-migration Stake Pool v2 data
    let afterHeight = 2936526;
    const hash1 = await api.rpc.chain.getBlockHash(afterHeight);
    const api1 = await api.at(hash1);
    const newPoolData = await api1.query.phalaBasePool.pools.entries();
    const newPoolItems = newPoolData.map(([k, v]) => [k.args[0], v.unwrap()]);

    let oldPools = {};
    let newPools = {};
    for (let [k, v] of poolItems) {
        oldPools[k.toString()] = v;
    }
    for (let [k, v] of  newPoolItems) {
        newPools[k.toString()] = v;
    }

    let total = new BN(0);
    let items = 0;

    // Iterate over the user staking entrys (pid, user)
    for (let [[bnPid, user], value] of stakerItems) {
        const pid = bnPid.toNumber();
        const newPool = newPools[pid];
        const newBasePool = newPool.asStakePool.basepool;

        if (newBasePool.totalShares.isZero()) {
            continue;
        }

        // Reconstruct the claimable reward before the migration
        // - prePending: the pending reward (share * reward_acc - debt)
        // - preClaimable: sum of prePending and availableRewards
        const prePending = value.shares
            .mul(oldPools[pid].rewardAcc)
            .div(bn2e64)
            .sub(value.rewardDebt);
        const preClaimable = value.availableRewards.add(prePending);

        // Reconstruct the share gain caused by the reward distribution, which is
        // (share * (sharePrice - 1))
        const afterGain = value.shares
            .mul(newBasePool.totalValue.sub(newBasePool.totalShares))
            .div(newBasePool.totalShares);

        if (!preClaimable.isZero() && !afterGain.isZero() && preClaimable.gt(afterGain)) {
            // The missing claimable reward is the diff of the two number
            let missing = preClaimable.sub(afterGain);
            total = total.add(missing);
            items++;
            // Output as a CSV line
            console.log(`${pid},${user.toString()},${preClaimable.toString()},${afterGain.toString()},${missing.toString()}`);
        }
    }

    console.log(`Total,${total.toString()}`);
    console.log(`Items,${items}`);
}

main().catch(console.error).finally(() => process.exit());
