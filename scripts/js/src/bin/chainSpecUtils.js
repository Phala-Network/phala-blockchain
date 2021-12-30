// Utility to manipuate a chain spec
//
// Can be useful after a "hard-spoon" of a live blockchain to spawn a new chain for testing.
// Partically forked from fork-off-substrate (https://raw.githubusercontent.com/maxsam4/fork-off-substrate)

const { loadJson, /*writeJson*/ } = require('../utils/common');
const { xxhashAsHex } = require('@polkadot/util-crypto');

const { program } = require('commander');

program
    .option('--replace-storage <storage-json-file>', 'Replace the storage in the chain-spec')
    .option(
        '--skip-pallets <pallets>',
        'The pallets not to replace the storage, separated by commas. This doesn\'t affect --kept-prefix.',
        'System,Session,Babe,Grandpa,GrandpaFinality,FinalityTracker,Authorship,CollatorSelection,ParachainInfo,ParachainSystem,Aura,AuraExt'
    )
    .option(
        '--kept-prefixes <prefixes>',
        'The storage prefixes to always replace, separated by commas.',
        '0x26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da9' // System.Account
    )
    .option(
        '--remove-storage-version <pallets>',
        'Remove the storage version of the specific pallet to trigger storage migration, separated by commas.'
    )
    .option('--rename-spec <suffix>', 'Append -"suffix" to the chain name and id.')
    .option(
        '--trigger-upgrade',
        'Delete System.LastRuntimeUpgrade to trigger a runtime upgrade.', false
    )
    .option('--set-alice-as-root', 'Set //Alice as the root account.', false)
    .argument('<orig-spec>', 'The original chain spec to modify.')
    .action(main)
    .parse(process.argv);

// async function fixParachinStates (api, forkedSpec) {
//   const skippedKeys = [
//     api.query.parasScheduler.sessionStartBlock.key()
//   ];
//   for (const k of skippedKeys) {
//     delete forkedSpec.genesis.raw.top[k];
//   }
// }

function main(origSpecPath) {
    const opts = program.opts();
    // const origSpec = loadJson(origSpecPath);
    const targetSpec = loadJson(origSpecPath);

    if (opts.replaceStorage) {
        const storageSpec = loadJson(opts.replaceStorage);

        const keptPrefixes = opts.keptPrefixes.split(',');
        const skipPallets = opts.skipPallets.split(',');
        const skipPrefixes = skipPallets.map(p => xxhashAsHex(p, 128));

        // Grab the items to be moved, then iterate through and insert into storage
        Object.entries(storageSpec.genesis.raw.top)
            .filter(([k, _v]) =>
                k != '0x3a636f6465'
                && (keptPrefixes.some(prefix => k.startsWith(prefix))
                    || !skipPrefixes.some(prefix => k.startsWith(prefix)))
            )
            .forEach(([key, value]) => (targetSpec.genesis.raw.top[key] = value));
    }

    // Remove <Pallet>/:__STORAGE_VERSION__: to ensure the storage migration can be triggered
    if (opts.removeStorageVersion) {
        const STORAGE_VERSION_POSTFIX = xxhashAsHex(':__STORAGE_VERSION__:').substring(2);
        const prefixes = opts.removeStorageVersion
            .split(',')
            .map(p => xxhashAsHex(p, 128) + STORAGE_VERSION_POSTFIX);
        Object.keys(targetSpec.genesis.raw.top)
            .filter(k => prefixes.some(prefix => k.startsWith(prefix)))
            .forEach(k => delete targetSpec.genesis.raw.top[k]);
    }

    // Delete System.LastRuntimeUpgrade to ensure that the on_runtime_upgrade event is triggered
    if (opts.triggerUpgrade) {
        delete targetSpec.genesis.raw.top['0x26aa394eea5630e07c48ae0c9558cef7f9cce9c888469bb1a0dceaa129672ef8'];
    }

    // Modify chain name and id
    if (opts.renameSpec) {
        targetSpec.name = originalSpec.name + '-' + opts.renameSpec;
        targetSpec.id = originalSpec.id + '-' + opts.renameSpec;
    }

    // Set sudo key to //Alice
    if (opts.setAliceAsRoot) {
        targetSpec.genesis.raw.top['0x5c0d1176a568c1f92944340dbfed9e9c530ebca703c85910e7164cb7d1c9e47b'] = '0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d';
    }

    // fixParachinStates(api, forkedSpec);

    // Replace the code
    // forkedSpec.genesis.raw.top['0x3a636f6465'] = '0x' + fs.readFileSync(hexPath, 'utf8').trim();

    // To prevent the validator set from changing mid-test, set Staking.ForceEra to ForceNone ('0x02')
    const STAKING_FORCE_ERA = '0x5f3e4907f716ac89b6347d15ececedcaf7dad0317324aecae8744b87fc95f2f3';
    if (targetSpec.genesis.raw.top[STAKING_FORCE_ERA]) {
        forkedSpec.genesis.raw.top[STAKING_FORCE_ERA] = '0x02';
    }

    console.log(JSON.stringify(targetSpec, null, 2));
}
