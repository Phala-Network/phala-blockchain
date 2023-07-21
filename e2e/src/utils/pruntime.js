const axios = require('axios');
const { pruntime_rpc } = require('../proto/pruntime_rpc');
const { prpc } = require('../proto/prpc');

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
                .catch((error) => {
                    const status = error.response.status;
                    if (status == 500 || status == 400) {
                        const pbError = prpc.PrpcError.decode(error.response.data);
                        return callback(pbError);
                    } else {
                        return callback(error);
                    }
                })
        });
    }

    async getInfo() {
        return await this.rpc.getInfo({});
    }

    async getContractInfo(contractId) {
        const { contracts } = await this.rpc.getContractInfo({ contracts: [contractId] });
        return contracts[0];
    }

    async uploadSidevmCode(contract, code) {
        return await this.rpc.uploadSidevmCode({ contract, code });
    }

    async calculateContractId(args) {
        console.log(`calculateContractId(${JSON.stringify(args)})`);
        return await this.rpc.calculateContractId(args);
    }
}

// function rand() {
//     return (Math.random() * 65536) | 0;
// }

module.exports = { PRuntimeApi };
