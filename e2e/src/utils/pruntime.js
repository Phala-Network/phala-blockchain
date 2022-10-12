const axios = require('axios');
const { pruntime_rpc } = require('../proto/pruntime_rpc');

// TODO: make it a library (copied from scripts/js/console.js)
class PRuntimeApi {
    constructor(endpoint) {
        this.uri = endpoint;
        const client = axios.create({
            baseURL: endpoint,
            headers: {
                'Content-Type': 'application/octet-stream',
            },
            responseType: 'arraybuffer',
        });
        this.rpc = new pruntime_rpc.PhactoryAPI((method, data, callback) => {
            client.post('/prpc/PhactoryAPI.' + method.name, data)
                .then((r) => callback(null, r.data))
                .catch((error) => callback(error))
        });
    }

    async getInfo() {
        return await this.rpc.getInfo({});
    }

    async getContractInfo(contractId) {
        const contractIds = [contractId];
        const { contracts } = await this.rpc.getContractInfo({ contractIds });
        return contracts[0];
    }
}

// function rand() {
//     return (Math.random() * 65536) | 0;
// }

module.exports = { PRuntimeApi };
