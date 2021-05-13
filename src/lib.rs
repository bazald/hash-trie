
#![no_std]

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

#[cfg(test)]
#[macro_use]
extern crate std;

#[macro_use]
mod result;
mod bit_indexed_array;
mod flag;
mod set;
mod node;
pub mod traits;

/// The only error you'll find is `NotFound`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum HashTrieError {
    /// The value was not found.
    NotFound,
}

pub use set::HashTrieSet as HashTrieSet;
