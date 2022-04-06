use pink_sidevm_env::{OcallError, Poll, Result};
use std::{pin::Pin, task::Poll::*};
use tokio::{sync::mpsc::Receiver, time::Sleep};
use Resource::*;

pub enum Resource {
    Sleep(Pin<Box<Sleep>>),
    ChannelRx(Receiver<Vec<u8>>),
}

impl Resource {
    pub(crate) fn poll(&mut self, task_id: i32) -> Result<Poll<Option<Vec<u8>>>> {
        use crate::async_context::poll_in_task_cx;

        match self {
            Sleep(handle) => match poll_in_task_cx(handle.as_mut(), task_id) {
                Ready(_) => Ok(Poll::Ready(None)),
                Pending => Ok(Poll::Pending),
            },
            ChannelRx(rx) => {
                let fut = rx.recv();
                futures::pin_mut!(fut);
                Ok(poll_in_task_cx(fut, task_id).into())
            }
        }
    }

    pub(crate) fn poll_read(&mut self, task_id: i32, _buf: &mut [u8]) -> Result<Poll<u32>> {
        match self {
            Sleep(_) => self.poll(task_id).map(|state| match state {
                Poll::Pending => Poll::Pending,
                Poll::Ready(_) => Poll::Ready(0),
            }),
            _ => {
                return Err(OcallError::UnsupportedOperation);
            }
        }
    }

    pub(crate) fn poll_write(&mut self, _task_id: i32, _buf: &[u8]) -> Result<Poll<u32>> {
        return Err(OcallError::UnsupportedOperation);
    }
}

#[derive(Default)]
pub struct ResourceKeeper {
    resources: Vec<Option<Resource>>,
}

const RESOURCE_ID_MAX: usize = 8192;

impl ResourceKeeper {
    pub fn get_mut(&mut self, id: i32) -> Option<&mut Resource> {
        self.resources.get_mut(id as usize)?.as_mut()
    }

    pub fn push(&mut self, resource: Resource) -> Option<i32> {
        for (i, res) in self.resources.iter_mut().enumerate() {
            if res.is_none() {
                let id = i.try_into().ok()?;
                *res = Some(resource);
                return Some(id);
            }
        }
        if self.resources.len() >= RESOURCE_ID_MAX.min(i32::MAX as _) {
            return None;
        }
        let id = self.resources.len().try_into().ok()?;
        self.resources.push(Some(resource));
        Some(id)
    }

    pub fn take(&mut self, resource_id: i32) -> Option<Resource> {
        let resource_id = resource_id as u32 as usize;
        if resource_id >= self.resources.len() {
            return None;
        }
        self.resources[resource_id].take()
    }
}
