use std::pin::Pin;
use tokio::time::Sleep;

pub enum Resource {
    Empty,
    Sleep(Pin<Box<Sleep>>),
}

#[derive(Default)]
pub struct ResourceKeeper {
    resources: Vec<Resource>,
}

impl ResourceKeeper {
    pub fn get_mut(&mut self, id: usize) -> Option<&mut Resource> {
        self.resources.get_mut(id)
    }

    pub fn push(&mut self, resource: Resource) -> Option<usize> {
        self.resources.push(resource);
        Some(self.resources.len() - 1)
    }
}
