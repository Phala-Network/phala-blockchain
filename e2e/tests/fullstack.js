const { assert } = require('chai');
const path = require('path');

const { Process, TempDir } = require('../pm');
const { PRuntime } = require('../pruntime');
const { checkUntil } = require('../utils');

const pathNode = path.resolve('../target/release/phala-node');
const pathRelayer = path.resolve('../target/release/phost');
const pathPRuntime = path.resolve('../pruntime/bin/app');


describe('A full stack', function () {
	this.timeout(10000);
	let processNode;
	let processRelayer;
	let processPRuntime;

	const tmpDir = new TempDir();
	const pruntime = new PRuntime();
	// const api = new PolkadotJsApi...
	const tmpPath = tmpDir.dir;
	console.log(tmpPath);

	before(async () => {
		processNode = new Process(pathNode, ['--dev', '--base-path=' + path.resolve(tmpPath, 'phala-node')]);
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
	});

	it('should be up and running', async function () {
		assert.isFalse(processNode.stopped);
		assert.isFalse(processRelayer.stopped);
		assert.isFalse(processPRuntime.stopped);
	});

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
		it('can fund the controller account', async function () {

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
		await Promise.all([
			processNode.kill(),
			processPRuntime.kill(),
			processRelayer.kill(),
		]);
		tmpDir.cleanup();
	});

});
