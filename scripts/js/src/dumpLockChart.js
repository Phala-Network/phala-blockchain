// Utility to dump the account stake info across blocks
//
// USAGE:
//   ACCOUNT=<your-account> PID=<pid> node src/dumpLockChart.js
//
// Parameters:
//   SINCE: the starting block
//   UNTIL: the end block
//   STEP: the interval of the block to take snapshots
//   ENDPOINT: the substrate ws endpoint (archive node is required) (default: ws://localhost:9944)
//   OUT: the output tsv file path (default: ./tmp/dump-<account>.csv

require('dotenv').config();

const fs = require('fs');

const BN = require('bn.js');
const Papa = require('papaparse');
const { ApiPromise, WsProvider } = require('@polkadot/api');
const bn1e12 = new BN(10).pow(new BN(12));

/// Returns the api at a certain block height
async function getApiAt(api, height) {
    const h = await api.rpc.chain.getBlockHash(height);
    return await api.at(h);
}

/// Simply returns the integer part of the PHA value
function pha(value) {
    return value.div(bn1e12).toNumber();
}

/// Parses the timestamp number object to local date
function date(ts) {
    return new Date(ts.toNumber() * 1000).toLocaleString();
}

/// Snapshots the pool status with the given pid and account
async function poolStatus(apiAt, pid, account) {
    // Default value to return
    let status = {
        releasing: 0,
        free: 0,
        lockedInPool: 0,
        withdrawing: 0,
        startTime: ''
    };
    // Try to add pool and withdraw infomation
    let poolInfo = await apiAt.query.phalaStakePool.stakePools(pid);
    if (poolInfo.isSome) {
        poolInfo = poolInfo.unwrap();
        const w = poolInfo.withdrawQueue.find(r => r.user.toString() == account);
        status = {
            ...status,
            releasing: pha(poolInfo.releasingStake),
            free: pha(poolInfo.freeStake),
            withdrawing: w ? pha(w.shares) : 0,
            startTime: w ? date(w.startTime) : '',
        };
    }
    // Try to add the pool staking information
    let poolStaker = await apiAt.query.phalaStakePool.poolStakers([pid, account]);
    if (poolStaker.isSome) {
        const s = poolStaker.unwrap();
        status = {
            ...status,
            lockedInPool: pha(s.locked),
        };
    }

    return status;
}

async function main() {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider });

    // Parse the params
    const since = parseInt(process.env.SINCE || '411774');
    const until = parseInt(process.env.UNTIL || '831721');
    const step = parseInt(process.env.STEP || '7200');
    const pid = parseInt(process.env.PID) || -1;
    const account = process.env.ACCOUNT.trim();
    const outfile = process.env.OUT || `./tmp/lock-${account}.csv`;

    // Scan the blocks and sample data
    const rows = [];
    for (let blockNumber = since; blockNumber <= until; blockNumber += step) {
        const h = await api.rpc.chain.getBlockHash(blockNumber);
        const apiAt = await api.at(h);

        // Add the total locked amount of this account
        let amountLocked = 0;
        const locks = await apiAt.query.balances.locks(account);
        const lock = locks.filter(l => l.id.toHuman() == 'phala/sp');
        if (lock.length > 0) {
            amountLocked = pha(lock[0].amount);
        }

        // Add pool related status info if there's PID specified.
        let p = {};
        if (pid >= 0) {
            p = await poolStatus(apiAt, pid, account);
        }
        rows.push({blockNumber, amountLocked, ...p});

        console.log(blockNumber, amountLocked);
    }

    // Output a tsv file
    const csv = Papa.unparse(rows, {delimiter: '\t'});
    fs.writeFileSync(outfile, csv, {encoding: 'utf-8'});
}

main().catch(console.error).finally(() => process.exit());
