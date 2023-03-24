# Steps to add a worker to an existing cluster

- 1. Ensure the new worker is synced up-to-date and registered on-chain.

- 2. Add the new worker to the cluster by invoking the `add_worker_to_cluster` extrinsic.

- 3. Pause syncing on the new worker.

- 4. Follow instructions in the [cluster-state-transfer.sh](/standalone/pruntime/scripts/cluster-state-transfer.sh) to transfer the state from the old worker to the new worker.

- 5. Resume syncing on the new worker.
