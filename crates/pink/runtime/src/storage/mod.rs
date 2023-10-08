use crate::{
    capi::OCallImpl,
    runtime::{ExecSideEffects, Pink as PalletPink, System, SystemEvents, Timestamp},
    types::{EventsBlock, EventsBlockBody, EventsBlockHeader, Hash, Hashing},
};
use pink_capi::v1::ocall::ExecContext;
use scale::Encode;
use sp_state_machine::{backend::AsTrieBackend, Backend as StorageBackend, Ext, OverlayedChanges};

pub use external_backend::ExternalStorage;
use phala_trie_storage::BackendTransaction;

pub trait CommitTransaction: StorageBackend<Hashing> {
    fn commit_transaction(&mut self, root: Hash, transaction: BackendTransaction<Hashing>);
}

pub struct Storage<Backend> {
    backend: Backend,
}

impl<Backend> Storage<Backend> {
    pub fn new(backend: Backend) -> Self {
        Self { backend }
    }
}

impl<Backend> Storage<Backend>
where
    Backend: StorageBackend<Hashing> + CommitTransaction + AsTrieBackend<Hashing>,
{
    pub fn execute_with<R>(
        &self,
        exec_context: &ExecContext,
        f: impl FnOnce() -> R,
    ) -> (R, ExecSideEffects, OverlayedChanges<Hashing>) {
        let backend = self.backend.as_trie_backend();

        let mut overlay = OverlayedChanges::default();
        overlay.start_transaction();
        let mut ext = Ext::new(&mut overlay, backend, None);
        let (rv, effects) = sp_externalities::set_and_run_with_externalities(&mut ext, move || {
            Timestamp::set_timestamp(exec_context.now_ms);
            System::set_block_number(exec_context.block_number);
            System::reset_events();
            let result = f();
            let (system_events, effects) = crate::runtime::get_side_effects();
            maybe_emit_system_event_block(system_events);
            (result, effects)
        });
        overlay
            .commit_transaction()
            .expect("BUG: mis-paired transaction");
        (rv, effects, overlay)
    }

    pub fn execute_mut<R>(
        &mut self,
        context: &ExecContext,
        f: impl FnOnce() -> R,
    ) -> (R, ExecSideEffects) {
        let (rv, effects, overlay) = self.execute_with(context, f);
        self.commit_changes(overlay);
        (rv, effects)
    }

    pub fn changes_transaction(
        &self,
        changes: OverlayedChanges<Hashing>,
    ) -> (Hash, BackendTransaction<Hashing>) {
        let delta = changes
            .changes()
            .map(|(k, v)| (&k[..], v.value().map(|v| &v[..])));
        let child_delta = changes.children().map(|(changes, info)| {
            (
                info,
                changes.map(|(k, v)| (&k[..], v.value().map(|v| &v[..]))),
            )
        });

        self.backend
            .full_storage_root(delta, child_delta, sp_core::storage::StateVersion::V0)
    }

    pub fn commit_changes(&mut self, changes: OverlayedChanges<Hashing>) {
        let (root, transaction) = self.changes_transaction(changes);
        self.backend.commit_transaction(root, transaction)
    }

    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.backend
            .as_trie_backend()
            .storage(key)
            .expect("Failed to get storage key")
    }
}

pub mod external_backend;

pub fn maybe_emit_system_event_block(events: SystemEvents) {
    use pink_capi::v1::ocall::OCalls;
    if events.is_empty() {
        return;
    }
    if !OCallImpl.exec_context().mode.is_transaction() {
        return;
    }
    let body = EventsBlockBody {
        phala_block_number: System::block_number(),
        contract_call_nonce: OCallImpl.contract_call_nonce(),
        entry_contract: OCallImpl.entry_contract(),
        events,
    };
    let number = PalletPink::take_next_event_block_number();
    let header = EventsBlockHeader {
        parent_hash: PalletPink::last_event_block_hash(),
        number,
        runtime_version: crate::version(),
        body_hash: body.using_encoded(sp_core::hashing::blake2_256).into(),
    };
    let header_hash = sp_core::hashing::blake2_256(&header.encode()).into();
    PalletPink::set_last_event_block_hash(header_hash);
    let block = EventsBlock { header, body };
    OCallImpl.emit_system_event_block(number, block.encode());
}
