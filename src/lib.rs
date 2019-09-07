//! Support for storing dynamically-sized types on the stack
//!
//! The `StackDST` type provides a fixed size (7 word in the current version) buffer in which a trait object
//! or array can be stored, without resorting to a heap allocation.
//!
//! # Examples
//! ## An unboxed any
//! As a quick example - The following wraps a 64-bit integer up in a StackDST using the Any trait.
//!
//! ```rust
//! use std::any::Any;
//! type StackDST<T> = stack_dst::Value<T>;
//!
//! let dst = StackDST::<Any>::new(1234u64).ok().expect("Integer did not fit in allocation");
//! println!("dst as u64 = {:?}", dst.downcast_ref::<u64>());
//! println!("dst as i8 = {:?}", dst.downcast_ref::<i8>());
//! ```
//! 
//! ## Stack-allocated closure!
//! The following snippet shows how small (`'static`) closures can be returned using StackDST
//!
//! ```rust
//! # fn main() {
//! type StackDST<T> = stack_dst::Value<T>;
//! 
//! fn make_closure(value: u64) -> StackDST<FnMut()->String> {
//!     StackDST::new(move || format!("Hello there! value={}", value)).ok().expect("Closure doesn't fit")
//! }
//! let mut closure = make_closure(666);
//! assert_eq!( (&mut *closure)(), "Hello there! value=666" );
//! # }
//! ```
#![feature(unsize)]	// needed for Unsize

#![cfg_attr(not(feature="std"),no_std)]
#![crate_type="lib"]
#![crate_name="stack_dst"]
#![deny(missing_docs)]
use std::{mem,slice};

#[cfg(not(feature="std"))]
mod std {
	pub use core::{ops,mem,slice,marker,ptr};
}

/// Trait used to represent the data buffer for StackDSTA.
/// 
/// Typically you'll passs a [usize; N] array
pub trait DataBuf: Copy+Default+AsMut<[usize]>+AsRef<[usize]> {
}
impl<T: Copy+Default+AsMut<[usize]>+AsRef<[usize]>> DataBuf for T {
}

pub use value::{ValueA,Value};
pub use stack::{StackA};

mod value;
mod stack;

/// Obtain mutable access to a pointer's words
fn ptr_as_slice<T: ?Sized>(ptr: &mut *const T) -> &mut [usize] {
	assert!( mem::size_of::<&T>() % mem::size_of::<usize>() == 0 );
	let words = mem::size_of::<&T>() / mem::size_of::<usize>();
	// SAFE: Points to valid memory (a raw pointer)
	unsafe {
		slice::from_raw_parts_mut(ptr as *mut _ as *mut usize, words)
	}
}
