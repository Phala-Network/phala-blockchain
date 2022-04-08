//! Networking support.

use std::future::Future;
use std::io::Error;
use std::pin::Pin;
use std::task::{Context, Poll};

use hyper::server::accept::Accept;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::env::{self, Result};
use crate::{ocall, ResourceId};

/// A TCP socket server, listening for connections.
pub struct TcpListener {
    res_id: ResourceId,
}

/// A connected TCP socket.
pub struct TcpStream {
    res_id: ResourceId,
}
struct Acceptor<'a> {
    listener: &'a TcpListener,
}

impl Future for Acceptor<'_> {
    type Output = Result<TcpStream>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let rv = match ocall::tcp_accept(self.listener.res_id.0) {
            Ok(rv) => rv,
            Err(err) => return Poll::Ready(Err(err)),
        };
        match rv {
            env::Poll::Ready(res_id) => Poll::Ready(Ok(TcpStream {
                res_id: ResourceId(res_id),
            })),
            env::Poll::Pending => Poll::Pending,
        }
    }
}

impl TcpListener {
    /// Listen on the specified address for incoming TCP connections.
    pub async fn listen(addr: &str) -> Result<Self> {
        let res_id = ResourceId(ocall::tcp_listen(addr.into(), 10)?);
        Ok(Self { res_id })
    }

    /// Accept a new incoming connection.
    pub async fn accept(&self) -> Result<TcpStream> {
        Acceptor { listener: self }.await
    }
}

impl Accept for TcpListener {
    type Conn = TcpStream;

    type Error = env::OcallError;

    fn poll_accept(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
        let mut accept = Acceptor {
            listener: self.get_mut(),
        };
        Pin::new(&mut accept).poll(cx).map(Some)
    }
}

impl AsyncRead for TcpStream {
    fn poll_read(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let result = {
            let size = buf.remaining().min(512);
            let buf = buf.initialize_unfilled_to(size);
            ocall::poll_read(self.res_id.0, buf)
        };
        match result {
            Ok(env::Poll::Pending) => Poll::Pending,
            Ok(env::Poll::Ready(len)) => {
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
            Err(err) => Poll::Ready(Err(Error::from_raw_os_error(err as i32))),
        }
    }
}

impl AsyncWrite for TcpStream {
    fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        match ocall::poll_write(self.res_id.0, buf) {
            Ok(env::Poll::Ready(len)) => Poll::Ready(Ok(len as _)),
            Ok(env::Poll::Pending) => Poll::Pending,
            Err(err) => Poll::Ready(Err(Error::from_raw_os_error(err as i32))),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        match ocall::poll_shutdown(self.res_id.0) {
            Ok(env::Poll::Ready(())) => Poll::Ready(Ok(())),
            Ok(env::Poll::Pending) => Poll::Pending,
            Err(err) => Poll::Ready(Err(Error::from_raw_os_error(err as i32))),
        }
    }
}
