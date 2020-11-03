# XToken event roadmap

## XCM V0(implemented on polkadot gav-xcmp branch)

### xtoken.transfer_to_parachain(origin, x_currency_id: XCurrencyId, para_id: ParaId, dest: T::AccountId dest_network: NetworkId, amount: T::Balance)

transfer asset to specific parachain, no matter whether our chain is the observe chain of the asset.

 - events deposited by sender(our chain)

```sh
 xcmAdapter.WithdrawAsset -> messageBroker.UmpMessageSent -> localXcmHandler.XcmSuccess -> phalaXToken.TransferredToParachain
 ```

  - events deposited by relay chain

```sh
parachains.DmpMessageSent
```

 - events deposited by receiver(dest parachain)

 ```sh
xcmAdapter.WithdrawAsset -> xcmAdapter.DepositAsset -> messageBroker.DmpSuccess
 ```

 ### xtoken.transfer_to_relay_chain(origin, dest: T::AccountId, amount: T::Balance)

  - events deposited by sender(our chain)

```sh
xcmAdapter.WithdrawAsset -> messageBroker.UmpMessageSent -> localXcmHandler.XcmSuccess->phalaXToken.TransferredToRelayChain
```

 - events deposited by receiver(relay chain)

 ```sh
parachains.UmpSuccess
 ```

## XCM V0(implemented on polkadot master branch)