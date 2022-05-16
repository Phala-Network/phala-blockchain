//! Networking support.

use std::future::Future;
use std::io::Error;
use std::pin::Pin;
use std::task::{Context, Poll};

use hyper::server::accept::Accept;
use tokio::io::{AsyncBufRead, AsyncRead, AsyncWrite};

use crate::env::{self, Result};
use crate::{ocall, ResourceId};

/// A TCP socket server, listening for connections.
pub struct TcpListener {
    res_id: ResourceId,
}

const TODO: &str = "Split the buf out and define a wrapper BufferedTcpStrem";
/// A connected TCP socket.
#[derive(Debug)]
pub struct TcpStream {
    res_id: ResourceId,
    buf: [u8; 32],
    filled: usize,
    start: usize,
}

struct Acceptor<'a> {
    listener: &'a TcpListener,
}

impl Future for Acceptor<'_> {
    type Output = Result<TcpStream>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        use env::OcallError;
        match ocall::tcp_accept(self.listener.res_id.0) {
            Ok(res_id) => Poll::Ready(Ok(TcpStream {
                res_id: ResourceId(res_id),
                buf: Default::default(),
                start: 0,
                filled: 0,
            })),
            Err(OcallError::Pending) => Poll::Pending,
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

impl TcpListener {
    /// Listen on the specified address for incoming TCP connections.
    pub async fn listen(addr: &str) -> Result<Self> {
        // Side notes: could be used to probe enabled interfaces and occupied ports. We may
        // consider to introduce some manifest file to further limit the capability in the future
        let todo = "prevent local interface probing and port occupation";
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
        let x = Pin::new(&mut accept).poll(cx).map(Some);
        log::info!("Poll accept = {:?}", x);
        x
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

impl AsyncBufRead for TcpStream {
    fn poll_fill_buf(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<std::io::Result<&[u8]>> {
        // When the `start` reaches `filled`, both are reset to zero.
        if self.filled == self.start {
            use env::OcallError;
            match ocall::poll_read(self.res_id.0, &mut self.buf[..]) {
                Err(OcallError::Pending) => return Poll::Pending,
                Err(err) => return Poll::Ready(Err(Error::from_raw_os_error(err as i32))),
                Ok(sz) => {
                    self.filled = sz as _;
                }
            }
        }
        let me = self.get_mut();
        Poll::Ready(Ok(&me.buf[me.start..me.filled]))
    }

    fn consume(mut self: Pin<&mut Self>, amt: usize) {
        self.start += amt;
        if self.start >= self.filled {
            self.start = 0;
            self.filled = 0;
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
            Ok(len) => Poll::Ready(Ok(len as _)),
            Err(env::OcallError::Pending) => Poll::Pending,
            Err(err) => Poll::Ready(Err(Error::from_raw_os_error(err as i32))),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        match ocall::poll_shutdown(self.res_id.0) {
            Ok(()) => Poll::Ready(Ok(())),
            Err(env::OcallError::Pending) => Poll::Pending,
            Err(err) => Poll::Ready(Err(Error::from_raw_os_error(err as i32))),
        }
    }
}
