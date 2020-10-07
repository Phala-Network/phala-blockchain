const { assert } = require('chai');
const path = require('path');
const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');
const { cryptoWaitReady, mnemonicGenerate } = require('@polkadot/util-crypto');

const { Process, TempDir } = require('../pm');
const { PRuntime } = require('../pruntime');
const { checkUntil } = require('../utils');
const types = require('../typedefs.json');

const pathNode = path.resolve('../target/release/phala-node');
const pathRelayer = path.resolve('../target/release/phost');
const pathPRuntime = path.resolve('../pruntime/bin/app');

const EPS = 1e-8;

// TODO: Switch to [instant-seal-consensus](https://substrate.dev/recipes/kitchen-node.html) for faster test

describe('A full stack', function () {
	this.timeout(30000);
	let processNode;
	let processRelayer;
	let processPRuntime;

	let api, keyring, alice, root;
	const pruntime = new PRuntime();
	const tmpDir = new TempDir();
	const tmpPath = tmpDir.dir;

	before(async () => {
		processNode = new Process(pathNode, ['--dev', '--base-path=' + path.resolve(tmpPath, 'phala-node'), '--ws-port=9944']);
		processRelayer = new Process(pathRelayer, ['--dev']);
		processPRuntime = new Process(pathPRuntime, [], {
			cwd: path.dirname(pathPRuntime),
		});
		// launch nodes
		await Promise.all([
			processNode.startAndWaitForOutput(/Listening for new connections on 127\.0\.0\.1:9944/),
			processPRuntime.startAndWaitForOutput(/Rocket has launched from http:\/\/0\.0\.0\.0:8000/),
		]);
		await processRelayer.startAndWaitForOutput(/runtime_info:InitRuntimeResp/);
		// create polkadot api and keyring
		api = await ApiPromise.create({ provider: new WsProvider('ws://localhost:9944'), types });
		await cryptoWaitReady();
		keyring = new Keyring({ type: 'sr25519' });
		root = alice = keyring.addFromUri('//Alice');
	});

	it('should be up and running', async function () {
		assert.isFalse(processNode.stopped);
		assert.isFalse(processRelayer.stopped);
		assert.isFalse(processPRuntime.stopped);
	});

	describe('PhalaNode', () => {
		it('should have Alice registered', async function () {

		});
	})

	describe('PRuntime', () => {
		it('is initialized', async function () {
			assert.isTrue(await checkUntil(async () => {
				const info = await pruntime.getInfo();
				return info.initialized;
			}, 1000), 'not initialized in time');
		});

		it('can sync block', async function() {
			assert.isTrue(await checkUntil(async () => {
				const info = await pruntime.getInfo();
				return info.blocknum > 0;
			}, 7000), 'stuck at block 0');
		});
	});

	describe('Miner workflow', () => {

		let stash, controller, reward;
		before(function () {
			stash = keyring.addFromUri(mnemonicGenerate());
			controller = keyring.addFromUri(mnemonicGenerate());
			reward = keyring.addFromUri(mnemonicGenerate());
		});

		async function getNonce(address) {
			const info = await api.query.system.account(address);
			return info.nonce.toNumber();
		}
		async function waitTxAccepted(account, nonce) {
			await checkUntil(async () => {
				return await getNonce(account) == nonce + 1;
			});
		}

		it('can fund the accounts', async function () {
			const unit10 = 10 * 1e12;
			const nonce = await getNonce(alice.address)
			await api.tx.balances.transfer(stash.address, unit10).signAndSend(alice, {nonce: nonce});
			await api.tx.balances.transfer(controller.address, unit10).signAndSend(alice, {nonce: nonce + 1});
			await api.tx.balances.transfer(reward.address, unit10).signAndSend(alice, {nonce: nonce + 2});

			await waitTxAccepted(alice.address, nonce + 2);
			const stashInfo = await api.query.system.account(stash.address);
			const controllerInfo = await api.query.system.account(controller.address);
			assert.equal(stashInfo.data.free.toNumber(), unit10);
			assert.equal(controllerInfo.data.free.toNumber(), unit10);
		});

		it('can create a stash', async function () {
			const nonce = await getNonce(stash.address);
			await api.tx.phalaModule.setStash(controller.address)
				.signAndSend(stash, { nonce });

			await waitTxAccepted(stash.address, nonce);
			assert.equal(
				await api.query.phalaModule.stash(controller.address),
				stash.address,
				'controller stash mismatch');
			assert.equal(
				(await api.query.phalaModule.stashState(stash.address)).controller,
				controller.address,
				'controller stash mismatch');
		});

		it('can set payout preference', async function () {
			// commission: 50%, target: reward address
			await assertSuccess(
				api.tx.phalaModule.setPayoutPrefs(50, reward.address),
				controller);

			const stashInfo = await api.query.phalaModule.stashState(stash.address);
			assert.equal(stashInfo.payoutPrefs.commission.toNumber(), 50, 'failed to set commission');
			assert.equal(stashInfo.payoutPrefs.target, reward.address, 'failed to set payout target');
		});

		it('can force register a worker', async function () {
			const pubkey = '0x0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798';
			await assertSuccess(
				api.tx.sudo.sudo(
					api.tx.phalaModule.forceRegisterWorker(stash.address, 'fake_mid', pubkey)),
				root);

			const workerInfo = await api.query.phalaModule.workerState(stash.address);
			assert.equal(workerInfo.status.toNumber(), 0);
		});

		it('can start mining', async function () {
			await assertSuccess(
				api.tx.phalaModule.startMine(),
				controller);

			const { status } = await api.query.phalaModule.workerState(stash.address);
			assert.equal(status.toNumber(), 1);
		});

		it('can stop mining', async function () {
			await assertSuccess(
				api.tx.phalaModule.stopMine(),
				controller);

			const { status } = await api.query.phalaModule.workerState(stash.address);
			assert.equal(status.toNumber(), 0);
		});
	})


	after(async function () {
		await api.disconnect();
		await Promise.all([
			processNode.kill(),
			processPRuntime.kill(),
			processRelayer.kill(),
		]);
		tmpDir.cleanup();
	});

});

async function assertSuccess(txBuilder, signer) {
	return await new Promise(async (resolve, _reject) => {
		const unsub = await txBuilder.signAndSend(signer, (result) => {
			if (result.status.isInBlock) {
				let error;
				for (const e of result.events) {
					const { event: { data, method, section } } = e;
					if (section === 'system' && method === 'ExtrinsicFailed') {
						error = data[0];
					}
				}
				if (error) {
					assert.fail(`Extrinsic failed with error: ${error}`);
				}
				unsub();
				resolve(result.status.asInBlock);
			} else if (result.status.isInvalid) {
				assert.fail('Invalid transaction');
				unsub();
				resolve();
			}
		});
	});
}
