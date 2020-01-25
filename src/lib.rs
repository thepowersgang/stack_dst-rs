//! Support for storing dynamically-sized types on the stack
//!
//! The `Value` type provides a fixed size (7 word in the current version) buffer in which a trait object
//! or array can be stored, without resorting to a heap allocation.
//!
//! # Examples
//! ## An unboxed any
//! As a quick example - The following wraps a 64-bit integer up in an inline DST using the Any trait.
//!
//! ```rust
//! use std::any::Any;
//! use stack_dst::Value;
//!
//! let dst = Value::<dyn Any>::new_stable(1234u64, |p| p as _).ok().expect("Integer did not fit in allocation");
//! println!("dst as u64 = {:?}", dst.downcast_ref::<u64>());
//! println!("dst as i8 = {:?}", dst.downcast_ref::<i8>());
//! ```
//! 
//! ## Stack-allocated closure!
//! The following snippet shows how small (`'static`) closures can be returned using this crate
//!
//! ```rust
//! # fn main() {
//! use stack_dst::Value;
//! 
//! fn make_closure(value: u64) -> Value<dyn FnMut()->String> {
//!     Value::new_stable(move || format!("Hello there! value={}", value), |p| p as _)
//!         .ok().expect("Closure doesn't fit")
//! }
//! let mut closure = make_closure(666);
//! assert_eq!( (&mut *closure)(), "Hello there! value=666" );
//! # }
//! ```
//!
//! # Features
//! ## `std` (default)
//! Enables the use of the standard library as a dependency
//! ## `alloc` (default)
//! Provides the `StackDstA::new_or_boxed` method (if `unsize` feature is active too)
//! ## `unsize` (optional)
//! Uses the nightly feature `unsize` to provide a more egonomic API (no need for the `|p| p` closures)
//!
#![cfg_attr(feature="unsize",feature(unsize))]	// needed for Unsize

#![cfg_attr(not(feature="std"),no_std)]
#![crate_type="lib"]
#![crate_name="stack_dst"]
#![deny(missing_docs)]
use std::{mem,slice};

#[cfg(feature="std")]
extern crate core;
#[cfg(not(feature="std"))]
mod std {
	pub use core::{ops,mem,slice,marker,ptr};
}
#[cfg(feature="alloc")]
extern crate alloc;

/// Trait used to represent a data buffer, typically you'll passs a `[usize; N]` array.
pub trait DataBuf: Copy+Default+AsMut<[usize]>+AsRef<[usize]> {
}
impl<T: Copy+Default+AsMut<[usize]>+AsRef<[usize]>> DataBuf for T {
}

pub use value::{ValueA,Value};
pub use stack::{StackA};

mod value;
mod stack;

/// Obtain mutable access to a pointer's words
fn ptr_as_slice<T>(ptr: &mut T) -> &mut [usize] {
	assert!( mem::size_of::<T>() % mem::size_of::<usize>() == 0 );
	let words = mem::size_of::<T>() / mem::size_of::<usize>();
	// SAFE: Points to valid memory (a raw pointer)
	unsafe {
		slice::from_raw_parts_mut(ptr as *mut _ as *mut usize, words)
	}
}

/// Re-construct a fat pointer
unsafe fn make_fat_ptr<T: ?Sized>(data_ptr: usize, meta_vals: &[usize]) -> *mut T {
	let mut rv = mem::MaybeUninit::<*mut T>::uninit();
	{
		let s = ptr_as_slice(&mut rv);
		s[0] = data_ptr;
		s[1..].copy_from_slice(meta_vals);
	}
	let rv = rv.assume_init();
	assert_eq!(rv as *const (), data_ptr as *const ());
	rv
}

fn round_to_words(len: usize) -> usize {
	(len + mem::size_of::<usize>()-1) / mem::size_of::<usize>()
}
