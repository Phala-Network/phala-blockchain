const { assert } = require('chai');
const path = require('path');
const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');
const { cryptoWaitReady, mnemonicGenerate } = require('@polkadot/util-crypto');

const { Process, TempDir } = require('../pm');
const { PRuntime } = require('../pruntime');
const { checkUntil } = require('../utils');

const pathNode = path.resolve('../target/release/phala-node');
const pathRelayer = path.resolve('../target/release/phost');
const pathPRuntime = path.resolve('../pruntime/bin/app');

const EPS = 1e-8;

describe('A full stack', function () {
	this.timeout(10000);
	let processNode;
	let processRelayer;
	let processPRuntime;

	let api, keyring, alice, bob;
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
		api = await ApiPromise.create({ provider: new WsProvider('ws://localhost:9944') });
		await cryptoWaitReady();
		keyring = new Keyring({ type: 'sr25519' });
		alice = keyring.addFromUri('//Alice');
		bob = keyring.addFromUri('//Bob');
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

		let stash, controller;
		before(function () {
			stash = keyring.addFromUri(mnemonicGenerate());
			controller = keyring.addFromUri(mnemonicGenerate());
		});

		async function getNonce(address) {
			const info = await api.query.system.account(address);
			return info.nonce.toNumber();
		}

		it('can fund the accounts', async function () {
			const unit10 = 10 * 1e12;
			const nonce = await getNonce(alice.address)
			await api.tx.balances.transfer(stash.address, unit10).signAndSend(alice, {nonce: nonce});
			await api.tx.balances.transfer(controller.address, unit10).signAndSend(alice, {nonce: nonce + 1});

			// wait until all block get included
			await checkUntil(async () => {
				return await getNonce(alice.address) == nonce + 2;
			});

			const stashInfo = await api.query.system.account(stash.address);
			const controllerInfo = await api.query.system.account(controller.address);
			assert.equal(stashInfo.data.free.toNumber(), unit10);
			assert.equal(controllerInfo.data.free.toNumber(), unit10);
		});

		it('can create a stash', async function () {

		});

		it('can set payout preference', async function () {
		});

		it('can force register a worker', async function () {
		});

		it('can start mining', async function () {
		});

		it('can stop mining', async function () {
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
