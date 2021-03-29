// USAGE:
//   ENDPOINT=wss://rococo-rpc.polkadot.io PARAID=30 IN=/tmp/phala-prticipants.json node dumpCrowdloan.js
require('dotenv').config();

 const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');
 const { cryptoWaitReady } = require('@polkadot/util-crypto');
 const fs = require('fs');
 
 const main = async () => { 
     const wsProvider = new WsProvider(process.env.ENDPOINT);
     const api = await ApiPromise.create({ provider: wsProvider });
     const paraId = parseInt(process.env.PARAID || 30);
     const participantsPath = process.env.IN;

     const participantsJson = fs.readFileSync(participantsPath, {encoding: 'utf-8'});
     const participants = JSON.parse(participantsJson);
     
     await cryptoWaitReady();
 
     const keyring = new Keyring({ type: 'sr25519' });
     const sender = keyring.addFromUri(process.env.PRIVKEY);
     let nonce = (await api.query.system.account(sender.address)).nonce.toNumber();

     const promises = [];
     for (const {from} of participants) {
        promises.push(
            api.tx.crowdloan.withdraw(from, paraId).signAndSend(sender, {nonce: nonce++})
        );
     }

    const r = await Promise.all(promises);
     console.log('Withdraw sent:', r);
 }
 
 main().catch(console.error).finally(() => process.exit())
 