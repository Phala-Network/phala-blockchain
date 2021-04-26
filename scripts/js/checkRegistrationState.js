/// Checks the state consistency before and after a register worker call

require('dotenv').config();

const util = require('util')
const { ApiPromise, WsProvider } = require('@polkadot/api');
const { decodeAddress } = require('@polkadot/keyring');
const { u8aToHex } = require('@polkadot/util');


const typedefs = require('@phala/typedefs').latest;


async function main () {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider, types: typedefs });

    const regHeight = parseInt(process.env.REG_HEIGHT);
    const controller = process.env.CONTROLLER;

    const hashBefore = await api.rpc.chain.getBlockHash(regHeight - 1);
    const hashAfter = await api.rpc.chain.getBlockHash(regHeight);

    const stash = await api.query.phala.stash.at(hashAfter, controller);
    const delta = await api.query.phala.pendingExitingDelta.at(hashAfter);

    const onlineWorkerBefore = await api.query.phala.onlineWorkers.at(hashBefore);
    const onlineWorkerAfter = await api.query.phala.onlineWorkers.at(hashAfter);

    const workerInfoAfter = await api.query.phala.workerState.at(hashAfter, stash);
    const workerInfoBefore = await api.query.phala.workerState.at(hashBefore, stash);

    console.log(util.inspect({
        controller,
        stash: stash.toJSON(),
        delta: delta.toJSON(),
        onlineWorkers: {
            onlineWorkerBefore: onlineWorkerBefore.toNumber(),
            onlineWorkerAfter: onlineWorkerAfter.toNumber(),
        },
        workerInfo: {
            before: workerInfoBefore.toJSON(),
            after: workerInfoAfter.toJSON(),
        },
    }, {depth: null}));

    // additional check
    const machienIdBefore = workerInfoBefore.machineId.toJSON();
    const machienIdAfter = workerInfoAfter.machineId.toJSON();
    if (machienIdBefore != machienIdAfter) {
        // Check machineId <==> stash @before
        const oldStash0 = await api.query.phala.machineOwner.at(hashBefore, machienIdBefore);
        console.assert(
            oldStash0.toJSON() === stash.toJSON(),
            'Before reg machineIdBefore is owned by stash');

        // Who owns the old machineId @after?
        const newOwnerOfOldMachine = await api.query.phala.machineOwner.at(hashAfter, machienIdBefore);
        const newWorkerInfoOldMachine = await api.query.phala.workerState.at(hashAfter, newOwnerOfOldMachine);

        console.log('MachineId check', util.inspect({
            newOwnerOfOldMachine: newOwnerOfOldMachine.toJSON(),
            newWorkerInfoOldMachine: newWorkerInfoOldMachine.toJSON(),
        }, {depth: null}));
    }
}

main().catch(console.error).finally(() => process.exit());

