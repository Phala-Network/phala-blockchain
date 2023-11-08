# @phala/sdk

## Quickstart

Here is a glance of how to use `@phala/sdk` with code snippets. We don't cover too much detail here, but it can be a cheatsheet when you working with it.

We recommend not install `@polkadot` packages directly, `@phala/sdk` will handle that with correlative dependencies.

```shell
npm install --save @phala/sdk
# Or yarn
yarn add @phala/sdk
```

You need create the `apiPromise` instance first, also the `OnChainRegistry` instance for the next:

```js
import { ApiPromise, WsProvider, Keyring } from '@polkadot/api'
import { options, OnChainRegistry, signCertificate, PinkContractPromise } from '@phala/sdk'

const RPC_MAINNET_URL = 'wss://api.phala.network/ws'
const RPC_TESTNET_URL = 'wss://poc6.phala.network/ws'

async function main() {
  const api = await ApiPromise.create(
    options({
      provider: new WsProvider(RPC_TESTNET_URL),
      noInitWarn: true,
    })
  )
  const phatRegistry = await OnChainRegistry.create(api)
}

main()
  .then(() => process.exit(0))
  .catch((err) => {
    console.error(err)
    process.exit(1)
  })
```

You might already upload and instantiate your Phat Contract, and you need prepare 3 things before go next:

