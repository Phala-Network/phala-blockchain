require('dotenv').config();

const { ApiPromise, Keyring, WsProvider } = require('@polkadot/api');
const BN = require('bn.js');
const { checkUntil } = require('../../e2e/utils');

const bnUnit = new BN(1e12);
function token(n) {
    return new BN(n).mul(bnUnit);
}

async function main () {
    const wsProvider = new WsProvider(process.env.ENDPOINT);
    const api = await ApiPromise.create({ provider: wsProvider });

    async function getNonce(address) {
        const info = await api.query.system.account(address);
        return info.nonce.toNumber();
    }
    async function waitTxAccepted(account, nonce) {
        await checkUntil(async () => {
            return await getNonce(account) == nonce + 1;
        });
    }

    console.log('// Prepare keypairs');
    const keyring = new Keyring({ type: 'sr25519' });
    const alice = keyring.addFromUri('//Alice');
    const aliceStash = keyring.addFromUri('//Alice//stash');
    const bob = keyring.addFromUri('//Bob');
    const bobStash = keyring.addFromUri('//Bob//stash');
    const root = alice;

    let nonceAlice = await getNonce(alice.address);
    let nonceAliceStash = await getNonce(aliceStash.address);
    let nonceBob = await getNonce(bob.address);
    let nonceBobStash = await getNonce(bobStash.address);

    console.log('// Add money to stashes');
    await api.tx.balances.transfer(aliceStash.address, token(1000)).signAndSend(alice, {nonce: nonceAlice++});
    await api.tx.balances.transfer(bobStash.address, token(1000)).signAndSend(bob, {nonce: nonceBob++});
    await waitTxAccepted(bob.address, nonceBob - 1);

    console.log('// Register Bob & Stash');
    await api.tx.phala.setStash(bob.address).signAndSend(bob, {nonce: nonceBob++});
    await api.tx.sudo.sudo(
        api.tx.phala.forceRegisterWorker(bobStash.address, 'fake_mid', 'fake_pubkey')
    ).signAndSend(root, {nonce: nonceAlice++});

    console.log('// Deposit stake');
    await api.tx.miningStaking.deposit(token(114)).signAndSend(aliceStash, {nonce: nonceAliceStash++});
    await api.tx.miningStaking.stake(aliceStash.address, new BN(19).mul(bnUnit)).signAndSend(aliceStash, {nonce: nonceAliceStash++});
    await api.tx.miningStaking.stake(bobStash.address, new BN(19).mul(bnUnit)).signAndSend(aliceStash, {nonce: nonceAliceStash++});
    await api.tx.miningStaking.deposit(token(514)).signAndSend(bobStash, {nonce: nonceBobStash++});
    await api.tx.miningStaking.stake(aliceStash.address, new BN(1).mul(bnUnit)).signAndSend(bobStash, {nonce: nonceBobStash++});
    await api.tx.miningStaking.stake(bobStash.address, new BN(2).mul(bnUnit)).signAndSend(bobStash, {nonce: nonceBobStash++});

    console.log('// Start two miners');
    await api.tx.phala.startMiningIntention().signAndSend(alice, {nonce: nonceAlice++});
    await api.tx.phala.startMiningIntention().signAndSend(bob, {nonce: nonceBob++});

    console.log('// Trigger next round');
    await api.tx.sudo.sudo(
        api.tx.miningStaking.forceTriggerRoundEnd()
    ).signAndSend(root, {nonce: nonceAlice++});
    await api.tx.sudo.sudo(
        api.tx.phala.forceNextRound()
    ).signAndSend(root, {nonce: nonceAlice++});
    await waitTxAccepted(alice.address, nonceAlice - 1);
}

main().catch(console.error).finally(() => process.exit());
