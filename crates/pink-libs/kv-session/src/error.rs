#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    FailedToGetStorage,
    FailedToDecode,
}

pub type Result<T> = core::result::Result<T, Error>;
