//! Networking support.

use std::future::Future;
use std::task::{Poll, Context};
use std::pin::Pin;

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

impl TcpListener {
    /// Listen on the specified address for incoming TCP connections.
    pub async fn listen(addr: &str) -> Result<Self> {
        let res_id = ResourceId(ocall::tcp_listen(addr.into(), 10)?);
        Ok(Self { res_id })
    }

    /// Accept a new incoming connection.
    pub async fn accept(&self) -> Result<TcpStream> {
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

        Acceptor { listener: self }.await
    }
}
