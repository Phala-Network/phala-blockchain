import { pruntime_rpc as pruntimeRpc } from "./proto"
import { fetch } from 'undici'

/**
 * Create a http client prepared for protobuf
 *
 * @param baseURL The base URL of the pruntime server
 * @returns A PhactoryAPI object
 */
export default function createPruntimeClient(baseURL: string) {
    const pruntimeApi = pruntimeRpc.PhactoryAPI.create(
        async (method, requestData, callback) => {
            try {
                const resp = await fetch(
                    `${baseURL}/prpc/PhactoryAPI.${method.name}`,
                    {
                        method: 'POST',
                        headers:{
                            "Content-Type": "application/octet-stream",
                        },
                        body: new Uint8Array(requestData),
                    }
                )
                const buffer = await (await resp.blob()).arrayBuffer()
                callback(null, new Uint8Array(buffer))
            } catch (err) {
                // @NOTE How can we improve the error handling here?
                console.error('PRuntime Transport Error:', err)
                throw err
            }
        }
    )
    return pruntimeApi;
}

