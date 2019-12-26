use super::{ReadError::EndOfStream, Result};
use crate::imports::*;
use smallvec::SmallVec;
use std::cmp::min;
use std::convert::TryFrom;

/// A view on immutable binary data.
///
/// Views tend to be only valid for a segment. You can never navigate back. You can
/// "navigate forward", which really just restricts the view by making the first
/// couple of bytes inaccessible.
///
/// Reading may have side effects. For instance, if the view comes from a network
/// stream, then this may tell the remote end which segment was now requested.
///
/// Many of these methods return a [`Result`], which can be used to tell the caller
/// to re-try later (perhaps when a certain promise is fulfilled).
///
/// Transcribe amounts are given in [`usize`], because the result must fit into
/// memory. Skip amounts are given in [`u64`], because the underlying file may be
/// larger than memory.
pub trait View: Clone {
    /// Try to read a single byte. This does not change the position of the view.
    ///
    /// This should be the same as `.transcribe(0).map(|vec| vec[0])`.
    ///
    /// If we cannot read the full range right away, then return failure.
    ///
    fn read_byte(&self) -> Result<u8>;

    /// Try to read a range of bytes. If we cannot read the full range right away,
    /// then return failure.
    ///
    /// Beware of transcribing too carelessly. Transcribing allocates memory.
    fn transcribe(&self, len: usize) -> Result<SmallVecU8>;

    /// Create a new view that skips a number of bytes.
    ///
    /// Should only return Err for permanent failures, such as when we try to skip
    /// "strictly past" the end of a file on disk. Skipping to the end of a file is
    /// legal.
    ///
    /// May have side effects. For instance, for views that fetch from disk or a
    /// network resource, this may trigger a fetch.
    fn skip(self, bytes: u64) -> Result<Self>;

    /// Create a new view that can read no more than `len` bytes (from here). There is no
    /// guarantee that we will can actually read `len` bytes.
    ///
    /// If this view is already bound, then this function cannot loosen the bound.
    fn bound(self, len: u64) -> Self;

    /// If Some, then we know that there are at maximum so many bytes in this view, and
    /// reading or skipping past that will be an error. There is no guarantee that so
    /// many bytes will actually be available.
    fn bound_len(&self) -> Option<usize>;

    /// Return the number of bytes that are immediately available, if such knowledge is
    /// available. This may never be more than bound_len.
    fn hint_available_bytes(&self) -> Option<usize> {
        None
    }
}

pub type SmallVecU8 = smallvec::SmallVec<[u8; 8]>;

#[derive(Debug, Clone)]
pub struct BorrowView<T: Clone + Borrow<[u8]>> {
    handle: T,
    offset: usize,               // Can never skip beyond memory
    bound_offset: Option<usize>, // Offset of first byte that is not allowed to be read
}

impl<T: Clone + Borrow<[u8]>> BorrowView<T> {
    pub fn new(handle: T) -> Self {
        Self::new_offset(handle, 0)
    }

    pub fn new_offset(handle: T, offset: usize) -> Self {
        BorrowView {
            handle,
            offset,
            bound_offset: None,
        }
    }
}

impl<T: Clone + Borrow<[u8]>> Borrow<[u8]> for BorrowView<T> {
    fn borrow(&self) -> &[u8] {
        self.handle.borrow()
    }
}

impl<T: Clone + Borrow<[u8]>> View for BorrowView<T> {
    fn read_byte(&self) -> Result<u8> {
        self.handle
            .borrow()
            .get(self.offset)
            .copied()
            .ok_or(EndOfStream)
    }
    fn transcribe(&self, len: usize) -> Result<SmallVecU8> {
        let slice = self
            .handle
            .borrow()
            .get(self.offset..self.offset + len)
            .ok_or(EndOfStream)?;
        Ok(SmallVec::from_slice(slice))
    }
    fn skip(self, bytes: u64) -> Result<Self> {
        let bytes2: usize = usize::try_from(bytes).map_err(|_| EndOfStream)?;
        let new_offset = self.offset.saturating_add(bytes2);
        if let Some(bound) = self.bound_offset {
            if new_offset > bound {
                return Err(EndOfStream);
            }
        }
        Ok(BorrowView {
            handle: self.handle,
            offset: new_offset,
            bound_offset: self.bound_offset,
        })
    }

    fn bound(mut self, len: u64) -> Self {
        match (self.bound_offset, usize::try_from(len)) {
            (_, Err(_)) => {} // bound past memory does nothing

            (None, Ok(len2)) => self.bound_offset = Some(self.offset.saturating_add(len2)),

            (Some(cur), Ok(len2)) => {
                let suggested_bound_offset = self.offset.saturating_add(len2);
                self.bound_offset = Some(min(cur, suggested_bound_offset))
            }
        }
        self
    }

    fn bound_len(&self) -> Option<usize> {
        Some(self.handle.borrow().len().saturating_sub(self.offset))
    }
    fn hint_available_bytes(&self) -> Option<usize> {
        Some(self.handle.borrow().len().saturating_sub(self.offset))
    }
}

#[cfg(test)]
mod test {
    // todo
}