- The ABI file. It names `metadata.json` in general, and you can found it along with your `.contract` file.
- `ContractId`. You can get this after upload & instantiate your Phat Contract.
- Your account. You can learn more from the [Keyring](https://polkadot.js.org/docs/api/start/keyring/) section in polkadot.js documentation.

> Tips
> You can found the information mentioned above in our web based UI tools [Phat Contract UI](https://phat.phala.network/)

We continue with `//Alice` in follow up code snippets, so all 3 things can be ready like:

```javascript
const keyring = new Keyring({ type: 'sr25519' })
const pair = keyring.addFromUri('//Alice')
const contractId = ''
const abi = JSON.parse(fs.readFileSync('./your_local_path/target/ink/metadata.json', 'utf-8'))
```

Now let's initializing the `PinkContractPromise` instance first.

```javascript
const contractKey = await phatRegistry.getContractKeyOrFail(contractId)
const contract = new PinkContractPromise(api, phatRegistry, abi, contractId, contractKey)
```

In the original version of polkadot.js, `tx` refer to the `write` operation and `query` refer to the `read` operation. But in Phat Contract, they got different concepts. **`tx` means on-chain operations, `query` means off-chain operations**. We recommended you put your computation logic in off-chain codes as much as possible.

We also need sign a certificate when using Phat Contract. Unlike signing for a transaction, the certificate is reusable. We recommended you cache it for a period of time.

```javascript
const cert = await signCertificate({ pair })
```

For off-chain computations (or `query` calls), we don't need set `gasLimit` and `storageDepositLimit` like what we did in the original polkadot contract, we use `cert` instead:

```javascript
// (We perform the send from an account, here using Alice's address)
const { gasRequired, storageDeposit, result, output } = await contract.query.methodName(pair.address, { cert })
```

For on-chain computations (or `tx` calls), you need estimate gas fee first. It's same as the original polkadot.js API Contract:

```javascript
const incValue = 1

const { gasRequired, storageDeposit } = await contract.query.inc(pair.address, { cert }, incValue)
const options = {
  gasLimit: gasRequired.refTime,
  storageDepositLimit: storageDeposit.isCharge ? storageDeposit.asCharge : null,
}
const result = await contract.tx.inc(options, incValue).signAndSend(pair, { nonce: -1 })
await result.waitFinalized()
```

> [!NOTE]
> Starting from version 0.5.4, we have introduced a new mode called "send" that handles complex gas fees and cluster balance management.
> We refer to this mode as "auto-deposit":

```javascript
const result = await contract.send.inc({ pair, cert, address: pair.address }, incValue)
await result.waitFinalized()
```

> [!IMPORTANT]
> We recommend call `waitFinalized` for the transaction to be finalized after submitting it, both for `tx` and `send`.
> When you call `waitFinalized`, the SDK will listen for any transaction errors and throw an error if one occurs. This feature provides a clear mindset when building apps with Phat Contract SDK.

This is a basic workaround using Phat Contract that can cover most of your use scenarios. You can read on for more advantage topics or check out the <a href="https://github.com/Leechael/phat-contract-sdk-cookbook">cookbook</a>.

---

## Adavantage Topics

### Sign Certificate with Browser Wallet Extensions

The `signCertificate` works a bit different with Browser Wallet Extensions, include the original [polkadot{js} extension](https://polkadot.js.org/docs/extension), [Talisman](https://www.talisman.xyz/), and [SubWallet](https://www.subwallet.app/).

We have two ways interact with extensions.

First one is the documented in the [polkadot{js} extension documentation](https://polkadot.js.org/docs/extension/cookbook):

```javascript
import { web3Accounts, web3Enable, web3FromSource } from '@polkadot/extension-dapp'

// this call fires up the authorization popup
const extensions = await web3Enable('My cool Phat Contract dApp')

const availableAccounts = await web3Accounts()
const account = availableAccounts[0] // assume you choice the first visible account.
const injector = await web3FromSource(account.meta.source)
const signer = injector.signer

const cert = await signCertificate({ signer, account })
```

The second one is less document but more progressive one. You can access the `window.injectedWeb3` object and inspect which Wallet Extension has been installed, let the user pick the one already in used, and interact with it directly.

```javascript
// polkadot-js: the original polkadot-js extension
// talisman: the Talisman extension
// subwallet-js: the SubWallet extension
const provider = 'polkadot-js'

const injector = window.injectedWeb3[provider]
const extension = await injector.enable('My cool Phat Contract dApp')
const accounts = await extension.accounts.get(true)
const account = availableAccounts[0] // assume you choice the first visible account.
const signer = extension.signer

const cert = await signCertificate({ signer, account })
```

### Connect to a specified PRuntime Node

Sometimes, we assign one PRuntime Node to handle all requests. This can be easily accomplished with a few lines of:

```javascript
const apiPromise = await ApiPromise.create(
  options({
    provider: new WsProvider('https://poc6.phala.network/ws'),
    noInitWarn: true,
  })
)
const registry = new OnChainRegistry(apiPromise)
await registry.connect({ pruntimeURL: '' })
```

On the other hands, you can design priorized strategy or shuffle simply:

```javascript
const apiPromise = await ApiPromise.create(
  options({
    provider: new WsProvider(ws),
    noInitWarn: true,
  })
)
const registry = new OnChainRegistry(apiPromise)
//
// Get all available workers for cluster 0x01.
//
const clusterId = '0x0000000000000000000000000000000000000000000000000000000000000001'
const workers = await registry.getClusterWorkers(clusterId)
//
// shuffle
//
const idx = Math.floor(workers.length * Math.random())
const picked = workers[idx]
//
// connect
//
await registry.connect(picked)
```

### Fetch Logs

We provide `PinkLoggerContractPromise` which can fetch logs from specified Pruntime node. You can use it fetching logs:

```javascript
const pinkLogger = await PinkLoggerContractPromise.create(api, phatRegistry, phatRegistry.systemContract)
```

The `PinkLoggerContractPromise` provides `head` and `tail` function like the command line utility `head` and `tail`:

- `head`: Fetch first N records from logserver.
- `tail`: Fetch last N records from logserver.

NOTE: logserver won't keep full log records, it only keep a small size of records for logs prevent consumes too much memory.

Let said we have 70 records in logserver:

- `pinkLogger.head()` / `pinkLogger.tail()`: fetch first/last 10 records.
- `pinkLogger.head(20)` / `pinkLogger.tail(20)`: fetch first/last 20 records.
- `pinkLogger.head(20, 15)`: fetch from starts but skip the first 15 records then returns next 20 records, so the
  records with sequence number between 15 and 35 will returns.
- `pinkLogger.tail(20, 15)`: fetch from ends but skip last 15 records, so the records with sequence number between 35
  and 55 will returns.
- `pinkLoger.head(10, { contract, block_number })` / `pinkLogger.tail(10, { contract, block_number })`: fetch first/last
  10 records which satisfied the specified filter conditions. Either `contract` and `block_number` are optional, but if
  you want use the filtering, you need at least provide one. `contract` is refer to the contract ID here.
- `pinkLogger.head(number, skip, filter)` / `pinkLogger.tail(number, skip, filter)`: fetching `number` log records and
  omit first/last `skip` records, and it should matched specified filter conditions.

You can check out a simple example (here)[https://github.com/Leechael/phat-contract-cli-upload/blob/main/tail.js]

### Retrieving Contract Information

We divide into multiple scenarios.

1. Get contract key by contract ID

You can fetch it from on-chain storage:

```javascript
const contractKey = await this.api.query.phalaRegistry.contractKeys(contractId)
```

We also provides it with `OnChainRegistry`:

```javascript
const contractKey = await phatRegistry.getContractKey(contractId)
```

Or use `getContractKeyOrFail`, which will throws error when no contract key found:

```javascript
const contractKey = await phatRegistry.getContractKeyOrFail(contractId)
```

2. Get code hash by Contract ID

You can query code hash by Contract ID via the phactory object:

```javascript
const contractInfo = await phatRegistry.phactory.getContractInfo({ contracts: [contractId] })
const codeHash = contractInfo.contracts[0].codeHash
```

3. Check code exists on the cluster or not

```typescript
const { output } = await systemContract.query['system::codeExists']<Bool>(account.address, { cert }, codeHash, 'Ink')
const hasExists = output && output.isOk && output.asOk.isTrue
```

### Upload & Instantiate Contracts

You can checkout the code example here: https://github.com/Leechael/phat-contract-sdk-cookbook/blob/main/upload.js

## Experimental Features

Some experimental features may change in the future, we will prefix the function name with `unstable_`.

### Sign certificate with Ethereum Wallets (EIP-712 compatible)

It means you can sign a certificate for Phat Contract off-chain query with any Ethereum Wallet. The feature built on-top
of [viem](https://viem.sh/).

Node.js code snippet example:

```javascript
import { createTestClient, http } from 'viem'
import { mainnet } from 'viem/chains'
import { privateKeyToAccount } from 'viem/accounts'
import { etherAddressToCompactPubkey, unstable_signEip712Certificate } from '@phala/sdk'

const account = privateKeyToAccount('<YOUR_PRIVATE_KEY>')
const client = createTestClient({ account, chain: mainnet, mode: 'anvil', transport: http() })
const compactPubkey = await etherAddressToCompactPubkey(client, account)
const cert = await unstable_signEip712Certificate({ client, account, compactPubkey })
// init & create your own PinkContractPromise instance.
const result = await contract.query.version(address, { cert })
```

Browser code snippet example (tested with MetaMask):

```javascript
import { createWalletClient, custom } from 'viem'
import { mainnet } from 'viem/chains'
import { etherAddressToCompactPubkey, unstable_signEip712Certificate } from '@phala/sdk'

const client = createWalletClient({ chain: mainnet, transport: custom(window.ethereum) })
const [address] = await client.requestAddresses()
const compactPubkey = await etherAddressToCompactPubkey(client, address)
const cert = await unstable_signEip712Certificate({ client, account: address, compactPubkey })
// init & create your own PinkContractPromise instance.
const result = await contract.query.version(address, { cert })
```
