use scale::{Decode, Encode};

/// TLS server configuration.
#[derive(Encode, Decode, Clone)]
pub enum TlsServerConfig {
    V0 {
        /// Certificate in PEM format.
        cert: String,
        /// PKCS8-encoded private key of the certificate in PEM format.
        key: String,
    },
}
