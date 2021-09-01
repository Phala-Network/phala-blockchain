const { PRuntimeApi } = require('./utils/pruntime');

const api = new PRuntimeApi('http://127.0.0.1:8000');


async function main() {
    const r = await api.getInfo();
    console.log(r.toJSON());
}

main().catch(console.error).finally(() => process.exit());
