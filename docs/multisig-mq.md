# Multi-signed Cluster Egress Message Queue

This document discusses a new message queue system to prevent potential state divergences within clusters.

## Introduction

In our existing system, message queues (mq) are employed for workers and Gatekeepers to communicate messages to the chain. However, when dealing with clusters hosting Phat Contracts, the situation becomes a bit differrent due to the complicated tech stack involved. This complexity leads to higher chances of state divergence among the workers within a cluster.

Our current efforts to minimize state divergence include deploying versioned ink runtimes, but the inherent complexity of the tech stack means that there is no guarantee against divergence. Various factors, including minute software state discrepancies or external disturbances such as cosmic rays, can induce divergences in state among different workers.

When state divergence occurs, it is crucial to have a mechanism that can:
- Identify and reject messages from the compromised workers
- Terminate the compromised workers

## Proposed Solution: Multi-signed Sync Cluster Message API

To tackle the aforementioned problem, we propose a new kind of message queue submit API specifically designed for clusters, which we will call `sync_cluster_message`. 

This API functions by requiring messages to contain a list of signatures derived from the hash of the message, obtained from N workers within the cluster. Here, N is a configurable parameter that can be set according to the specific requirements of each cluster.

In addition to the message itself, the signed content includes a Serial Number (SN), the hash of the previous message and `the current cluster state root hash`. The SN is monotonically increasing to ensure a consistent order.

Upon receiving a message, the chain will validate the contents with each of the signatures. If any of the verifications fail, the message will be rejected and the SN will not increment.

When a message is accepted, the chain will deliver a ACK message to the sender. The ACK message will contain the SN of the accepted message and its hash. When processing the ACK message, it would check if the SN and hash matches their local queue. If not, that means the local state is diverged from the chain, and the worker would stop itself.

## Message Hunter Implementation

Since a multi-signature approach is being used, we need a specialized component, which we term as the “message hunter”, to collect messages from one worker and gather signatures from multiple workers within the cluster. To achieve this, two public RPCs need to be integrated into pRuntime: one for retrieving cluster messages and another for collecting signatures.

## Incentive Mechanism (Tokenomics) - To Be Developed

It is important to incentivize message hunters for their role in collecting and submitting messages. To achieve this, a reward system needs to be established. This involves the message sender compensating the message hunter with rewards for their service.

The reward for the message hunter will be distributed on-chain, whereas the payment from the message sender will be executed off-chain. The specifics of this tokenomics model are yet to be developed.

## Conclusion

TODO