const axios = require('axios');
const {pruntime_rpc} = require('../proto/pruntime_rpc');

// TODO: make it a library (copied from scripts/js/console.js)
class PRuntimeApi {
    constructor(endpoint) {
        this.api = axios.create({
            baseURL: endpoint,
            headers: {
                'Content-Type': 'application/octet-stream',
            },
            responseType: 'arraybuffer',
        });
    }
    async req(method, data = undefined) {
        const r = await this.api.post('/prpc/' + method, data);
        return pruntime_rpc.PhactoryInfo.decode(r.data);
    }
    async query(_contractId, _request) {
        throw new Error('Unimplemented');
    }

    async getInfo() {
        return await this.req('PhactoryAPI.GetInfo');
    }
}

// function rand() {
//     return (Math.random() * 65536) | 0;
// }

module.exports = { PRuntimeApi };
