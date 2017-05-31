//! 
//!
//!
use std::{mem,marker,ptr,ops};

/// A fixed-capacity stack that can contain dynamically-sized types
///
/// Uses an array of usize as a backing store for a First-In, Last-Out stack
/// of items that can unsize to `T`.
///
/// Note: Each item in the stack takes at least one `usize` (to store the metadata)
pub struct StackA<T: ?Sized, D: ::DataBuf>
{
	_pd: marker::PhantomData<*const T>,
	next_ofs: usize,
	data: D,	// 1KB/512B
}

impl<T: ?Sized, D: ::DataBuf> ops::Drop for StackA<T,D>
{
	fn drop(&mut self)
	{
		while ! self.empty()
		{
			self.pop();
		}
	}
}

impl<T: ?Sized, D: ::DataBuf> StackA<T,D>
{
	/// Construct a new (empty) stack
	pub fn new() -> StackA<T, D>
	{
		StackA {
			_pd: marker::PhantomData,
			next_ofs: 0,
			data: Default::default(),
			}
	}

	/// Tests if the stack is empty
	pub fn empty(&self) -> bool
	{
		self.next_ofs == 0
	}

	/// Push a value at the top of the stack
	pub fn push<U: marker::Unsize<T>>(&mut self, v: U) -> Result<(), U>
	{
		// - Ensure that Self is aligned same as data requires
		assert!(mem::align_of::<U>() <= mem::align_of::<Self>(), "TODO: Enforce alignment >{} (requires {})",
			mem::align_of::<Self>(), mem::align_of::<U>());

		let words = round_to_words( mem::size_of::<U>() );
		if self.next_ofs + words + 1 <= self.data.as_ref().len()
		{
			let meta = get_meta_usize(&v as &T);
			if words > 0
			{
				self.data.as_mut()[self.next_ofs + words-1] = 0;
				unsafe { 
					ptr::write( &mut self.data.as_mut()[self.next_ofs] as *mut _ as *mut U, v );
				}
			}
			self.data.as_mut()[self.next_ofs+words] = meta;
			self.next_ofs += words+1;
			Ok( () )
		}
		else
		{
			Err(v)
		}
	}

	fn top_item_size_words(&self) -> usize {
		assert!(self.next_ofs != 0);
		
		// TODO: Is this safe? It shouldn't access the data, right?
		let size_bytes = mem::size_of_val( unsafe { &*make_fat_ptr::<T>(1, self.data.as_ref()[self.next_ofs - 1]) } );
		round_to_words(size_bytes)
	}

	fn top_raw(&self) -> Option<*mut T>
	{
		if self.next_ofs == 0
		{
			None
		}
		else
		{
			let size = self.top_item_size_words();
			Some( make_fat_ptr( 
				&self.data.as_ref()[self.next_ofs - 1 - size] as *const _ as usize,
				self.data.as_ref()[self.next_ofs - 1]
				) )
		}
	}
	/// Returns a pointer to the top item on the stack
	pub fn top(&self) -> Option<&T>
	{
		self.top_raw().map(|x| unsafe { &*x })
	}
	/// Returns a pointer to the top item on the stack (unique/mutable)
	pub fn top_mut(&mut self) -> Option<&mut T>
	{
		self.top_raw().map(|x| unsafe { &mut *x })
	}
	/// Pop the top item off the stack
	pub fn pop(&mut self)
	{
		if let Some(ptr) = self.top_raw()
		{
			assert!(self.next_ofs > 0);
			// SAFE: Pointer is valid, and will never be accessed after this point
			unsafe { ptr::drop_in_place(ptr); }
			let words = self.top_item_size_words();
			self.next_ofs -= words+1;
		}
	}
}

impl<D: ::DataBuf> StackA<str,D>
{
	/// Push the contents of a string slice as an item onto the stack
	pub fn push_str(&mut self, v: &str) -> Result<(),()>
	{
		let words = round_to_words( v.len() );
		if self.next_ofs + words + 1 <= self.data.as_ref().len()
		{
			let meta = v.len();
			if words > 0
			{
				self.data.as_mut()[self.next_ofs + words-1] = 0;
				unsafe { 
					ptr::copy( v.as_bytes().as_ptr(), &mut self.data.as_mut()[self.next_ofs] as *mut _ as *mut u8, v.len() );
				}
			}
			self.data.as_mut()[self.next_ofs+words] = meta;
			self.next_ofs += words+1;
			Ok( () )
		}
		else
		{
			Err( () )
		}
	}
}
impl<D: ::DataBuf, T: Clone> StackA<[T],D>
{
	/// Pushes a set of items (cloning out of the input slice)
	pub fn push_cloned(&mut self, v: &[T]) -> Result<(),()>
	{
		let words = round_to_words( mem::size_of_val(v) );
		if self.next_ofs + words + 1 <= self.data.as_ref().len()
		{
			let meta = v.len();
			if words > 0
			{
				self.data.as_mut()[self.next_ofs + words-1] = 0;
				let mut ptr = &mut self.data.as_mut()[self.next_ofs] as *mut _ as *mut T;
				for val in v
				{
					unsafe {
						ptr::write(ptr, val.clone());
						ptr = ptr.offset(1);
					}
				}
			}
			self.data.as_mut()[self.next_ofs+words] = meta;
			self.next_ofs += words+1;
			Ok( () )
		}
		else
		{
			Err( () )
		}
	}
}

/// Obtains the metadata for the given fat pointer as a usize
///
/// Asserts if the pointer isn't a two-word pointer, or if the layout isn't (data, meta)
fn get_meta_usize<T: ?Sized>(mut p: *const T) -> usize
{
	let p_data = p as *const ();
	let s = unsafe { super::ptr_as_slice(&mut p) };
	assert_eq!( s.len(), 2 );
	assert_eq!( p_data as usize, s[0], "Fat pointer layout not as expected, first word isn't the data pointer" );
	s[1]
}

fn make_fat_ptr<T: ?Sized>(data_ptr: usize, meta_val: usize) -> *mut T {
	// SAFE: Nothing glaring
	unsafe
	{
		let mut rv: *const T = mem::zeroed();
		{
			let s = super::ptr_as_slice(&mut rv);
			s[0] = data_ptr;
			s[1] = meta_val;
		}
		assert_eq!(rv as *const (), data_ptr as *const ());
		rv as *mut T
	}
}

fn round_to_words(len: usize) -> usize {
	(len + mem::size_of::<usize>()-1) / mem::size_of::<usize>()
}
