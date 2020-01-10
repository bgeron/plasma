mod view;
pub use self::view::{BorrowView, View};
use crate::imports::*;

pub type Result<T> = std::result::Result<T, ReadError>;

/// A reason why we could not read enough data. These may be intermittent errors, for
/// streams that pull from network asynchronously, for instance it could signal that we
/// don't have enough data available yet.
#[derive(Debug)]
pub enum ReadError {
    EndOfStream,
    Other(Box<dyn ViewReadError>),
}

use ReadError::EndOfStream;

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EndOfStream => write!(f, "end of stream"),
            ReadError::Other(other) => write!(f, "other: {}", other),
        }
    }
}

impl Error for ReadError {}

/// Convenient for tests. Two `Other` values are never equal.
impl std::cmp::PartialEq for ReadError {
    fn eq(&self, other: &Self) -> bool {
        match self {
            EndOfStream => match other {
                EndOfStream => true,
                _ => false,
            },
            _ => false,
        }
    }
}

/// A way to propagate custom read errors from a view, besides `EndOfStream`.
///
/// Because this is [`Any`], you can cast back into the original type.
pub trait ViewReadError: Any + Error {}

/// Similar to `Deref<Target=[u8]>`, but with convenient automatic impls.
pub trait DerefU8 {
    fn deref_u8(&self) -> &[u8];
}
impl DerefU8 for [u8] {
    fn deref_u8(&self) -> &[u8] {
        self
    }
}
impl DerefU8 for Vec<u8> {
    fn deref_u8(&self) -> &[u8] {
        self
    }
}
impl<T: DerefU8 + ?Sized> DerefU8 for &T {
    fn deref_u8(&self) -> &[u8] {
        self.deref().deref_u8()
    }
}
impl<T: DerefU8 + ?Sized> DerefU8 for Box<T> {
    fn deref_u8(&self) -> &[u8] {
        (**self).deref_u8()
    }
}
impl<T: DerefU8 + ?Sized> DerefU8 for Rc<T> {
    fn deref_u8(&self) -> &[u8] {
        self.deref().deref_u8()
    }
}
impl<T: DerefU8 + ?Sized> DerefU8 for Arc<T> {
    fn deref_u8(&self) -> &[u8] {
        self.deref().deref_u8()
    }
}
