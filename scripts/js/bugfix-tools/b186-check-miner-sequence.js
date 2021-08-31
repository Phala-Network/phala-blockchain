// Learn more at: https://github.com/Phala-Network/phala-blockchain/issues/186

require('dotenv').config();

const { ApiPromise, WsProvider } = require('@polkadot/api');

const typedefs = require('@phala/typedefs').phalaDev;

async function hashOfHeight(api, height) {
    return await api.rpc.chain.getBlockHash(height);
}

async function seqAt(api, height, stash) {
    return (await api.query.phala.workerIngress.at(
        await hashOfHeight(api, height),
        stash,
    )).toNumber(); 
}

async function miningStateAt(api, height, stash) {
    return (await api.query.phala.workerState.at(
        await hashOfHeight(api, height),
        stash
    )).toHuman();
}

async function lastActivityAt(api, height, stash) {
    return (await api.query.phala.lastWorkerActivity.at(
        await hashOfHeight(api, height),
        stash,
    )).toNumber(); 
}

async function main () {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider, types: typedefs });

    const stash = '45psUjc7jD6Nj3y8o27dxif71CGnTE7YgVTd17jkFqWWhCf5';
    const controller = (await api.query.phala.stashState.at(
        await hashOfHeight(api, 1820543),
        stash
    )).toJSON().controller;
    
    const seqAfterReg = await seqAt(api, 1820543, stash);

    const miningStatusBegin = await miningStateAt(api, 1820640, stash);
    const activityBegin = await lastActivityAt(api, 1820640, stash);

    const seqAfterHit1 = await seqAt(api, 1821428, stash);
    const miningStatusHit1 = await miningStateAt(api, 1821428, stash);
    const activityHit1 = await lastActivityAt(api, 1821428, stash);

    const seqAfterHit2 = await seqAt(api, 1821587, stash);
    const miningStatusHit2Before = await miningStateAt(api, 1821587-1, stash);
    const miningStatusHit2 = await miningStateAt(api, 1821587, stash);
    const activityHit2 = await lastActivityAt(api, 1821587, stash);

    console.dir({
        controller,
        seqAfterReg,

        miningStatusBegin,
        activityBegin,

        seqAfterHit1,
        miningStatusHit1,
        activityHit1,

        seqAfterHit2,
        miningStatusHit2Before,
        miningStatusHit2,
        activityHit2,
    }, {
        depth: 5
    });
}

main().catch(console.error).finally(() => process.exit());
