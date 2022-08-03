module.exports = {
    resolveDefaultEndpoint(endpoint) {
        const defaultEndpoints = {
            'polkadot': 'wss://rpc.polkadot.io',
            'kusama': 'wss://kusama-rpc.polkadot.io',
            'khala': 'wss://khala-api.phala.network/ws',
            'phala': 'wss://api.phala.network/ws',
            'local': 'ws://localhost:9944',
        }
        if (endpoint in defaultEndpoints) {
            return defaultEndpoints[endpoint];
        }
        return endpoint;
    }
}
