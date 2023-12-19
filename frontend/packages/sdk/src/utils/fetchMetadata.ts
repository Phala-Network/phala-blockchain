import fetch from 'cross-fetch'

async function requestRpc(http: string, body: string) {
  return fetch(http, {
    method: 'POST',
    body,
    headers: { 'content-type': 'application/json' },
  }).then((resp) => resp.json())
}

export type LiteralRpc = `ws://${string}` | `wss://${string}`

export async function fetchMetadata(rpc: LiteralRpc) {
  const http = rpc
    .replace('wss://', 'https://')
    .replace('ws://', 'http://')
    .replace('ws:/', 'http://')
    .replace('wss:/', 'https://')
  const [blockHashResp, specResp, metadataResp] = await Promise.all([
    requestRpc(http, '{"id":1,"jsonrpc":"2.0","method":"chain_getBlockHash","params":[0]}'),
    requestRpc(http, '{"id":2,"jsonrpc":"2.0","method":"state_getRuntimeVersion","params":[]}'),
    requestRpc(http, '{"id":3,"jsonrpc":"2.0","method":"state_getMetadata","params":[]}'),
  ])
  const id = `${blockHashResp.result}-${specResp.result.specVersion}`

  return {
    [id]: metadataResp.result,
  }
}
