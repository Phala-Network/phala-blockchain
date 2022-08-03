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
const { checkUntil, skipSlowTest, sleep } = require('./utils');

const pathNode = path.resolve('../target/release/phala-node');
const pathRelayer = path.resolve('../target/release/pherry');

const pRuntimeBin = "pruntime";
const pathPRuntime = path.resolve(`../standalone/pruntime/bin/${pRuntimeBin}`);


// TODO: Switch to [instant-seal-consensus](https://substrate.dev/recipes/kitchen-node.html) for faster test

describe('A full stack', function () {
    this.timeout(60000);

    let cluster;
    let api, keyring, alice, bob, root;
    let pruntime;
    const tmpDir = new TempDir();
    const tmpPath = tmpDir.dir;

    before(async () => {
        // Check binary files
        [pathNode, pathRelayer, pathPRuntime].map(fs.accessSync);
        // Bring up a cluster
        cluster = new Cluster(4, pathNode, pathRelayer, pathPRuntime, tmpPath);
        await cluster.start();
        // APIs
        api = await cluster.api;
        pruntime = cluster.workers.map(w => w.api);
        // Create polkadot api and keyring
        await cryptoWaitReady();
        keyring = new Keyring({ type: 'sr25519', ss58Format: 30 });
        root = alice = keyring.addFromUri('//Alice');
        bob = keyring.addFromUri('//Bob');
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

            // Check if the on-chain role is Gatekeeper
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

    describe('Gatekeeper2', () => {
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

            // Check if the on-chain role is Gatekeeper
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
            const dataDir = "data";
            assert.isTrue(
                fs.existsSync(`${tmpPath}/pruntime1/${dataDir}/protected_files/master_key.seal`),
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

        it('can be unregistered', async function () {
            const info = await pruntime[1].getInfo();
            await assert.txAccepted(
                api.tx.sudo.sudo(
                    api.tx.phalaRegistry.unregisterGatekeeper(hex(info.publicKey))
                ),
                alice,
            );
            // Check if the role is no longer Gatekeeper
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[1].getInfo();
                const gatekeepers = await api.query.phalaRegistry.gatekeeper();
                // console.log(`Gatekeepers after unregisteration: ${gatekeepers}`);
                return info.gatekeeper.role != 2 && !gatekeepers.includes(hex(info.publicKey));
            }, 4 * 6000), 'not unregistered');
        });

        it('can be re-registered before rotation', async function () {
            const info = await pruntime[1].getInfo();
            await assert.txAccepted(
                api.tx.sudo.sudo(
                    api.tx.phalaRegistry.registerGatekeeper(hex(info.publicKey))
                ),
                alice,
            );

            // Check if the on-chain role is Gatekeeper
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[1].getInfo();
                const gatekeepers = await api.query.phalaRegistry.gatekeeper();
                // console.log(`Gatekeepers after registeration: ${gatekeepers}`);
                return gatekeepers.includes(hex(info.publicKey));
            }, 4 * 6000), 'not registered as gatekeeper');
            // the GK should resume without restart
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[1].getInfo();
                return info.gatekeeper.role == 2;  // 2: GatekeeperRole.Active in protobuf
            }, 4 * 6000), 'gatekeeper role not changed')
        });
    });

    describe('Master Key Rotation', () => {
        it('can register and un-reg gatekeeper4', async function () {
            // Register worker4 as Gatekeeper
            const info = await pruntime[3].getInfo();
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
                const info = await pruntime[3].getInfo();
                return info.registered;
            }, 4 * 6000), 'not registered in time');

            // Check if the on-chain role is Gatekeeper
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[3].getInfo();
                const gatekeepers = await api.query.phalaRegistry.gatekeeper();
                // console.log(`Gatekeepers after registeration: ${gatekeepers}`);
                return gatekeepers.includes(hex(info.publicKey));
            }, 4 * 6000), 'not registered as gatekeeper');

            assert.isTrue(
                await cluster.waitWorkerExitAndRestart(3, 10 * 6000),
                'worker4 restart timeout'
            );

            // Ensure it is up-to-date
            const gatekeeper = api.createType('MessageOrigin', 'Gatekeeper');
            let seqStart = await api.query.phalaMq.offchainIngress(gatekeeper);
            seqStart = seqStart.unwrap().toNumber();
            assert.isTrue(await checkUntil(async () => {
                let seq = await api.query.phalaMq.offchainIngress(gatekeeper);
                seq = seq.unwrap().toNumber();
                return seq >= seqStart + 1;
            }, 500 * 6000), 'ingress stale');

            // Unregister
            await assert.txAccepted(
                api.tx.sudo.sudo(
                    api.tx.phalaRegistry.unregisterGatekeeper(hex(info.publicKey))
                ),
                alice,
            );
            // Check if the role is no longer Gatekeeper
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[3].getInfo();
                const gatekeepers = await api.query.phalaRegistry.gatekeeper();
                // console.log(`Gatekeepers after unregisteration: ${gatekeepers}`);
                return info.gatekeeper.role != 2 && !gatekeepers.includes(hex(info.publicKey));
            }, 4 * 6000), 'not unregistered');
        });

        it('can rotate master key', async function () {
            const info = await pruntime[0].getInfo();
            const old_master_pubkey = hex(info.gatekeeper.masterPublicKey);
            await assert.txAccepted(
                api.tx.sudo.sudo(
                    api.tx.phalaRegistry.rotateMasterKey()
                ),
                alice,
            );
            // only one rotation is allowed one time
            await assert.txFailed(
                api.tx.sudo.sudo(
                    api.tx.phalaRegistry.rotateMasterKey()
                ),
                alice,
            )

            assert.isTrue(await checkUntil(async () => {
                const info1 = await pruntime[0].getInfo();
                const info2 = await pruntime[1].getInfo();
                return hex(info1.gatekeeper.masterPublicKey) != old_master_pubkey
                    && hex(info1.gatekeeper.masterPublicKey) == hex(info2.gatekeeper.masterPublicKey);
            }, 4 * 6000), 'local master key not rotated');

            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[0].getInfo();
                const masterPubkey = await api.query.phalaRegistry.gatekeeperMasterPubkey();
                // console.log(`Master PubKey: ${masterPubkey}`);
                return masterPubkey == hex(info.gatekeeper.masterPublicKey);
            }, 4 * 6000), 'on-chain master key not updated');
        });

        it('will kill unregistered gatekeeper', async function () {
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[3].getInfo();
                return info.gatekeeper.role == 0;
            }, 4 * 6000), 'outdated gatekeeper not remove');
        });

        it('can re-register the gatekeeper4 after rotation', async function () {
            // Register worker4 as Gatekeeper
            const info = await pruntime[3].getInfo();
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
                const info = await pruntime[3].getInfo();
                return info.registered;
            }, 4 * 6000), 'not registered in time');

            // Check if the on-chain role is Gatekeeper
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[3].getInfo();
                const gatekeepers = await api.query.phalaRegistry.gatekeeper();
                // console.log(`Gatekeepers after registeration: ${gatekeepers}`);
                return gatekeepers.includes(hex(info.publicKey));
            }, 4 * 6000), 'not registered as gatekeeper');

            assert.isTrue(
                await cluster.waitWorkerExitAndRestart(3, 10 * 6000),
                'worker4 restart timeout'
            );

            // Ensure it is up-to-date
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

    describe('Gatekeeper3 after rotation', () => {
        it('can be registered after rotation', async function () {
            // Register worker3 as Gatekeeper
            const info = await pruntime[2].getInfo();
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
                const info = await pruntime[2].getInfo();
                return info.registered;
            }, 4 * 6000), 'not registered in time');

            // Check if the on-chain role is Gatekeeper
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[2].getInfo();
                const gatekeepers = await api.query.phalaRegistry.gatekeeper();
                // console.log(`Gatekeepers after registeration: ${gatekeepers}`);
                return gatekeepers.includes(hex(info.publicKey));
            }, 4 * 6000), 'not registered as gatekeeper');
        });

        it('can receive master key', async function () {
            // Wait for the successful dispatch of master key
            // pRuntime[2] should be down
            assert.isTrue(
                await cluster.waitWorkerExitAndRestart(2, 10 * 6000),
                'worker1 restart timeout'
            );
            const dataDir = "data";
            assert.isTrue(
                fs.existsSync(`${tmpPath}/pruntime2/${dataDir}/protected_files/master_key.seal`),
                'master key not received'
            );
        });

        it('becomes active', async function () {
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[2].getInfo();
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

        it('syncs latest master key', async function () {
            assert.isTrue(await checkUntil(async () => {
                const info = await pruntime[2].getInfo();
                const masterPubkey = await api.query.phalaRegistry.gatekeeperMasterPubkey();
                // console.log(`Master PubKey: ${masterPubkey}`);
                return masterPubkey == hex(info.gatekeeper.masterPublicKey);
            }, 1000));
        });
    });

    describe('Cluster & Contract', () => {
        let wasmFile = './res/flipper.wasm';
        let codeHash = hex('0x5b1f7f0b85908c168c0d0ca65efae0e916455bf527b8589d3e655311ec7b1f2c');
        let initSelector = hex('0xed4b9d1b'); // for default() function
        let clusterId;

        it('can create cluster', async function () {
            const perm = api.createType('ClusterPermission', { 'OnlyOwner': alice.address });
            const runtime0 = await pruntime[0].getInfo();
            const runtime1 = await pruntime[1].getInfo();
            const { events } = await assert.txAccepted(
                api.tx.phalaFatContracts.addCluster(perm, [hex(runtime0.publicKey), hex(runtime1.publicKey)]),
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

        it('can deploy cluster to multiple workers', async function () {
            assert.isTrue(await checkUntil(async () => {
                const clusterWorkers = await api.query.phalaFatContracts.clusterWorkers(clusterId);
                return clusterWorkers.length == 2;
            }, 4 * 6000), 'cluster not deployed');
        });

        it('can upload code with access control', async function () {
            let code = fs.readFileSync(wasmFile, 'hex');
            let InkCode = 0;
            // For now, there is no way to check whether code is uploaded in script
            // since this requires monitering the async CodeUploaded event
            await assert.txAccepted(
                api.tx.phalaFatContracts.clusterUploadResource(clusterId, InkCode, hex(code)),
                alice,
            );
            await assert.txFailed(
                api.tx.phalaFatContracts.clusterUploadResource(clusterId, InkCode, hex(code)),
                bob,
            )
        });

        it('can instantiate contract with access control', async function () {
            const codeIndex = api.createType('CodeIndex', { 'WasmCode': codeHash });
            await assert.txFailed(
                api.tx.phalaFatContracts.instantiateContract(codeIndex, initSelector, 0, clusterId),
                bob,
            );
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

    describe('Workerkey handover', () => {
        it('can handover worker key', async function () {
            await cluster.launchKeyHandoverAndWait();

            const workers = cluster.key_handover_cluster.workers.map(w => w.api);
            const server_info = await workers[0].getInfo();

            assert.isTrue(await checkUntil(async () => {
                const dataDir = "data";
                return fs.existsSync(`${tmpPath}/pruntime_key_client/${dataDir}/protected_files/runtime-data.seal`);
            }, 6000), 'key not received');

            await cluster.restartKeyHandoverClient();
            assert.isTrue(await checkUntil(async () => {
                // info.publicKey can be null since client worker will be initiated after the finish of server worker syncing
                let info = await workers[1].getInfo();
                return info.publicKey != null
                    && hex(info.publicKey) == hex(server_info.publicKey);
            }, 6000), 'key handover failed');
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
        this.key_handover_cluster = {
            workers: [{}, {}],
            relayer: {},
        };
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
            ]).flat(),
            this.key_handover_cluster.relayer.processRelayer.kill(),
            ...this.key_handover_cluster.workers.map(w => w.processPRuntime.kill('SIGKILL')).flat(),
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
            ...this.workers.map((w, i) => portfinder.getPortPromise({ port: 8100 + i * 10 }))
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
            '//Charlie',
            '//Dave',
            '//Eve',
            '//Fredie',
        ];
        const w = this.workers[i];
        const gasAccountKey = AVAILBLE_ACCOUNTS[i];
        const key = '0'.repeat(63) + (i + 1).toString();
        w.processRelayer = newRelayer(this.wsPort, w.port, this.tmpPath, gasAccountKey, key, `relayer${i}`);
        w.processPRuntime = newPRuntime(w.port, this.tmpPath, `pruntime${i}`);
    }

    async launchKeyHandoverAndWait() {
        const cluster = this.key_handover_cluster;

        const [...workerPorts] = await Promise.all([
            ...cluster.workers.map((w, i) => portfinder.getPortPromise({ port: 8200 + i * 10 }))
        ]);
        cluster.workers.forEach((w, i) => w.port = workerPorts[i]);

        const server = cluster.workers[0];
        const client = cluster.workers[1];
        server.processPRuntime = newPRuntime(server.port, this.tmpPath, `pruntime_key_server`);
        client.processPRuntime = newPRuntime(client.port, this.tmpPath, `pruntime_key_client`);

        const gasAccountKey = '//Fredie';
        const key = '0'.repeat(62) + '10';
        cluster.relayer.processRelayer = newRelayer(this.wsPort, server.port, this.tmpPath, gasAccountKey, key, `pruntime_key_relayer`, client.port);

        await Promise.all([
            ...cluster.workers.map(w => waitPRuntimeOutput(w.processPRuntime)),
            waitRelayerOutput(cluster.relayer.processRelayer)
        ]);

        cluster.workers.forEach(w => {
            w.api = new PRuntimeApi(`http://localhost:${w.port}`);
        })
    }

    async restartKeyHandoverClient() {
        const cluster = this.key_handover_cluster;
        await checkUntil(async () => {
            return cluster.relayer.processRelayer.stopped
        }, 6000);

        const client = cluster.workers[1];
        const gasAccountKey = '//Fredie';
        cluster.relayer.processRelayer = newRelayer(this.wsPort, client.port, this.tmpPath, gasAccountKey, '', `pruntime_key_relayer`);

        await waitRelayerOutput(cluster.relayer.processRelayer);
    }

    // Returns false if waiting is timeout; otherwise it restarts the pherry and the key handover client
    async waitKeyHandoverClientExitAndRestart(timeout) {
        const w = this.cluster.workers[1];
        const succeed = await checkUntil(async () => {
            return w.processPRuntime.stopped && w.processRelayer.stopped
        }, timeout);
        if (!succeed) {
            return false;
        }
        const client = cluster.workers[1];
        client.processPRuntime = newPRuntime(client.port, this.tmpPath, `pruntime_key_client`);
        // connect the pherry to the new pRuntime and inject no key
        cluster.relayer.processRelayer = newRelayer(this.wsPort, client.port, this.tmpPath, gasAccountKey, '', `pruntime_key_relayer`);
        await waitPRuntimeOutput(client.processPRuntime);
        await waitRelayerOutput(cluster.relayer.processRelayer);
        return true;
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
    return p.startAndWaitForOutput(/Rocket has launched from/);
}
function waitRelayerOutput(p) {
    return p.startAndWaitForOutput(/runtime_info: InitRuntimeResp/);
}
function waitNodeOutput(p) {
    return p.startAndWaitForOutput(/Imported #1/);
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

function newPRuntime(teePort, tmpPath, name = 'app') {
    const workDir = path.resolve(`${tmpPath}/${name}`);
    const sealDir = path.resolve(`${workDir}/data`);
    if (!fs.existsSync(workDir)) {
        fs.mkdirSync(workDir);
        fs.mkdirSync(sealDir);
        const filesMustCopy = ['Rocket.toml', pRuntimeBin];
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
    const args = [
        '--cores=0',  // Disable benchmark
        '--port', teePort.toString()
    ];
    return new Process([
        `${workDir}/${pRuntimeBin}`, args, {
            cwd: workDir,
            env: {
                ...process.env,
                ROCKET_PORT: teePort.toString(),
                RUST_LOG: 'debug'
            }
        }
    ], { logPath: `${tmpPath}/${name}.log` });
}

function newRelayer(wsPort, teePort, tmpPath, gasAccountKey, key = '', name = 'relayer', keyClientPort = '') {
    const args = [
        '--no-wait',
        `--mnemonic=${gasAccountKey}`,
        `--substrate-ws-endpoint=ws://localhost:${wsPort}`,
        `--pruntime-endpoint=http://localhost:${teePort}`,
        '--dev-wait-block-ms=1000',
    ];

    if (key) {
        args.push(`--inject-key=${key}`);
    }
    if (keyClientPort) {
        args.push(`--next-pruntime-endpoint=http://localhost:${keyClientPort}`);
    }

    return new Process([
        pathRelayer, args
    ], { logPath: `${tmpPath}/${name}.log` });
}

function hex(b) {
    if (!b.startsWith('0x')) {
        return '0x' + b;
    } else {
        return b;
    }
}
