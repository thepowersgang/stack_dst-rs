//! Stack-based Dynamically-Sized Types
//!
//! The `StackDST` type provides a fixed size (7 word in the current version) buffer in which a trait object
//! or array can be stored, without resorting to a heap allocation.
//!
//! # Examples
//! ## An unboxed any
//! As a quick example - The following wraps a 64-bit integer up in a StackDST using the Any trait.
//!
//! ```
//! use stack_dst::StackDST;
//! use std::any::Any;
//!
//! let dst = StackDST::<Any>::new(1234u64).expect("Integer did not fit in allocation");
//! println!("dst as u64 = {:?}", dst.downcast_ref::<u64>());
//! println!("dst as i8 = {:?}", dst.downcast_ref::<i8>());
//! ```
//! 
//! ## Stack-allocated closure!
//! The following snippet shows how small (`'static`) closures can be returned using StackDST
//!
//! ```
//! use stack_dst::StackDST;
//! 
//! fn make_closure(value: u64) -> StackDST<FnMut()->String> {
//!     StackDST::new(move || format!("Hello there! value={}", value)).expect("Closure doesn't fit")
//! }
//! let mut closure = make_closure(666);
//! assert_eq!( (&mut *closure)(), "Hello there! value=666" );
//! ```
#![feature(core_intrinsics,unsize)]	// needed for intrinsics, raw, and Unsize
#![cfg_attr(no_std,feature(no_std,core,core_prelude,core_slice_ext))]
#![cfg_attr(no_std,no_std)]
#![crate_type="lib"]
#![crate_name="stack_dst"]

#[cfg(no_std)]
#[macro_use]
extern crate core;

#[cfg(no_std)]
use core::prelude::*;

#[cfg(not(no_std))]
use std::{ops,mem,intrinsics,slice,marker};

#[cfg(no_std)]
use core::{ops,mem,intrinsics,slice,marker};


const DST_SIZE: usize = 8;

/// Stack-allocated DST
pub struct StackDST<T: ?Sized>
{
	_pd: marker::PhantomData<T>,
	data: [usize; DST_SIZE],
}

unsafe fn ptr_as_slice<'p, T: ?Sized>(ptr: &'p mut &T) -> &'p mut [usize] {
	assert!( mem::size_of::<&T>() % mem::size_of::<usize>() == 0 );
	let words = mem::size_of::<&T>() / mem::size_of::<usize>();
	slice::from_raw_parts_mut(ptr as *mut &T as *mut usize, words)
}
unsafe fn as_ptr<T: ?Sized>(s: &StackDST<T>) -> *mut T {
	let mut ret: &T = mem::zeroed();
	{
		let ret_as_slice = ptr_as_slice(&mut ret);
		// 1. Data pointer
		ret_as_slice[0] = s.data[ret_as_slice.len()-1..].as_ptr() as usize;
		// 2. Pointer info
		for i in (1 .. ret_as_slice.len()) {
			ret_as_slice[i] = s.data[i-1];
		}
	}
	ret as *const _ as *mut _
}

impl<T: ?Sized> StackDST<T>
{
	/// Construct a stack-based DST
	pub fn new<U: marker::Unsize<T>>(val: U) -> Option<StackDST<T>> {
		let rv = unsafe {
			let mut ptr: &T = &val;
			let words = ptr_as_slice(&mut ptr);
			assert!(words[0] == &val as *const _ as usize, "BUG: Pointer layout is not (data, ...)");
			assert!(mem::min_align_of::<U>() <= mem::size_of::<usize>(), "TODO: Enforce alignment >{} (requires {})",
				mem::size_of::<usize>(), mem::min_align_of::<U>());
			
			StackDST::new_raw(&words[1..], words[0] as *mut (), mem::size_of::<U>())
			};
		// Prevent the destructor from running, now that we've copied it away
		mem::forget(val);
		rv
	}
	
	unsafe fn new_raw(info: &[usize], data: *mut (), size: usize) -> Option<StackDST<T>>
	{
		if info.len()*mem::size_of::<usize>() + size > mem::size_of::<[usize; DST_SIZE]>() {
			None
		}
		else {
			let mut rv = StackDST {
					_pd: marker::PhantomData,
					data: mem::zeroed(),
				};
			for i in (0 .. info.len()) {
				rv.data[i] = info[i];
			}
			
			let src_ptr = data as *const u8;
			let dataptr = rv.data[info.len()..].as_mut_ptr() as *mut u8;
			for i in (0 .. size) {
				*dataptr.offset(i as isize) = *src_ptr.offset(i as isize);
			}
			Some(rv)
		}
	}
}
impl<T: ?Sized> ops::Deref for StackDST<T> {
	type Target = T;
	fn deref(&self) -> &T {
		unsafe {
			&*as_ptr(self)
		}
	}
}
impl<T: ?Sized> ops::DerefMut for StackDST<T> {
	fn deref_mut(&mut self) -> &mut T {
		unsafe {
			&mut *as_ptr(self)
		}
	}
}
impl<T: ?Sized> ops::Drop for StackDST<T> {
	fn drop(&mut self) {
		unsafe {
			intrinsics::drop_in_place(&mut **self)
		}
	}
}
