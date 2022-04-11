use pink_sidevm_env::{OcallError, Poll, Result};
use std::{io::ErrorKind, net::SocketAddr, pin::Pin, task::Poll::*};
use tokio::{io::AsyncWrite as _, net, sync::mpsc::Receiver, time::Sleep};
use Resource::*;

use crate::async_context::get_task_cx;

pub enum Resource {
    Sleep(Pin<Box<Sleep>>),
    ChannelRx(Receiver<Vec<u8>>),
    TcpListener(net::TcpListener),
    TcpStream {
        stream: net::TcpStream,
        remote_addr: SocketAddr,
    },
}

impl Resource {
    pub(crate) fn poll(&mut self) -> Result<Poll<Option<Vec<u8>>>> {
        use crate::async_context::poll_in_task_cx;

        match self {
            Sleep(handle) => match poll_in_task_cx(handle.as_mut()) {
                Ready(_) => Ok(Poll::Ready(None)),
                Pending => Ok(Poll::Pending),
            },
            ChannelRx(rx) => {
                let fut = rx.recv();
                futures::pin_mut!(fut);
                Ok(poll_in_task_cx(fut).into())
            }
            _ => Err(OcallError::UnsupportedOperation),
        }
    }

    pub(crate) fn poll_read(&mut self, buf: &mut [u8]) -> Result<Poll<u32>> {
        match self {
            Sleep(_) => self.poll().map(|state| match state {
                Poll::Pending => Poll::Pending,
                Poll::Ready(_) => Poll::Ready(0),
            }),
            TcpStream { stream, .. } => loop {
                match stream.try_read(buf) {
                    Ok(sz) => break Ok(Poll::Ready(sz as _)),
                    Err(err) => {
                        if err.kind() == ErrorKind::WouldBlock {
                            match get_task_cx(|cx| stream.poll_read_ready(cx)) {
                                Pending => break Ok(Poll::Pending),
                                Ready(Err(_err)) => break Err(OcallError::IoError),
                                Ready(Ok(())) => continue,
                            }
                        } else {
                            break Err(OcallError::IoError);
                        }
                    }
                }
            },
            _ => Err(OcallError::UnsupportedOperation),
        }
    }

    pub(crate) fn poll_write(&mut self, buf: &[u8]) -> Result<Poll<u32>> {
        match self {
            TcpStream { stream, .. } => loop {
                match stream.try_write(buf) {
                    Ok(sz) => break Ok(Poll::Ready(sz as _)),
                    Err(err) => {
                        if err.kind() == ErrorKind::WouldBlock {
                            match get_task_cx(|cx| stream.poll_write_ready(cx)) {
                                Pending => break Ok(Poll::Pending),
                                Ready(Err(_err)) => break Err(OcallError::IoError),
                                Ready(Ok(())) => continue,
                            }
                        } else {
                            break Err(OcallError::IoError);
                        }
                    }
                }
            },
            _ => Err(OcallError::UnsupportedOperation),
        }
    }

    pub(crate) fn poll_shutdown(&mut self) -> Result<Poll<()>> {
        match self {
            TcpStream { stream, .. } => {
                let stream = Pin::new(stream);
                match get_task_cx(|cx| stream.poll_shutdown(cx)) {
                    Pending => Ok(Poll::Pending),
                    Ready(Err(_err)) => Err(OcallError::IoError),
                    Ready(Ok(())) => Ok(Poll::Ready(())),
                }
            }
            _ => Err(OcallError::UnsupportedOperation),
        }
    }
}

#[derive(Default)]
pub struct ResourceKeeper {
    resources: Vec<Option<Resource>>,
}

const RESOURCE_ID_MAX: usize = 8192;

impl ResourceKeeper {
    pub fn get_mut(&mut self, id: i32) -> Result<&mut Resource> {
        self.resources
            .get_mut(id as usize)
            .and_then(Option::as_mut)
            .ok_or(OcallError::ResourceNotFound)
    }

    pub fn push(&mut self, resource: Resource) -> Result<i32> {
        for (i, res) in self.resources.iter_mut().enumerate() {
            if res.is_none() {
                let id = i.try_into().or(Err(OcallError::ResourceLimited))?;
                *res = Some(resource);
                return Ok(id);
            }
        }
        if self.resources.len() >= RESOURCE_ID_MAX.min(i32::MAX as _) {
            return Err(OcallError::ResourceLimited);
        }
        let id = self
            .resources
            .len()
            .try_into()
            .or(Err(OcallError::ResourceLimited))?;
        self.resources.push(Some(resource));
        Ok(id)
    }

    pub fn take(&mut self, resource_id: i32) -> Option<Resource> {
        let resource_id = resource_id as u32 as usize;
        if resource_id >= self.resources.len() {
            return None;
        }
        self.resources[resource_id].take()
    }
}
