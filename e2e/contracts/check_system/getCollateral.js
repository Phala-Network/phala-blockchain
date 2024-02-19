async function httpGet(url) {
    const response = await fetch(url);
    if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
    }
    return response;
}

async function getCollateral(baseUrl, fmspc) {
    // PCK CRL
    let pckCrlResponse = await httpGet(`${baseUrl}/pckcrl?ca=processor`);
    const pckCrlIssuerChain = pckCrlResponse.headers.get("SGX-PCK-CRL-Issuer-Chain");
    const pckCrl = await pckCrlResponse.text();

    // Root CA CRL
    let rootCaCrlResponse = await httpGet(`${baseUrl}/rootcacrl`);
    const rootCaCrl = await rootCaCrlResponse.text();

    // TCB Info
    let tcbInfoResponse = await httpGet(`${baseUrl}/tcb?fmspc=${fmspc}`);
    const tcbInfoIssuerChain = tcbInfoResponse.headers.get("SGX-TCB-Info-Issuer-Chain") || tcbInfoResponse.headers.get("TCB-Info-Issuer-Chain");
    const rawTcbInfo = await tcbInfoResponse.text();

    // QE Identity
    let qeIdentityResponse = await httpGet(`${baseUrl}/qe/identity`);
    const qeIdentityIssuerChain = qeIdentityResponse.headers.get("SGX-Enclave-Identity-Issuer-Chain");
    const rawQeIdentity = await qeIdentityResponse.text();

    // Parse JSON Data
    const tcbInfoJson = JSON.parse(rawTcbInfo);
    const tcbInfo = JSON.stringify(tcbInfoJson["tcbInfo"]);
    const tcbInfoSignature = Sidevm.hexDecode(tcbInfoJson["signature"]);

    const qeIdentityJson = JSON.parse(rawQeIdentity);
    const qeIdentity = JSON.stringify(qeIdentityJson["enclaveIdentity"]);
    const qeIdentitySignature = Sidevm.hexDecode(qeIdentityJson["signature"]);

    // Construct result object
    return {
        pckCrlIssuerChain,
        rootCaCrl,
        pckCrl,
        tcbInfoIssuerChain,
        tcbInfo,
        tcbInfoSignature,
        qeIdentityIssuerChain,
        qeIdentity,
        qeIdentitySignature
    };
}

async function main() {
    const pccsUrl = scriptArgs[0];
    const fmspc = scriptArgs[1];
    const collateral = await getCollateral(pccsUrl, fmspc);
    const typeRegistry = `
        String=str
        Collateral={
            pckCrlIssuerChain: String,
            rootCaCrl: String,
            pckCrl: String,
            tcbInfoIssuerChain: String,
            tcbInfo: String,
            tcbInfoSignature: Vec<u8>,
            qeIdentityIssuerChain: String,
            qeIdentity: String,
            qeIdentitySignature: Vec<u8>,
        }
    `;
    return Sidevm.SCALE.encode(collateral, "Collateral", typeRegistry);
}

main()
    .then(result => scriptOutput = result)
    .catch(error => {
        console.error(`Error: ${error}`);
        scriptOutput = error
    })
    .finally(() => process.exit());