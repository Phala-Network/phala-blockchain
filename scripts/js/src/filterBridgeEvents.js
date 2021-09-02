// USAGE:
//   ENDPOINT=ws://127.0.0.1:9944 node filterBridgeEvents.js
require('dotenv').config();

const { ApiPromise, WsProvider } = require('@polkadot/api');
const phala_typedefs = require('@phala/typedefs').phalaDev;

const getTransferEvents = async (api, blockHash) => {
    return (await api.query.chainBridge.bridgeEvents.at(blockHash)).toJSON();
}

const main = async () => {
    const api = await ApiPromise.create({
        provider: new WsProvider(process.env.ENDPOINT),
        types: {...phala_typedefs, ...{
            BridgeEvent: {
                _enum: {
                    FungibleTransfer: "(BridgeChainId, DepositNonce, ResourceId, U256, Vec<u8>)",
                    NonFungibleTransfer: "(BridgeChainId, DepositNonce, ResourceId, Vec<u8>, Vec<u8>, Vec<u8>)",
                    GenericTransfer: "(BridgeChainId, DepositNonce, ResourceId, Vec<u8>)"
                }
            }
        }}
    });

    const unsubscribe = await api.rpc.chain.subscribeFinalizedHeads(async (header) => {
        console.log(`Bridge Transfer within block[${header.number}]: ${JSON.stringify(await getTransferEvents(api, header.hash), null, 4)}`)
    });

    const exitHandler = (options, exitCode) => {
        unsubscribe();
        if (options.cleanup) console.log('do cleanup');
        if (exitCode || exitCode === 0) console.log(exitCode);
        if (options.exit) process.exit();
    }

    process.stdin.resume();//so the program will not close instantly

    // Following copy from: https://stackoverflow.com/questions/14031763/doing-a-cleanup-action-just-before-node-js-exits/14032965#14032965
    // do something when app is closing
    process.on('exit', exitHandler.bind(null,{cleanup:true}));
    //catches ctrl+c event
    process.on('SIGINT', exitHandler.bind(null, {exit:true}));
    // catches "kill pid" (for example: nodemon restart)
    process.on('SIGUSR1', exitHandler.bind(null, {exit:true}));
    process.on('SIGUSR2', exitHandler.bind(null, {exit:true}));
    //catches uncaught exceptions
    process.on('uncaughtException', exitHandler.bind(null, {exit:true}));
}

main();
// main().catch(console.error).finally(() => process.exit())