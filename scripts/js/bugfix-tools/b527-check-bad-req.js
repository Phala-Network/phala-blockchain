require('dotenv').config();
const fs = require('fs');
const { ApiPromise, WsProvider } = require('@polkadot/api');
const typedefs = require('@phala/typedefs').khalaDev;

async function main() {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider, types: typedefs });

    let pools = await api.query.phalaStakePool.stakePools.entries();
    pools = pools.map(([key, value]) => [key.args[0].toNumber(), value.unwrap()]);

    for (let [pid, pool] of pools) {
        const seenUsers = [];

        for (const req of pool.withdrawQueue) {
            const user = req.user.toString();
            if (seenUsers.includes(user)) {
                console.warn('Dup! Should not happen.');
            }
            const userInfo = (await api.query.phalaStakePool.poolStakers([pid, user])).unwrap();
            if (req.shares.gt(userInfo.shares)) {
                console.warn('Dirty req found:', {
                    pid, user,
                    withdrawing: req.shares.toHuman(),
                    has: userInfo.shares.toHuman()
                });
            }
        }
    }
}

main().catch(console.error).finally(() => process.exit());
