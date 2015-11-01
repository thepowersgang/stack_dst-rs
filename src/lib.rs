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
//! let dst = StackDST::<Any>::new(1234u64).ok().expect("Integer did not fit in allocation");
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
//!     StackDST::new(move || format!("Hello there! value={}", value)).ok().expect("Closure doesn't fit")
//! }
//! let mut closure = make_closure(666);
//! assert_eq!( (&mut *closure)(), "Hello there! value=666" );
//! ```
#![feature(unsize,drop_in_place)]	// needed for intrinsics, raw, and Unsize
#![cfg_attr(no_std,feature(no_std,core_slice_ext))]
#![cfg_attr(no_std,no_std)]
#![crate_type="lib"]
#![crate_name="stack_dst"]
#![deny(missing_docs)]

#[cfg(not(no_std))]
use std::{ops,mem,slice,marker,ptr};

#[cfg(no_std)]
use core::{ops,mem,slice,marker,ptr};

/// Trait used to represent the data buffer for StackDSTA.
/// 
/// Typically you'll passs a [usize; N] array
pub trait DataBuf: Default+AsMut<[usize]>+AsRef<[usize]> {
}
impl<T: Default+AsMut<[usize]>+AsRef<[usize]>> DataBuf for T {
}

/// 8 data words, plus one metadata
pub const DEFAULT_SIZE: usize = 8+1;

/// Stack-allocated DST (using a default size)
pub type StackDST<T/*: ?Sized*/> = StackDSTA<T, [usize; DEFAULT_SIZE]>;

/// Stack-allocated dynamically sized type
///
/// `T` is the unsized type contaned.
/// `D` is the buffer used to hold the unsized type (both data and metadata).
pub struct StackDSTA<T: ?Sized, D: DataBuf> {
	// Force alignment to be 8 bytes (for types that contain u64s)
	_align: [u64; 0],
	_pd: marker::PhantomData<T>,
	// Data contains the object data first, then padding, then the pointer information
	data: D,
}

unsafe fn ptr_as_slice<'p, T: ?Sized>(ptr: &'p mut &T) -> &'p mut [usize] {
	assert!( mem::size_of::<&T>() % mem::size_of::<usize>() == 0 );
	let words = mem::size_of::<&T>() / mem::size_of::<usize>();
	slice::from_raw_parts_mut(ptr as *mut &T as *mut usize, words)
}

/// Obtain raw pointer given a StackDST reference
unsafe fn as_ptr<T: ?Sized, D: DataBuf>(s: &StackDSTA<T, D>) -> *mut T {
	let mut ret: &T = mem::zeroed();
	{
		let ret_as_slice = ptr_as_slice(&mut ret);
		let data = s.data.as_ref();
		// 1. Data pointer
		ret_as_slice[0] = data[..].as_ptr() as usize;
		// 2. Pointer info
		let info_size = ret_as_slice.len() - 1;
		let info_ofs = data.len() - info_size;
		for i in 0 .. info_size {
			ret_as_slice[1+i] = data[info_ofs + i];
		}
	}
	ret as *const _ as *mut _
}

impl<T: ?Sized, D: DataBuf> StackDSTA<T, D>
{
	/// Construct a stack-based DST
	/// 
	/// Returns Ok(dst) if the allocation was successful, or Err(val) if it failed
	pub fn new<U: marker::Unsize<T>>(val: U) -> Result<StackDSTA<T,D>,U> {
		let rv = unsafe {
			let mut ptr: &T = &val;
			let words = ptr_as_slice(&mut ptr);
			assert!(words[0] == &val as *const _ as usize, "BUG: Pointer layout is not (data_ptr, info...)");
			// - Ensure that Self is aligned same as data requires
			assert!(mem::align_of::<U>() <= mem::align_of::<Self>(), "TODO: Enforce alignment >{} (requires {})",
				mem::align_of::<Self>(), mem::align_of::<U>());
			
			StackDSTA::new_raw(&words[1..], words[0] as *mut (), mem::size_of::<U>())
			};
		match rv
		{
		Some(r) => {
			// Prevent the destructor from running, now that we've copied it away
			mem::forget(val);
			Ok(r)
			},
		None => {
			Err(val)
			},
		}
	}
	
	unsafe fn new_raw(info: &[usize], data: *mut (), size: usize) -> Option<StackDSTA<T,D>>
	{
		if info.len()*mem::size_of::<usize>() + size > mem::size_of::<D>() {
			None
		}
		else {
			let mut rv = StackDSTA {
					_align: [],
					_pd: marker::PhantomData,
					data: D::default(),
				};
			assert!(info.len() + (size + mem::size_of::<usize>() - 1) / mem::size_of::<usize>() <= rv.data.as_ref().len());

			// Place pointer information at the end of the region
			// - Allows the data to be at the start for alignment purposes
			{
				let info_ofs = rv.data.as_ref().len() - info.len();
				let info_dst = &mut rv.data.as_mut()[info_ofs..];
				for (d,v) in Iterator::zip( info_dst.iter_mut(), info.iter() ) {
					*d = *v;
				}
			}
			
			let src_ptr = data as *const u8;
			let dataptr = rv.data.as_mut()[..].as_mut_ptr() as *mut u8;
			for i in 0 .. size {
				*dataptr.offset(i as isize) = *src_ptr.offset(i as isize);
			}
			Some(rv)
		}
	}
}
impl<T: ?Sized, D: DataBuf> ops::Deref for StackDSTA<T, D> {
	type Target = T;
	fn deref(&self) -> &T {
		unsafe {
			&*as_ptr(self)
		}
	}
}
impl<T: ?Sized, D: DataBuf> ops::DerefMut for StackDSTA<T, D> {
	fn deref_mut(&mut self) -> &mut T {
		unsafe {
			&mut *as_ptr(self)
		}
	}
}
impl<T: ?Sized, D: DataBuf> ops::Drop for StackDSTA<T, D> {
	fn drop(&mut self) {
		unsafe {
			ptr::drop_in_place(&mut **self)
		}
	}
}
