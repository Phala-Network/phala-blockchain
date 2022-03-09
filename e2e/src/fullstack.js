require('dotenv').config();
const { assert } = require('chai');
const path = require('path');
const portfinder = require('portfinder');
const fs = require('fs');
const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');
const { cryptoWaitReady, mnemonicGenerate } = require('@polkadot/util-crypto');

const { types, typeAlias } = require('./utils/typeoverride');
// TODO: fixit
// const types = require('@phala/typedefs').phalaDev;

const { Process, TempDir } = require('./utils/pm');
const { PRuntimeApi } = require('./utils/pruntime');
const { checkUntil, skipSlowTest } = require('./utils');

const pathNode = path.resolve('../target/release/phala-node');
const pathRelayer = path.resolve('../target/release/pherry');
const pathPRuntime = path.resolve('../standalone/pruntime/bin/pruntime');

// TODO: Switch to [instant-seal-consensus](https://substrate.dev/recipes/kitchen-node.html) for faster test

describe('A full stack', function () {
    this.timeout(60000);

    let cluster;
    let api, keyring, alice, root;
    let pruntime;
    const tmpDir = new TempDir();
    const tmpPath = tmpDir.dir;

    before(async () => {
        // Check binary files
        [pathNode, pathRelayer, pathPRuntime].map(fs.accessSync);
        // Bring up a cluster
        cluster = new Cluster(2, pathNode, pathRelayer, pathPRuntime, tmpPath);
        await cluster.start();
        // APIs
        api = await cluster.api;
        pruntime = cluster.workers.map(w => w.api);
        // Create polkadot api and keyring
        await cryptoWaitReady();
        keyring = new Keyring({ type: 'sr25519', ss58Format: 30 });
        root = alice = keyring.addFromUri('//Alice');
    });

    after(async function () {
        // TODO: consider handle the signals and process.on('exit') event:
        //   https://stackoverflow.com/questions/14031763/doing-a-cleanup-action-just-before-node-js-exits
        if (api) await api.disconnect();
        await cluster.kill();
        if (process.env.KEEP_TEST_FILES != '1') {
            tmpDir.cleanup();
        } else {
            console.log(`The test datadir is kept at ${cluster.tmpPath}`);
        }
    });

    it('should be up and running', async function () {
        assert.isFalse(cluster.processNode.stopped);
        for (const w of cluster.workers) {
            assert.isFalse(w.processRelayer.stopped);
            assert.isFalse(w.processPRuntime.stopped);
        }
    });

    let workerKey;
    describe('pRuntime', () => {
        it('is initialized', async function () {
            let info;
            assert.isTrue(await checkUntil(async () => {
                info = await pruntime[0].getInfo();
                return info.initialized;
            }, 1000), 'not initialized in time');
            // A bit guly. Any better way?
            workerKey = Uint8Array.from(Buffer.from(info.publicKey, 'hex'));
        });

        it('can sync block', async function () {
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[0].getInfo();
                return info.blocknum > 0;
            }, 7000), 'stuck at block 0');
        });

        it('is registered', async function () {
            if (skipSlowTest()) {
                this.skip();
            }
            // Finalization takes 2-3 blocks. So we wait for 3 blocks here.
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[0].getInfo();
                return info.registered;
            }, 4 * 6000), 'not registered in time');
        });

        it('finishes the benchmark', async function () {
            if (skipSlowTest()) {
                this.skip();
            }
            assert.isTrue(await checkUntil(async () => {
                const workerInfo = await api.query.phalaRegistry.workers(workerKey);
                return workerInfo.unwrap().initialScore.isSome;
            }, 3 * 6000), 'benchmark timeout');
        });
    });

    describe('Gatekeeper', () => {
        it('pre-mines blocks', async function () {
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[0].getInfo();
                return info.blocknum > 10;
            }, 10 * 6000), 'not enough blocks mined');
        });

        it('can be registered as first gatekeeper', async function () {
            // Register worker1 as Gatekeeper
            const info = await pruntime[0].getInfo();
            await assert.txAccepted(
                api.tx.sudo.sudo(
                    api.tx.phalaRegistry.forceRegisterWorker(
                        hex(info.publicKey),
                        hex(info.ecdhPublicKey),
                        null,
                    )
                ),
                alice,
            );
            await assert.txAccepted(
                api.tx.sudo.sudo(
                    api.tx.phalaRegistry.registerGatekeeper(hex(info.publicKey))
                ),
                alice,
            );
            // Finalization takes 2-3 blocks. So we wait for 3 blocks here.
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[0].getInfo();
                return info.registered;
            }, 4 * 6000), 'not registered in time');

            // Check if the role is Gatekeeper
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[0].getInfo();
                const gatekeepers = await api.query.phalaRegistry.gatekeeper();
                // console.log(`Gatekeepers after registeration: ${gatekeepers}`);
                return gatekeepers.includes(hex(info.publicKey));
            }, 4 * 6000), 'not registered as gatekeeper');
        });

        it('finishes master pubkey upload', async function () {
            assert.isTrue(await checkUntil(async () => {
                const masterPubkey = await api.query.phalaRegistry.gatekeeperMasterPubkey();
                return masterPubkey.isSome;
            }, 4 * 6000), 'master pubkey not uploaded');
        });
    });

    describe.skip('Gatekeeper2', () => {
        it('can be registered', async function () {
            // Register worker1 as Gatekeeper
            const info = await pruntime[1].getInfo();
            await assert.txAccepted(
                api.tx.sudo.sudo(
                    api.tx.phalaRegistry.forceRegisterWorker(
                        hex(info.publicKey),
                        hex(info.ecdhPublicKey),
                        null,
                    )
                ),
                alice,
            );
            await assert.txAccepted(
                api.tx.sudo.sudo(
                    api.tx.phalaRegistry.registerGatekeeper(hex(info.publicKey))
                ),
                alice,
            );
            // Finalization takes 2-3 blocks. So we wait for 3 blocks here.
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[1].getInfo();
                return info.registered;
            }, 4 * 6000), 'not registered in time');

            // Check if the role is Gatekeeper
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[1].getInfo();
                const gatekeepers = await api.query.phalaRegistry.gatekeeper();
                // console.log(`Gatekeepers after registeration: ${gatekeepers}`);
                return gatekeepers.includes(hex(info.publicKey));
            }, 4 * 6000), 'not registered as gatekeeper');
        });

        it('can receive master key', async function () {
            // Wait for the successful dispatch of master key
            // pRuntime[1] should be down
            assert.isTrue(
                await cluster.waitWorkerExitAndRestart(1, 10 * 6000),
                'worker1 restart timeout'
            );
            assert.isTrue(
                fs.existsSync(`${tmpPath}/pruntime1/master_key.seal`),
                'master key not received'
            );
        });

        it('becomes active', async function () {
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[1].getInfo();
                return info.gatekeeper.role == 2;  // 2: GatekeeperRole.Active in protobuf
            }, 1000))

            // Step 3: wait a few more blocks and ensure there are no conflicts in gatekeepers' shared mq
        });

        it('post-mines blocks', async function () {
            const gatekeeper = api.createType('MessageOrigin', 'Gatekeeper');
            let seqStart = await api.query.phalaMq.offchainIngress(gatekeeper);
            seqStart = seqStart.unwrap().toNumber();
            assert.isTrue(await checkUntil(async () => {
                let seq = await api.query.phalaMq.offchainIngress(gatekeeper);
                seq = seq.unwrap().toNumber();
                return seq >= seqStart + 1;
            }, 500 * 6000), 'ingress stale');
        });
    });

    describe('Cluster & Contract', () => {
        let wasmFile = './res/flipper.wasm';
        let codeHash = hex('0x5b1f7f0b85908c168c0d0ca65efae0e916455bf527b8589d3e655311ec7b1f2c');
        let initSelector = hex('0xed4b9d1b'); // for default() function
        let clusterId;

        it('can create cluster', async function () {
            const perm = api.createType('ClusterPermission', 'Public');
            const info = await pruntime[0].getInfo();
            const { events } = await assert.txAccepted(
                api.tx.phalaFatContracts.addCluster(perm, [hex(info.publicKey)]),
                alice,
            );
            assertEvents(events, [
                ['balances', 'Withdraw'],
                ['phalaFatContracts', 'ClusterCreated']
            ]);

            const { event } = events[1];
            clusterId = hex(event.toJSON().data[0]);
            const clusterInfo = await api.query.phalaFatContracts.clusters(clusterId);
            assert.isTrue(clusterInfo.isSome);
        });

        it('can generate cluster key', async function () {
            assert.isTrue(await checkUntil(async () => {
                const clusterKey = await api.query.phalaRegistry.clusterKeys(clusterId);
                return clusterKey.isSome;
            }, 4 * 6000), 'cluster pubkey not uploaded');
        });

        it('can deploy cluster to worker', async function () {
            assert.isTrue(await checkUntil(async () => {
                const clusterWorkers = await api.query.phalaFatContracts.clusterWorkers(clusterId);
                return clusterWorkers.length == 1;
            }, 4 * 6000), 'cluster not deployed');
        });

        it('can upload code', async function () {
            let code = fs.readFileSync(wasmFile, 'hex');
            await assert.txAccepted(
                api.tx.phalaFatContracts.uploadCodeToCluster(hex(code), clusterId),
                alice,
            );
            // For now, there is no way to check whether code is uploaded in script
            // since this requires monitering the async CodeUploaded event
        });

        it('can instantiate contract', async function () {
            const codeIndex = api.createType('CodeIndex', { 'WasmCode': codeHash });
            const { events } = await assert.txAccepted(
                api.tx.phalaFatContracts.instantiateContract(codeIndex, initSelector, 0, clusterId),
                alice,
            );
            assertEvents(events, [
                ['balances', 'Withdraw'],
                ['phalaFatContracts', 'Instantiating']
            ]);

            const { event } = events[1];
            let contractId = hex(event.toJSON().data[0]);
            let contractInfo = await api.query.phalaFatContracts.contracts(contractId);
            assert.isTrue(contractInfo.isSome, 'no contract info');

            assert.isTrue(await checkUntil(async () => {
                let key = await api.query.phalaRegistry.contractKeys(contractId);
                return key.isSome;
            }, 4 * 6000), 'contract key generation failed');

            assert.isTrue(await checkUntil(async () => {
                let clusterContracts = await api.query.phalaFatContracts.clusterContracts(clusterId);
                return clusterContracts.length == 1;
            }, 4 * 6000), 'instantiation failed');
        });

        it('cannot dup-instantiate', async function () {
            const codeIndex = api.createType('CodeIndex', { 'WasmCode': codeHash });
            await assert.txFailed(
                api.tx.phalaFatContracts.instantiateContract(codeIndex, initSelector, 0, clusterId),
                alice,
            );
        });
    });

    describe.skip('Solo mining workflow', () => {
        let miner;
        before(function () {
            miner = keyring.addFromUri(mnemonicGenerate());
        })

        // worker registered

        it.skip('can bind the worker', async function () {
            const workerInfo = await api.query.phalaRegistry.worker(workerKey);
            const operator = workerInfo.unwrap().operator.unwrap();
            assert.equal(operator.toHuman(), alice.address, 'bad operator');

            await assert.txAccepted(api.tx.phalaMining.bind(workerKey), alice);

            let actualWorker = await api.query.phalaMining.minerBinding(alice.address);
            let actualMiner = await api.query.phalaMining.workerBinding(workerKey);

            actualWorker = Uint8Array.from(actualWorker.unwrap());
            actualMiner = actualMiner.unwrap();
            assert.deepEqual(actualWorker, workerKey, 'wrong bounded worker');
            assert.equal(actualMiner.toHuman(), alice.address, 'wrong bounded miner');

            let minerInfo = await api.query.phalaMining.miners(alice.address);
            assert.isTrue(minerInfo.isSome, 'miner entity should exist');

            minerInfo = minerInfo.unwrap();
            assert.isTrue(
                minerInfo.state.isReady,
                'miner bounded to worker must be in Ready state'
            );
        })

        it('can deposite some stake')

        it.skip('can start mining', async function () {
            await assert.txAccepted(api.tx.phalaMining.startMining(), alice);
            let minerInfo = await api.query.phalaMining.miners(alice.address);
            minerInfo = minerInfo.unwrap();
            assert.isTrue(minerInfo.state.isIdle, 'miner state should be MiningIdle');
        })

        it('triggers a heartbeat and wait for payout')
        it('goes to MiningUnresponsive state if offline')
        it('can stop mining')
        it('settles after the cooling down period')
    })

    describe.skip('Miner workflow', () => {

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
            await api.tx.balances.transfer(stash.address, unit10).signAndSend(alice, { nonce: nonce });
            await api.tx.balances.transfer(controller.address, unit10).signAndSend(alice, { nonce: nonce + 1 });
            await api.tx.balances.transfer(reward.address, unit10).signAndSend(alice, { nonce: nonce + 2 });

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
            await assert.txAccepted(
                api.tx.phalaModule.setPayoutPrefs(50, reward.address),
                controller);

            const stashInfo = await api.query.phalaModule.stashState(stash.address);
            assert.equal(stashInfo.payoutPrefs.commission.toNumber(), 50, 'failed to set commission');
            assert.equal(stashInfo.payoutPrefs.target, reward.address, 'failed to set payout target');
        });

        it('can force register a worker', async function () {
            const pubkey = '0x0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798';
            await assert.txAccepted(
                api.tx.sudo.sudo(
                    api.tx.phalaModule.forceRegisterWorker(stash.address, 'fake_mid', pubkey)),
                root);

            const workerInfo = await api.query.phalaModule.workerState(stash.address);
            assert.equal(workerInfo.status.toNumber(), 0);
        });

        it('can start mining', async function () {
            await assert.txAccepted(
                api.tx.phalaModule.startMiningIntention(),
                controller);
            // Get into the next mining round
            const { events } = await assert.txAccepted(
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
            await assert.txAccepted(
                api.tx.phalaModule.stopMiningIntention(),
                controller);
            const { events } = await assert.txAccepted(
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

async function assertSubmission(txBuilder, signer, shouldSucceed = true) {
    return await new Promise(async (resolve, _reject) => {
        const unsub = await txBuilder.signAndSend(signer, (result) => {
            if (result.status.isInBlock) {
                let error;
                for (const e of result.events) {
                    const { event: { data, method, section } } = e;
                    if (section === 'system' && method === 'ExtrinsicFailed') {
                        if (shouldSucceed) {
                            error = data[0];
                        } else {
                            unsub();
                            resolve(error);
                        }
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
assert.txAccepted = assertSubmission;
assert.txFailed = (txBuilder, signer) => assertSubmission(txBuilder, signer, false);

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

function simplifyEvents(events, keepData = false) {
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

class Cluster {
    constructor(numWorkers, pathNode, pathRelayer, pathPRuntime, tmpPath) {
        this.numWorkers = numWorkers;
        this.pathNode = pathNode;
        this.pathRelayer = pathRelayer;
        this.pathPRuntime = pathPRuntime;
        this.tmpPath = tmpPath;
        [pathNode, pathRelayer, pathPRuntime].map(fs.accessSync);
        // Prepare empty workers
        const workers = [];
        for (let i = 0; i < this.numWorkers; i++) {
            workers.push({});
        }
        this.workers = workers;
    }

    async start() {
        await this._reservePorts();
        this._createProcesses();
        await this._launchAndWait();
        await this._createApi();
    }

    async kill() {
        await Promise.all([
            this.processNode.kill(),
            ...this.workers.map(w => [
                w.processPRuntime.kill('SIGKILL'),
                w.processRelayer.kill()
            ]).flat()
        ]);
    }

    // Returns false if waiting is timeout; otherwise it restart the specified worker
    async waitWorkerExitAndRestart(i, timeout) {
        const w = this.workers[i];
        const succeed = await checkUntil(async () => {
            return w.processPRuntime.stopped && w.processRelayer.stopped
        }, timeout);
        if (!succeed) {
            return false;
        }
        this._createWorkerProcess(i);
        await waitPRuntimeOutput(w.processPRuntime);
        await waitRelayerOutput(w.processRelayer);
        return true;
    }

    async _reservePorts() {
        const [wsPort, ...workerPorts] = await Promise.all([
            portfinder.getPortPromise({ port: 9944 }),
            ...this.workers.map(() => portfinder.getPortPromise({ port: 8000, stopPort: 9900 }))
        ]);
        this.wsPort = wsPort;
        this.workers.forEach((w, i) => w.port = workerPorts[i]);
    }

    _createProcesses() {
        this.processNode = newNode(this.wsPort, this.tmpPath, 'node');
        this.workers.forEach((_, i) => {
            this._createWorkerProcess(i);
        })
        this.processes = [
            this.processNode,
            ...this.workers.map(w => [w.processRelayer, w.processPRuntime]).flat()
        ];
    }

    _createWorkerProcess(i) {
        const AVAILBLE_ACCOUNTS = [
            '//Alice',
            '//Bob',
        ];
        const w = this.workers[i];
        const gasAccountKey = AVAILBLE_ACCOUNTS[i];
        const key = '0'.repeat(63) + (i + 1).toString();
        w.processRelayer = newRelayer(this.wsPort, w.port, this.tmpPath, gasAccountKey, key, `relayer${i}`);
        w.processPRuntime = newPRuntime(w.port, this.tmpPath, `pruntime${i}`);
    }

    async _launchAndWait() {
        // Launch nodes & pruntime
        await Promise.all([
            waitNodeOutput(this.processNode),
            ...this.workers.map(w => waitPRuntimeOutput(w.processPRuntime)),
        ]);
        // Launch relayers
        await Promise.all(this.workers.map(w => waitRelayerOutput(w.processRelayer)));
    }

    async _createApi() {
        this.api = await ApiPromise.create({
            provider: new WsProvider(`ws://localhost:${this.wsPort}`),
            types, typeAlias
        });
        this.workers.forEach(w => {
            w.api = new PRuntimeApi(`http://localhost:${w.port}`);
        })
    }

}

function waitPRuntimeOutput(p) {
    return p.startAndWaitForOutput(/Rocket has launched from http:\/\/0\.0\.0\.0:(\d+)/);
}
function waitRelayerOutput(p) {
    return p.startAndWaitForOutput(/runtime_info: InitRuntimeResp/);
}
function waitNodeOutput(p) {
    return p.startAndWaitForOutput(/Listening for new connections on 127\.0\.0\.1:(\d+)/);
}


function newNode(wsPort, tmpPath, name = 'node') {
    const cli = [
        pathNode, [
            '--dev',
            '--block-millisecs=1000',
            '--base-path=' + path.resolve(tmpPath, 'phala-node'),
            `--ws-port=${wsPort}`,
            '--rpc-methods=Unsafe',
        ]
    ];
    const cmd = cli.flat().join(' ');
    fs.writeFileSync(`${tmpPath}/start-${name}.sh`, `#!/bin/bash\n${cmd}\n`, { encoding: 'utf-8' });
    return new Process(cli, { logPath: `${tmpPath}/${name}.log` });
}

function newPRuntime(teePort, tmpPath, name = 'pruntime') {
    const workDir = path.resolve(`${tmpPath}/${name}`);
    const sealDir = path.resolve(`${workDir}/data`);
    if (!fs.existsSync(workDir)) {
        fs.mkdirSync(workDir);
        fs.mkdirSync(sealDir);
        const filesMustCopy = ['Rocket.toml', 'pruntime', 'pruntime.manifest'];
        const filesShouldCopy = ['GeoLite2-City.mmdb']
        filesMustCopy.forEach(f =>
            fs.copyFileSync(`${path.dirname(pathPRuntime)}/${f}`, `${workDir}/${f}`)
        );
        filesShouldCopy.forEach(f => {
            if (fs.existsSync(`${path.dirname(pathPRuntime)}/${f}`)) {
                fs.copyFileSync(`${path.dirname(pathPRuntime)}/${f}`, `${workDir}/${f}`)
            }
        });
    }
    return new Process([
        `gramine-direct`, [
            'pruntime',
            '--cores=0',	// Disable benchmark
            '--port', teePort.toString()
        ], {
            cwd: workDir,
        }
    ], { logPath: `${tmpPath}/${name}.log` });
}

function newRelayer(wsPort, teePort, tmpPath, gasAccountKey, key, name = 'relayer') {
    return new Process([
        pathRelayer, [
            '--no-wait',
            `--mnemonic=${gasAccountKey}`,
            `--inject-key=${key}`,
            `--substrate-ws-endpoint=ws://localhost:${wsPort}`,
            `--pruntime-endpoint=http://localhost:${teePort}`,
            '--dev-wait-block-ms=1000',
        ]
    ], { logPath: `${tmpPath}/${name}.log` });
}

function hex(b) {
    if (!b.startsWith('0x')) {
        return '0x' + b;
    } else {
        return b;
    }
}
