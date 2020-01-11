//! Single DST stored inline
//!
//!


use std::{ops,mem,marker,ptr};

/// 8 data words, plus one metadata
pub const DEFAULT_SIZE: usize = 8+1;

/// Stack-allocated DST (using a default size)
pub type Value<T/*: ?Sized*/> = ValueA<T, [usize; DEFAULT_SIZE]>;

/// Stack-allocated dynamically sized type
///
/// `T` is the unsized type contaned.
/// `D` is the buffer used to hold the unsized type (both data and metadata).
pub struct ValueA<T: ?Sized, D: ::DataBuf> {
	// Force alignment to be 8 bytes (for types that contain u64s)
	_align: [u64; 0],
	_pd: marker::PhantomData<T>,
	// Data contains the object data first, then padding, then the pointer information
	data: D,
}

impl<T: ?Sized, D: ::DataBuf> ValueA<T, D>
{
	/// Construct a stack-based DST
	/// 
	/// Returns Ok(dst) if the allocation was successful, or Err(val) if it failed
	pub fn new<U: marker::Unsize<T>>(val: U) -> Result<ValueA<T,D>,U> {
		let rv = unsafe {
			let mut ptr: *const T = &val as &T;
			let words = super::ptr_as_slice(&mut ptr);
			assert!(words[0] == &val as *const _ as usize, "BUG: Pointer layout is not (data_ptr, info...)");
			// - Ensure that Self is aligned same as data requires
			assert!(mem::align_of::<U>() <= mem::align_of::<Self>(), "TODO: Enforce alignment >{} (requires {})",
				mem::align_of::<Self>(), mem::align_of::<U>());
			
			ValueA::new_raw(&words[1..], words[0] as *mut (), mem::size_of::<U>())
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
	
	unsafe fn new_raw(info: &[usize], data: *mut (), size: usize) -> Option<ValueA<T,D>>
	{
		if info.len()*mem::size_of::<usize>() + size > mem::size_of::<D>() {
			None
		}
		else {
			let mut rv = ValueA {
					_align: [],
					_pd: marker::PhantomData,
					data: D::default(),
				};
			assert!(info.len() + super::round_to_words(size) <= rv.data.as_ref().len());

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

	/// Obtain raw pointer to the contained data
	unsafe fn as_ptr(&self) -> *mut T
	{
		let data = self.data.as_ref();
		let info_size = mem::size_of::<*mut T>() / mem::size_of::<usize>() - 1;
		let info_ofs = data.len() - info_size;
		super::make_fat_ptr( data[..].as_ptr() as usize, &data[info_ofs..] )
	}
}
impl<T: ?Sized, D: ::DataBuf> ops::Deref for ValueA<T, D> {
	type Target = T;
	fn deref(&self) -> &T {
		unsafe {
			&*self.as_ptr()
		}
	}
}
impl<T: ?Sized, D: ::DataBuf> ops::DerefMut for ValueA<T, D> {
	fn deref_mut(&mut self) -> &mut T {
		unsafe {
			&mut *self.as_ptr()
		}
	}
}
impl<T: ?Sized, D: ::DataBuf> ops::Drop for ValueA<T, D> {
	fn drop(&mut self) {
		unsafe {
			ptr::drop_in_place(&mut **self)
		}
	}
}

mod trait_impls;

