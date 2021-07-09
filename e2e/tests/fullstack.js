require('dotenv').config();
const { assert } = require('chai');
const path = require('path');
const portfinder = require('portfinder');
const fs = require('fs');
const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');
const { cryptoWaitReady, mnemonicGenerate } = require('@polkadot/util-crypto');

const types = require('@phala/typedefs').latest;

const { Process, TempDir } = require('../pm');
const { PRuntimeApi } = require('../pruntime');
const { checkUntil } = require('../utils');

const pathNode = path.resolve('../target/release/phala-node');
const pathRelayer = path.resolve('../target/release/phost');
const pathPRuntime = path.resolve('../standalone/pruntime/bin/app');

// TODO: Switch to [instant-seal-consensus](https://substrate.dev/recipes/kitchen-node.html) for faster test

describe('A full stack', function () {
	this.timeout(60000);
	let processNode;
	let processRelayer;
	let processPRuntime;

	let api, keyring, alice, root;
	let pruntime;
	const tmpDir = new TempDir();
	const tmpPath = tmpDir.dir;

	before(async () => {
		// Check binary files
		[pathNode, pathRelayer, pathPRuntime].map(fs.accessSync);
		// Node cli
		const wsPort = await portfinder.getPortPromise({port: 9944});
		const teePort = await portfinder.getPortPromise({port: 8000, stopPort: 9900});
		processNode = new Process([
			pathNode, [
				'--dev',
				'--base-path=' + path.resolve(tmpPath, 'phala-node'),
				`--ws-port=${wsPort}`,
				'--rpc-methods=Unsafe'
			]
		], { logPath: tmpPath + '/node.log' });
		processRelayer = new Process([
			pathRelayer, [
				'--dev',
				`--substrate-ws-endpoint=ws://localhost:${wsPort}`,
				`--pruntime-endpoint=http://localhost:${teePort}`
			]
		], { logPath: tmpPath + '/relayer.log' });
		processPRuntime = new Process([
			pathPRuntime, [
				'--cores=0',	// Disable benchmark
			], {
				cwd: path.dirname(pathPRuntime),
				env: {
					...process.env,
					ROCKET_PORT: teePort.toString(),
				}
			}
		], { logPath: tmpPath + '/pruntime.log' });
		// processRelayer.debug = true;
		// processPRuntime.debug = true;
		// Launch nodes
		await Promise.all([
			processNode.startAndWaitForOutput(/Listening for new connections on 127\.0\.0\.1:(\d+)/),
			processPRuntime.startAndWaitForOutput(/Rocket has launched from http:\/\/0\.0\.0\.0:(\d+)/),
		]);
		await processRelayer.startAndWaitForOutput(/runtime_info:Some\(InitRuntimeResp/);
		// Create polkadot api and keyring
		api = await ApiPromise.create({ provider: new WsProvider(`ws://localhost:${wsPort}`), types });
		await cryptoWaitReady();
		keyring = new Keyring({ type: 'sr25519' });
		root = alice = keyring.addFromUri('//Alice');
		// Create pRuntime API
		pruntime = new PRuntimeApi(`http://localhost:${teePort}`);
	});

	after(async function () {
		if (api) await api.disconnect();
		await Promise.all([
			processNode.kill(),
			processPRuntime.kill(),
			processRelayer.kill(),
		]);
		if (process.env.KEEP_TEST_FILES != '1') {
			tmpDir.cleanup();
		}
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
				api.tx.phalaModule.startMiningIntention(),
				controller);
			// Get into the next mining round
			const { events } = await assertSuccess(
				api.tx.sudo.sudo(api.tx.phalaModule.dbgNextRound()),
				root);

			assertEvents(events, [
				['sudo', 'Sudid'],
				['system', 'ExtrinsicSuccess']
			]);

			const { status } = await api.query.phalaModule.workerState(stash.address);
			assert.equal(status.toNumber(), 1);
			// MiningState
			const miningStatus = await api.query.phalaModule.miningState(stash.address);
			assert.isTrue(miningStatus.isMining.valueOf());
			assert.isTrue(miningStatus.startBlock.isSome);
		});

		it('can stop mining', async function () {
			await assertSuccess(
				api.tx.phalaModule.stopMiningIntention(),
				controller);
			const { events } = await assertSuccess(
				api.tx.sudo.sudo(api.tx.phalaModule.dbgNextRound()),
				root);

			assertEvents(events, [
				['phalaModule', 'GotCredits', [undefined, 200, 200]],
				['sudo', 'Sudid'],
				['system', 'ExtrinsicSuccess'],
			], true);

			const { status } = await api.query.phalaModule.workerState(stash.address);
			assert.equal(status.toNumber(), 0);
			// MiningState
			const miningStatus = await api.query.phalaModule.miningState(stash.address);
			assert.isFalse(miningStatus.isMining.valueOf());
			assert.isTrue(miningStatus.startBlock.isNone);
		});
	})

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
				resolve({
					hash: result.status.asInBlock,
					events: result.events,
				});
			} else if (result.status.isInvalid) {
				assert.fail('Invalid transaction');
				unsub();
				resolve();
			}
		});
	});
}

function fillPartialArray(obj, pattern) {
	for (const [idx, v] of obj.entries()) {
		if (pattern[idx] === undefined) {
			pattern[idx] = v;
		} else if (v instanceof Array && pattern[idx] instanceof Array) {
			fillPartialArray(v, pattern[idx]);
		}
	}
}

function assertEvents(actualEvents, expectedEvents, partial = true) {
	const simpleEvents = simplifyEvents(actualEvents, true);
	if (partial) {
		fillPartialArray(simpleEvents, expectedEvents);
	}
	assert.deepEqual(simpleEvents, expectedEvents, 'Events not equal');
}

function simplifyEvents (events, keepData = false) {
	const simpleEvents = [];
	for (const e of events) {
		const { event } = e;
		const { method, section } = event;
		if (keepData) {
			simpleEvents.push([section, method, event.toJSON().data]);
		} else {
			simpleEvents.push([section, method]);
		}
	}
	return simpleEvents;
}
