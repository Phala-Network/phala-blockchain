# Steps to add a worker to an existing cluster

- 1. Ensure the new worker is synced up-to-date and registered on-chain.

- 2. Add the new worker to the cluster by invoking the `add_worker_to_cluster` extrinsic.

- 3. Pause syncing on the new worker.

        The new worker must stop at a block before we take the snapshot on the old worker, because the new worker will pre-load the snapshot, sync to the snapshot block, and load it in the future.

- 4. Follow instructions in the [cluster-state-transfer.sh](/standalone/pruntime/scripts/cluster-state-transfer.sh) to transfer the state from the old worker to the new worker.

        Note: Make sure do the step 4 quickly after step 3, because the old worker will check the new worker's block height and reject the state transfer request if the new worker is too far behind(more than 128 blocks).

- 5. Resume syncing on the new worker.
