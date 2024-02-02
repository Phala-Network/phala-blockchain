use scale::{Decode, Encode};
use scale_info::TypeInfo;
use std::result::Result as StdResult;

/// A custom `Result` type that used by pink extension to avoid auto error handling by ink! macro.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Encode, Decode, TypeInfo, Hash)]
pub enum Result<T, E> {
    Ok(T),
    Err(E),
}

impl<T, E> From<StdResult<T, E>> for Result<T, E> {
    fn from(r: StdResult<T, E>) -> Self {
        match r {
            Ok(v) => Result::Ok(v),
            Err(e) => Result::Err(e),
        }
    }
}

impl<T, E> From<Result<T, E>> for StdResult<T, E> {
    fn from(r: Result<T, E>) -> Self {
        match r {
            Result::Ok(v) => Ok(v),
            Result::Err(e) => Err(e),
        }
    }
}

impl<T, E> Result<T, E> {
    pub fn ok(self) -> Option<T> {
        match self {
            Result::Ok(v) => Some(v),
            Result::Err(_) => None,
        }
    }

    pub fn err(self) -> Option<E> {
        match self {
            Result::Ok(_) => None,
            Result::Err(e) => Some(e),
        }
    }

    pub fn into_std(self) -> StdResult<T, E> {
        self.into()
    }
}
