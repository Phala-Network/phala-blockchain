import fetch from 'cross-fetch'
import { prpc, pruntime_rpc as pruntimeRpc } from './proto'

/**
 * Create a http client prepared for protobuf
 *
 * @param baseURL The base URL of the pruntime server
 * @returns A PhactoryAPI object
 */
export default function createPruntimeClient(baseURL: string) {
  const pruntimeApi = pruntimeRpc.PhactoryAPI.create(async (method, requestData, callback) => {
    try {
      const resp = await fetch(`${baseURL}/prpc/PhactoryAPI.${method.name}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/octet-stream',
        },
        body: new Uint8Array(requestData),
      })
      if (resp.status === 200) {
        const buffer = await (await resp.blob()).arrayBuffer()
        callback(null, new Uint8Array(buffer))
      } else if (resp.status === 400 || resp.status === 500) {
        // We assume it's an error message from PRPC so we try to decode it first,
        // then fail back to plain text.
        let error: Error
        try {
          const buffer = await (await resp.blob()).arrayBuffer()
          const prpcError = prpc.PrpcError.decode(new Uint8Array(buffer))
          error = new Error(`PrpcError: ${prpcError.message}`)
        } catch (err) {
          error = new Error(`ServerError: ${resp.status}: ${await resp.text()}`)
        }
        throw error
      } else {
        throw new Error(`Unexpected Response Status: ${resp.status}`)
      }
    } catch (err) {
      // @NOTE How can we improve the error handling here?
      console.error('PRuntime Transport Error:', err)
      throw err
    }
  })
  return pruntimeApi
}
