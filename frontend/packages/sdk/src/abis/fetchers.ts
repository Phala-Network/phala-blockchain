import { type Bool } from '@polkadot/types'
import fetch from 'cross-fetch'
import { type PinkContractPromise } from '../contracts/PinkContract'
import { type OnChainRegistry } from '../OnChainRegistry'
import { type CertificateData } from '../pruntime/certificate'

const OFFICIAL_ARTIFACTS_URL = 'https://phala-network.github.io/phat-contract-artifacts'

export interface CheckCodeHashExistsEnv {
  systemContract: PinkContractPromise
  cert: CertificateData
}

export function unsafeCheckCodeHashExists(env: CheckCodeHashExistsEnv) {
  const { systemContract, cert } = env
  return async function _unsafeCheckCodeHashExists(codeHash: string) {
    const { output } = await systemContract.query['system::codeExists']<Bool>(
      cert.address,
      { cert },
      `0x${codeHash}`,
      'Ink'
    )
    return output && output.isOk && output.asOk.isTrue
  }
}

export async function unsafeGetContractCodeHash(
  phatRegistry: OnChainRegistry,
  contractId: string
): Promise<string | null> {
  const payload = await phatRegistry.phactory.getContractInfo({ contracts: [contractId] })
  return payload?.contracts[0]?.codeHash || null
}

export async function unsafeGetAbiFromPatronByCodeHash(codeHash: string): Promise<Record<string, unknown>> {
  const codeHashWithoutPrefix = codeHash.indexOf('0x') === 0 ? codeHash.replace('0x', '') : codeHash
  const resp = await fetch(`https://api.patron.works/buildSessions/metadata/${codeHashWithoutPrefix}`)
  if (resp.status !== 200) {
    let payload
    try {
      payload = await resp.json()
    } catch (_err1) {
      try {
        const text = await resp.text()
        throw new Error(`Failed to get abi from Patron: ${resp.status}: ${text}`)
      } catch (_err2) {
        throw new Error(`Unknown Error: ${resp.status}: ${_err2}`)
      }
    }
    throw new Error(`Failed to get abi from Patron: ${resp.status}: ${(payload as any)?.error || 'Unknown Error'}`)
  }
  return (await resp.json()) as Record<string, unknown>
}

export async function unsafeGetAbiFromGitHubRepoByCodeHash(codeHash: string): Promise<Record<string, unknown>> {
  const codeHashWithPrefix = codeHash.indexOf('0x') !== 0 ? `0x${codeHash}` : codeHash
  const resp = await fetch(`${OFFICIAL_ARTIFACTS_URL}/artifacts/${codeHashWithPrefix}/metadata.json`)
  if (resp.status !== 200) {
    throw new Error(`Failed to get abi from GitHub: ${resp.status}`)
  }
  return (await resp.json()) as Record<string, unknown>
}

export async function unsafeGetWasmFromPatronByCodeHash(codeHash: string): Promise<Uint8Array> {
  const codeHashWithoutPrefix = codeHash.indexOf('0x') === 0 ? codeHash.replace('0x', '') : codeHash
  const resp = await fetch(`https://api.patron.works/buildSessions/wasm/${codeHashWithoutPrefix}`)
  if (resp.status !== 200) {
    throw new Error(`Failed to get wasm from Patron: ${resp.status}`)
  }
  const buffer = await resp.arrayBuffer()
  return new Uint8Array(buffer)
}

export async function unsafeGetWasmFromGithubRepoByCodeHash(codeHash: string): Promise<Uint8Array> {
  const codeHashWithPrefix = codeHash.indexOf('0x') !== 0 ? `0x${codeHash}` : codeHash
  const resp = await fetch(`${OFFICIAL_ARTIFACTS_URL}/artifacts/${codeHashWithPrefix}/out.wasm`)
  if (resp.status !== 200) {
    throw new Error(`Failed to get wasm from GitHub: ${resp.status}`)
  }
  const buffer = await resp.arrayBuffer()
  return new Uint8Array(buffer)
}
