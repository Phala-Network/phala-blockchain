const axios = require('axios');

class PRuntime {
	constructor(basePath='http://localhost:8000') {
		this.api = axios.create({baseURL: basePath});
	}

	async req(method, data={}) {
		const r = await this.api.post(method, {
			input: data,
			nonce: { val: Math.random().toString() },
		}, { headers: { 'Content-Type': 'application/json' } });
		const payload = r.data.payload;
		return JSON.parse(payload);
	}

	async getInfo() {
		return await this.req('get_info');
	}
}

module.exports = { PRuntime };
