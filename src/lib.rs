//! Stack-based Dynamically-Sized Types
//!
//! The `StackDST` type provides a fixed size (7 word in the current version) buffer in which a trait object
//! or array can be stored, without resorting to a heap allocation.
//!
//! # Examples
//! ## An unboxed any
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
//! assert_eq!( closure(), "Hello there! value=666" );
//! ```
#![feature(core)]	// needed for intrinsics, raw, and Unsize
#![cfg_attr(no_std,feature(no_std))]
#![cfg_attr(no_std,no_std)]
#![crate_type="lib"]
#![crate_name="stack_dst"]

#[cfg(no_std)]
#[macro_use]
extern crate core;

#[cfg(no_std)]
use core::prelude::*;

#[cfg(not(no_std))]
use std::{ops,mem,intrinsics,slice,raw,marker};

#[cfg(no_std)]
use core::{ops,mem,intrinsics,slice,raw,marker};


const DST_SIZE: usize = 7;

/// Stack-allocated DST
pub struct StackDST<T: ?Sized>
{
	_pd: marker::PhantomData<T>,
	vtable: *mut (),
	data: [usize; DST_SIZE],
}

impl<T: ?Sized> StackDST<T>
{
	/// Construct a stack-based DST
	pub fn new<U: marker::Unsize<T>>(val: U) -> Option<StackDST<T>> {
		let rv = unsafe {
			let mut ptr: &T = &val;
			let words = Self::ptr_as_slice(&mut ptr);
			if words.len() != 2 {
				//error!("StackDST with != 2 word pointers (len={})", words.len());
				None
			}
			else {
				let to_p = words.as_ptr() as *const raw::TraitObject;
				let raw::TraitObject { data, vtable } = *to_p;
				StackDST::new_raw(vtable, data, mem::size_of::<U>())
			}
			};
		// Prevent the destructor from running, now that we've copied it away
		mem::forget(val);
		rv
	}
	
	unsafe fn new_raw(vtable: *mut (), data: *mut (), size: usize) -> Option<StackDST<T>>
	{
		if size > mem::size_of::<[usize; DST_SIZE]>() {
			None
		}
		else {
			let mut rv = StackDST {
					_pd: marker::PhantomData,
					vtable: vtable,
					data: mem::zeroed(),
				};
			let src_ptr = data as *const u8;
			let dataptr = &mut rv.data as *mut _ as *mut u8;
			for i in (0 .. size) {
				*dataptr.offset(i as isize) = *src_ptr.offset(i as isize);
			}
			Some(rv)
		}
	}

	unsafe fn ptr_as_slice<'p>(ptr: &'p mut &T) -> &'p mut [usize] {
		assert!( mem::size_of::<&T>() % mem::size_of::<usize>() == 0 );
		let words = mem::size_of::<&T>() / mem::size_of::<usize>();
		slice::from_raw_parts_mut(ptr as *mut &T as *mut usize, words)
	}
	unsafe fn as_ptr(&self) -> *mut T {
		let mut ret: &T = mem::zeroed();
		{
			let ret_as_slice = Self::ptr_as_slice(&mut ret);
			assert!(ret_as_slice.len() == 2);
			ret_as_slice[0] = &self.data as *const _ as usize;
			ret_as_slice[1] = self.vtable as usize;
		}
		ret as *const _ as *mut _
	}
	fn as_ref(&self) -> &T {
		unsafe {
			&*self.as_ptr()
		}
	}
	fn as_mut(&mut self) -> &mut T {
		unsafe {
			&mut *self.as_ptr()
		}
	}
}
impl<T: ?Sized> ops::Deref for StackDST<T> {
	type Target = T;
	fn deref(&self) -> &T {
		self.as_ref()
	}
}
impl<T: ?Sized> ops::DerefMut for StackDST<T> {
	fn deref_mut(&mut self) -> &mut T {
		self.as_mut()
	}
}
impl<T: ?Sized> ops::Drop for StackDST<T> {
	fn drop(&mut self) {
		unsafe {
			intrinsics::drop_in_place(self.as_mut())
		}
	}
}
