# @phala/sdk

> ⚠️ This package is under developing, and some features might change in the future.

## Install

Minimum node version: v16.

```sh
npm install @phala/sdk
# or
yarn add @phala/sdk
```

To work with Fat Contracts, you should also install the following dependencies:

```
@polkadot/api @polkadot/api-contract
```

## Usage

To start using the SDK, you will need to create a [Polkadot.js](https://polkadot.js.org/docs/api/start/create) API object connecting to the blockchain, and then construct a "decorated" Fat Contract object with the API object.

```js
// Imports
// ....

// Create a Polkadot.js ApiPromise object connecting to a Phala node (assuming the default port).
const wsProvider = new WsProvider('ws://localhost:9944');
const api = await ApiPromise.create({
    provider: wsProvider,
    types
});

// Create a contract object with the metadata and the contract id.
const pruntimeURL = 'http://127.0.0.1:8000';  // assuming the default port
const contractId = '0xa5ef1d6cb746b21a481c937870ba491d6fe120747bbeb5304c17de132e8d0392';  // your contract id
const metadata = /* load the metadata.json... */;
const contract = new ContractPromise(
    await create({api, baseURL: pruntimeURL, contractId}),
    JSON.parse(metadata),
    contractId
);
```

## The contract object

Fat Contract is an extension to [Parity ink!](https://paritytech.github.io/ink-docs/) smart contract. Phala SDK allows you to create the contract object compatible to the ink! contract object. In the above sample code, the `contract` object is a [`ContractPromise`](https://polkadot.js.org/docs/api-contract/start/contract.read) which share the same interface with the original Polkadot.js contract APIs.

However, in Fat Contract the intreaction under the hood is different from the original ink. In Fat Contract, the read and write operations (query and command) are end-to-end encrypted. In addition, the queries are sent to the Secure Enclave workers directly, rather than the blockchain node. Phala SDK takes care of the encryption and transport of the data. Therefore when creating the `ContractPromise` object, we pass an `ApiPromise` _deocrated_ by the Phala SDK's `create()` function as the first argument.

You can learn more about how to interact with a contract object from the official Polkadot.js docs:

- [Contract](https://polkadot.js.org/docs/api-contract/start/contract.read): queries (actions without mutating the contract states)
- [Contract Tx](https://polkadot.js.org/docs/api-contract/start/contract.tx): commands (actions mutating the contrast states)

> This is the manual constructoin of the contract object. For a higher level usage in the Phala SDK UI scaffold, please refer to `ContractLoader`.

## Send commands

Commands are carried by signed transactions. You will need to sign the transaction. Usually you connect to a browser extension ([Polkadot.js Extension](https://polkadot.js.org/docs/extension)) or an in-memory [keyring](https://polkadot.js.org/docs/keyring). The former one is often used in DApps, while the latter one is often used in node.

To work with Polkadot.js Extension, you need to get the injected `Signer` object, and then call with the method name.

```js
const r = await contract.tx.methodName({}, arg1, arg2, ...)
    .signAndSend(address, {signer});
```

To work with `Keyring`, you need to pass a key pair.

```js
const r = await contract.tx.methodName({}, arg1, arg2, ...)
    .signAndSend(keypair);
```

Please note that the snake_case name defined in the ink! contract is automatically converted to camelCase in Polkadot.js. The first argument of the method is the calling option, followed by the arguments to the method.

The options you can specify are `value` and `gasLimit`. However the weight system is not used in Fat Contract so far. We always give a sufficient large weight to the executor.

Commands are always encrypted by the Phala SDK. It generates an ephemeral key to establish an end-to-end encryption channel to the worker (via ECDH) every time it sends a command.

## Send queries

Queries are sent to the Secure Enclave worker directly via a RPC call. Unlike the original ink! contract where the read calls are not authenticated, Fat Contract queries are usually signed and encrypted. This enables Access Control in Fat Contracts.

To send a query, you need to create a `Certificate` object and use it to sign the query. With Polkadot.js Extension, you can create it with the account and the signer object:

```js
// Imports
const {signCertificate, CertificateData} = require('@phala/sdk');

// Create the certiciate object
const account = /* ... your account address in the signer .. */;
const certificate = await signCertificate({
    api,
    account,
    signer,
});
```

With `Keyring`, you can create it with a keypair:

```js
const certificateData = await signCertificate({
  api,
  pair: keypair,
});
```

Send a query with the certificate.

```js
// Send the query
const outcome = await contract.query.methodName(certificate, {}, arg0, arg1, ...);
```

Similar to a command, you can send a query via the generated methods. The first two arguments are the certificate object and the call option. Then the arguments to the method follow.

The return value `outcome` is a `ContractCallOutcome` object. It includes the following fields:

- `output`: - The return value of the call
- `result`: ContractExecResultResult - The execution result indicating if it's successful or not
- `debugMessage`: Text - Debug message
- `gasConsumed`: u64 - Consumed gas (not used in Fat Contract)
- `gasRequired`: u64 - Required gas (not used in Fat Contract)
- `storageDeposit`: StorageDeposit - not used in Fat Contract

### The certificate object

A certificate represents the ownership of an on-chain account. Instead of using the wallet to sign the query directly, Fat Contract adopts a chain of certificates to sign the query.

Interactive wallets like Polkadot.js Extension triggers a popup every time when signing a message. This becomes annoying for frequent queries in Fat Contracts. To overcome the limitation, we can make an one-time grant by signing a certificate chain, and use the leaf certificate to sign the queries. Some additional advantages are fine-grained permission and TTL control on the authentication.

Although non-interactive wallets like `Keyring` doesn't require user interaction, we still make certificate object as a unified interface.

## Development

### TODO

1. More configurable options such as certificate ttl.
