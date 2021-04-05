// USAGE:
//   ACALA_ENDPOINT=wss://pc-test.phala.network/parachains/acala/ws FROM=//Alice TO=//Alice AMOUNT=666 node transferFromAcala.js
require('dotenv').config();

const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');
const { options } = require('@acala-network/api');
const { types } = require('@acala-network/types');
const { cryptoWaitReady } = require('@polkadot/util-crypto');
const fs = require('fs');
const BN = require('bn.js');

const bn1e13 = new BN(10).pow(new BN(13));
const phalaParaId = 30;
const acalaParaId = 666;

const main = async () => {
    const wsProvider = new WsProvider(process.env.ACALA_ENDPOINT);
    const api = await ApiPromise.create(options({ provider: wsProvider, types: types}));
    
    const acalaAccount = process.env.FROM || '//Alice';
    const phalaAccount = process.env.TO || '//Alice';
    const amount = new BN(process.env.AMOUNT);

    await cryptoWaitReady();

    const keyring = new Keyring({ type: 'sr25519' });
    const sender = keyring.addFromUri(acalaAccount);
    const receiver = keyring.addFromUri(phalaAccount);
    let nonce = (await api.query.system.account(sender.address)).nonce.toNumber();

    const transferToPhala = () => {
        return new Promise(async resolve => {
            const unsub = await api.tx.xTokens.transferToParachain(
                api.createType('XCurrencyId', {
                    chainId: api.createType('ChainId', {
                        Parachain: api.createType('Compact<U32>', acalaParaId),
                    }),
                    currencyId: "ACA"
                }),
                api.createType('Compact<U32>', phalaParaId),
                api.createType('MultiLocation', {
                    X1: api.createType('Junction', {
                        AccountId32: api.createType('AccountId32Junction', {
                            network: api.createType('NetworkId', 'Any'),
                            id: receiver.address,
                        })
                    })
                }),
                api.createType('Compact<U128>', bn1e13.mul(amount)),
            )
            .signAndSend(sender, {nonce: nonce, era: 0}, (result) => {
                console.log(`Current status is ${result.status}`);
                if (result.status.isInBlock) {
                    console.log(`Transaction included at blockHash ${result.status.asInBlock}`);
                } else if (result.status.isFinalized) {
                    console.log(`Transaction finalized at blockHash ${result.status.asFinalized}`);
                    unsub();
                    resolve();
                }
            });
        });
    }

    await transferToPhala();
    console.log('--- Transfer from Acala to Phala ---');
    console.log(`---   From: ${acalaAccount}`);
    console.log(`---     To: ${phalaAccount}`);
    console.log(`--- Amount: ${amount.toString()}`);
}

main().catch(console.error).finally(() => process.exit())