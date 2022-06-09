use scale::{Decode, Encode};

/// TLS server configuration.
#[derive(Encode, Decode, Clone)]
pub enum TlsServerConfig {
    V0 {
        /// Certificate in PEM format.
        cert: String,
        /// The private key of the certificate, in PEM format.
        key: String,
    },
}
