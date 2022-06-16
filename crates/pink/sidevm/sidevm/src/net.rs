//! Networking support.

use std::future::Future;
use std::io::Error;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};

use env::tls::TlsServerConfig;

use crate::env::{self, tasks, Result};
use crate::{ocall, ResourceId};

/// A TCP socket server, listening for connections.
pub struct TcpListener {
    res_id: ResourceId,
}

/// A resource pointing to a connecting TCP socket.
#[derive(Debug)]
pub struct TcpConnector {
    res: Result<ResourceId, env::OcallError>,
}

/// A connected TCP socket.
#[derive(Debug)]
pub struct TcpStream {
    res_id: ResourceId,
}

/// Future returned by `TcpListener::accept`.
pub struct Acceptor<'a> {
    listener: &'a TcpListener,
}

impl Future for Acceptor<'_> {
    type Output = Result<(TcpStream, SocketAddr)>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        use env::OcallError;
        let waker_id = tasks::intern_waker(cx.waker().clone());
        match ocall::tcp_accept(waker_id, self.listener.res_id.0) {
            Ok((res_id, remote_addr)) => Poll::Ready(Ok((
                TcpStream::new(ResourceId(res_id)),
                remote_addr
                    .parse()
                    .expect("ocall::tcp_accept returned an invalid remote address"),
            ))),
            Err(OcallError::Pending) => Poll::Pending,
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

impl TcpListener {
    /// Bind and listen on the specified address for incoming TCP connections.
    pub async fn bind(addr: &str) -> Result<Self> {
        // Side notes: could be used to probe enabled interfaces and occupied ports. We may
        // consider to introduce some manifest file to further limit the capability in the future
        let todo = "prevent local interface probing and port occupation";
        let res_id = ResourceId(ocall::tcp_listen(addr.into(), None)?);
        Ok(Self { res_id })
    }

    /// Bind and listen on the specified address for incoming TLS-enabled TCP connections.
    pub async fn bind_tls(addr: &str, config: TlsServerConfig) -> Result<Self> {
        let res_id = ResourceId(ocall::tcp_listen(addr.into(), Some(config))?);
        Ok(Self { res_id })
    }

    /// Accept a new incoming connection.
    pub fn accept(&self) -> Acceptor {
        Acceptor { listener: self }
    }
}

impl Future for TcpConnector {
    type Output = Result<TcpStream>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        use env::OcallError;

        let res_id = match &self.get_mut().res {
            Ok(res_id) => res_id,
            Err(err) => return Poll::Ready(Err(*err)),
        };

        match ocall::poll_res(env::tasks::intern_waker(ctx.waker().clone()), res_id.0) {
            Ok(res_id) => Poll::Ready(Ok(TcpStream::new(ResourceId(res_id)))),
            Err(OcallError::Pending) => Poll::Pending,
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

impl TcpStream {
    fn new(res_id: ResourceId) -> Self {
        Self { res_id }
    }

    /// Initiate a TCP connection to a remote host.
    pub fn connect(host: &str, port: u16, enable_tls: bool) -> TcpConnector {
        let res = if enable_tls {
            ocall::tcp_connect_tls(host.into(), port, env::tls::TlsClientConfig::V0)
        } else {
            ocall::tcp_connect(host, port)
        };
        let res = res.map(|res_id| ResourceId(res_id));
        TcpConnector { res }
    }
}

#[cfg(feature = "hyper")]
pub use impl_hyper::{AddrIncoming, AddrStream, HttpConnector};
#[cfg(feature = "hyper")]
mod impl_hyper {
    use super::*;
    use env::OcallError;
    use hyper::client::connect::{Connected, Connection};
    use hyper::server::accept::Accept;
    use hyper::{service::Service, Uri};
    use std::{io, task};

    macro_rules! ready_ok {
        ($poll: expr) => {
            match $poll {
                Poll::Ready(Ok(val)) => val,
                Poll::Ready(Err(OcallError::EndOfFile)) => return Poll::Ready(None),
                Poll::Ready(Err(err)) => return Poll::Ready(Some(Err(err))),
                Poll::Pending => return Poll::Pending,
            }
        };
    }

    impl Accept for TcpListener {
        type Conn = TcpStream;

        type Error = env::OcallError;

        fn poll_accept(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
            let (conn, _addr) = ready_ok!(Pin::new(&mut self.accept()).poll(cx));
            Poll::Ready(Some(Ok(conn)))
        }
    }

    impl Connection for TcpStream {
        fn connected(&self) -> Connected {
            Connected::new()
        }
    }

    impl TcpListener {
        /// Convert the listener into another one that outputs AddrStreams.
        pub fn into_addr_incoming(self) -> AddrIncoming {
            AddrIncoming { listener: self }
        }
    }

    /// An HTTP/HTTPS Connector for hyper working under sidevm.
    #[derive(Clone, Default, Debug)]
    pub struct HttpConnector;

    impl HttpConnector {
        /// Create a new HttpConnector.
        pub fn new() -> Self {
            Self
        }
    }

    impl Service<Uri> for HttpConnector {
        type Response = TcpStream;
        type Error = OcallError;
        type Future = TcpConnector;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, dst: Uri) -> Self::Future {
            let is_https = dst.scheme_str() == Some("https");
            let host = dst
                .host()
                .unwrap_or("")
                .trim_matches(|c| c == '[' || c == ']');
            let port = dst.port_u16().unwrap_or(if is_https { 443 } else { 80 });
            TcpStream::connect(host, port, is_https)
        }
    }

    /// A wrapper of  TcpListener that outputs AddrStreams.
    pub struct AddrIncoming {
        listener: TcpListener,
    }

    impl Accept for AddrIncoming {
        type Conn = AddrStream;
        type Error = env::OcallError;

        fn poll_accept(
            self: Pin<&mut Self>,
            cx: &mut task::Context<'_>,
        ) -> task::Poll<Option<Result<Self::Conn, Self::Error>>> {
            let (stream, remote_addr) = ready_ok!(Pin::new(&mut self.listener.accept()).poll(cx));
            Poll::Ready(Some(Ok(AddrStream {
                stream,
                remote_addr,
            })))
        }
    }

    /// A wrapper of TcpStream that keep remote address.
    #[pin_project::pin_project]
    pub struct AddrStream {
        #[pin]
        stream: TcpStream,
        remote_addr: SocketAddr,
    }

    impl AddrStream {
        /// Get the remote address of the connection.
        pub fn remote_addr(&self) -> SocketAddr {
            self.remote_addr
        }
    }

    #[cfg(feature = "tokio")]
    const _: () = {
        use tokio::io::{AsyncRead, AsyncWrite};
        impl AsyncRead for AddrStream {
            fn poll_read(
                self: Pin<&mut Self>,
                cx: &mut task::Context<'_>,
                buf: &mut tokio::io::ReadBuf<'_>,
            ) -> task::Poll<io::Result<()>> {
                self.project().stream.poll_read(cx, buf)
            }
        }

        impl AsyncWrite for AddrStream {
            fn poll_write(
                self: Pin<&mut Self>,
                cx: &mut task::Context<'_>,
                buf: &[u8],
            ) -> task::Poll<io::Result<usize>> {
                self.project().stream.poll_write(cx, buf)
            }

            fn poll_flush(
                self: Pin<&mut Self>,
                cx: &mut task::Context<'_>,
            ) -> task::Poll<io::Result<()>> {
                self.project().stream.poll_flush(cx)
            }

            fn poll_shutdown(
                self: Pin<&mut Self>,
                cx: &mut task::Context<'_>,
            ) -> task::Poll<io::Result<()>> {
                self.project().stream.poll_shutdown(cx)
            }
        }
    };

    const _: () = {
        use futures::{AsyncRead, AsyncWrite};

        impl AsyncRead for AddrStream {
            fn poll_read(
                self: Pin<&mut Self>,
                cx: &mut Context<'_>,
                buf: &mut [u8],
            ) -> Poll<io::Result<usize>> {
                self.project().stream.poll_read(cx, buf)
            }
        }

        impl AsyncWrite for AddrStream {
            fn poll_write(
                self: Pin<&mut Self>,
                cx: &mut Context<'_>,
                buf: &[u8],
            ) -> Poll<io::Result<usize>> {
                self.project().stream.poll_write(cx, buf)
            }

            fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
                self.project().stream.poll_flush(cx)
            }

            fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
                self.project().stream.poll_close(cx)
            }
        }
    };
}

