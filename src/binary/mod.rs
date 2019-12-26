mod view;
pub use self::view::View;
use crate::imports::*;
use std::error::Error;

pub type Result<T> = std::result::Result<T, ReadError>;

/// A reason why we could not read enough data. These may be intermittent errors, for
/// streams that pull from network asynchronously, for instance it could signal that we
/// don't have enough data available yet.
pub enum ReadError {
    EndOfStream,
    Other(Box<dyn ViewReadError>),
}

/// A way to propagate custom read errors from a view, besides `EndOfStream`.
///
/// Because this is [`Any`], you can cast back into the original type.
pub trait ViewReadError: Any + Error {}
