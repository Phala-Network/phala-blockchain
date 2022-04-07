/// Resource ID. Think of it as a FD.
pub struct ResourceId(pub i32);

impl From<i32> for ResourceId {
    fn from(i: i32) -> Self {
        ResourceId(i)
    }
}

impl Drop for ResourceId {
    fn drop(&mut self) {
        let _ = crate::ocall::close(self.0);
    }
}
