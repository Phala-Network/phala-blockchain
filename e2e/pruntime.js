const axios = require('axios');

// TODO: make it a library (copied from scripts/js/console.js)
class PRuntimeApi {
    constructor(endpoint) {
        this.api = axios.create({
            baseURL: endpoint,
            headers: {
                'Content-Type': 'application/json',
            }
        });
    }
    async req(method, data = {}) {
        const r = await this.api.post('/' + method, {
            input: data,
            nonce: { id: rand() }
        });
        if (r.data.status === 'ok') {
            return JSON.parse(r.data.payload);
        } else {
            throw new Error(`Got error response: ${r.data}`);
        }
    }
    async query(contractId, request) {
        const bodyJson = JSON.stringify({
            contract_id: contractId,
            nonce: rand(),
            request
        });
        const payloadJson = JSON.stringify({ Plain: bodyJson });
        const queryData = { query_payload: payloadJson };
        const response = await this.req('query', queryData);
        const plainResp = JSON.parse(response.Plain);
        return plainResp;
    }

    async getInfo() {
        return await this.req('get_info');
    }
}

function rand() {
    return (Math.random() * 65536) | 0;
}

module.exports = { PRuntimeApi };