#[cfg(feature = "tokio")]
mod impl_tokio {
    use super::*;
    use tokio::io::{AsyncRead, AsyncWrite};

    impl AsyncRead for TcpStream {
        fn poll_read(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut tokio::io::ReadBuf<'_>,
        ) -> Poll<std::io::Result<()>> {
            let result = {
                let size = buf.remaining().min(512);
                let buf = buf.initialize_unfilled_to(size);
                let waker_id = tasks::intern_waker(cx.waker().clone());
                ocall::poll_read(waker_id, self.res_id.0, buf)
            };
            use env::OcallError;
            match result {
                Ok(len) => {
                    let len = len as usize;
                    if len > buf.remaining() {
                        Poll::Ready(Err(Error::from_raw_os_error(
                            env::OcallError::InvalidEncoding as i32,
                        )))
                    } else {
                        buf.advance(len);
                        Poll::Ready(Ok(()))
                    }
                }
                Err(OcallError::Pending) => Poll::Pending,
                Err(err) => Poll::Ready(Err(Error::from_raw_os_error(err as i32))),
            }
        }
    }

    impl AsyncWrite for TcpStream {
        fn poll_write(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<Result<usize, Error>> {
            let waker_id = tasks::intern_waker(cx.waker().clone());
            into_poll(ocall::poll_write(waker_id, self.res_id.0, buf).map(|len| len as usize))
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
            Poll::Ready(Ok(()))
        }

        fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
            let waker_id = tasks::intern_waker(cx.waker().clone());
            into_poll(ocall::poll_shutdown(waker_id, self.res_id.0))
        }
    }
}

mod impl_futures_io {
    use super::*;
    use futures::io::{AsyncRead, AsyncWrite};

    impl AsyncRead for TcpStream {
        fn poll_read(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut [u8],
        ) -> Poll<std::io::Result<usize>> {
            let waker_id = tasks::intern_waker(cx.waker().clone());
            into_poll(ocall::poll_read(waker_id, self.res_id.0, buf).map(|len| len as usize))
        }
    }

    impl AsyncWrite for TcpStream {
        fn poll_write(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<std::io::Result<usize>> {
            let waker_id = tasks::intern_waker(cx.waker().clone());
            into_poll(ocall::poll_write(waker_id, self.res_id.0, buf).map(|len| len as usize))
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }

        fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            let waker_id = tasks::intern_waker(cx.waker().clone());
            into_poll(ocall::poll_shutdown(waker_id, self.res_id.0))
        }
    }
}

fn into_poll<T>(res: Result<T, env::OcallError>) -> Poll<std::io::Result<T>> {
    match res {
        Ok(v) => Poll::Ready(Ok(v)),
        Err(env::OcallError::Pending) => Poll::Pending,
        Err(err) => Poll::Ready(Err(std::io::Error::from_raw_os_error(err as i32))),
    }
}
