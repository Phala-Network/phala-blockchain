use std::future::Future;
use std::io;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use futures::ready;
use pink_sidevm_env::tls::TlsServerConfig;
use pink_sidevm_env::OcallError;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream as TokioTlsStream;
use tokio_rustls::Accept;
use tokio_rustls::{
    rustls::{self, ServerConfig},
    TlsAcceptor,
};

pub enum TlsStream {
    Handshaking(Accept<TcpStream>),
    Streaming(TokioTlsStream<TcpStream>),
}

impl TlsStream {
    pub(crate) fn new(stream: TcpStream, config: Arc<ServerConfig>) -> TlsStream {
        let accept = TlsAcceptor::from(config).accept(stream);
        TlsStream::Handshaking(accept)
    }
}

impl AsyncRead for TlsStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut ReadBuf,
    ) -> Poll<io::Result<()>> {
        let me = self.get_mut();
        match me {
            Self::Handshaking(ref mut accept) => match ready!(Pin::new(accept).poll(cx)) {
                Ok(mut stream) => {
                    let result = Pin::new(&mut stream).poll_read(cx, buf);
                    *me = Self::Streaming(stream);
                    result
                }
                Err(err) => Poll::Ready(Err(err)),
            },
            Self::Streaming(ref mut stream) => Pin::new(stream).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for TlsStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let me = self.get_mut();
        match me {
            Self::Handshaking(ref mut accept) => match ready!(Pin::new(accept).poll(cx)) {
                Ok(mut stream) => {
                    let result = Pin::new(&mut stream).poll_write(cx, buf);
                    *me = Self::Streaming(stream);
                    result
                }
                Err(err) => Poll::Ready(Err(err)),
            },
            Self::Streaming(ref mut stream) => Pin::new(stream).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.get_mut() {
            Self::Handshaking(_) => Poll::Ready(Ok(())),
            Self::Streaming(ref mut stream) => Pin::new(stream).poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.get_mut() {
            Self::Handshaking(_) => Poll::Ready(Ok(())),
            Self::Streaming(ref mut stream) => Pin::new(stream).poll_shutdown(cx),
        }
    }
}

pub(crate) fn load_tls_config(config: TlsServerConfig) -> Result<ServerConfig, OcallError> {
    let (cert_pem, key_pem) = match &config {
        TlsServerConfig::V0 { cert, key } => (cert, key),
    };

    let certs = load_certs(cert_pem)?;
    let key = load_private_key(key_pem)?;

    tokio_rustls::rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .or(Err(OcallError::InvalidParameter))
}

fn load_certs(pem_str: &str) -> Result<Vec<rustls::Certificate>, OcallError> {
    let certs =
        rustls_pemfile::certs(&mut pem_str.as_bytes()).or(Err(OcallError::InvalidParameter))?;
    Ok(certs.into_iter().map(rustls::Certificate).collect())
}

fn load_private_key(pem_str: &str) -> Result<rustls::PrivateKey, OcallError> {
    let keys = rustls_pemfile::pkcs8_private_keys(&mut pem_str.as_bytes())
        .or(Err(OcallError::InvalidParameter))?;
    if keys.len() != 1 {
        return Err(OcallError::InvalidParameter);
    }
    Ok(rustls::PrivateKey(keys[0].clone()))
}
