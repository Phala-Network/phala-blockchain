// Copyright 2017-2022 @polkadot/wasm-crypto authors & contributors
// SPDX-License-Identifier: Apache-2.0
import { packageInfo as bridgeInfo } from '@polkadot/wasm-bridge/packageInfo';
import { packageInfo as asmInfo } from '@polkadot/wasm-crypto-asmjs/packageInfo';
import { packageInfo as initInfo } from '@polkadot/wasm-crypto-init/packageInfo';
import { packageInfo as wasmInfo } from '@polkadot/wasm-crypto-wasm/packageInfo';
import { packageInfo as utilInfo } from '@polkadot/wasm-util/packageInfo';
export default [bridgeInfo, asmInfo, initInfo, wasmInfo, utilInfo];