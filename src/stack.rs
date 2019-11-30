//! 
//!
//!
use std::{mem,marker,ptr,ops};

// Implementation Notes
// -----
//
// The data array is filled from the back, with the metadata stored before (at a lower memory address)
// the actual data. This so the code can use a single integer to track the position (using size_of_val
// when popping items, and the known size when pushing).

/// A fixed-capacity stack that can contain dynamically-sized types
///
/// Uses an array of usize as a backing store for a First-In, Last-Out stack
/// of items that can unsize to `T`.
///
/// Note: Each item in the stack takes at least one `usize` (to store the metadata)
pub struct StackA<T: ?Sized, D: ::DataBuf>
{
	_pd: marker::PhantomData<*const T>,
	// Offset from the _back_ of `data` to the next free position.
	// I.e. data[data.len() - cur_ofs] is the first metadata word
	next_ofs: usize,
	data: D,
}

impl<T: ?Sized, D: ::DataBuf> ops::Drop for StackA<T,D>
{
	fn drop(&mut self)
	{
		while ! self.is_empty()
		{
			self.pop();
		}
	}
}
impl<T: ?Sized, D: ::DataBuf> Default for StackA<T,D> {
	fn default() -> Self {
		StackA::new()
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
	pub fn is_empty(&self) -> bool
	{
		self.next_ofs == 0
	}

	fn meta_words() -> usize
	{
		mem::size_of::<&T>() / mem::size_of::<usize>() - 1
	}
	fn push_inner(&mut self, fat_ptr: &T) -> Result<&mut [usize], ()>
	{
		let bytes = mem::size_of_val(fat_ptr);
		let words = super::round_to_words(bytes) + Self::meta_words();
		// Check if there is sufficient space for the new item
		if self.next_ofs + words <= self.data.as_ref().len()
		{
			// Get the base pointer for the new item
			self.next_ofs += words;
			let len = self.data.as_ref().len();
			let slot = &mut self.data.as_mut()[len - self.next_ofs..][..words];
			let (meta, rv) = slot.split_at_mut(Self::meta_words());

			// Populate the metadata
			let mut ptr_raw: *const T = fat_ptr;
			let ptr_words = ::ptr_as_slice(&mut ptr_raw);
			assert_eq!(ptr_words.len(), 1 + Self::meta_words());
			meta.clone_from_slice( &ptr_words[1..] );

			// Increment offset and return
			Ok( rv )
		}
		else
		{
			Err( () )
		}
	}

	/// Push a value at the top of the stack
	pub fn push<U: marker::Unsize<T>>(&mut self, v: U) -> Result<(), U>
	{
		// - Ensure that Self is aligned same as data requires
		assert!(mem::align_of::<U>() <= mem::align_of::<Self>(), "TODO: Enforce alignment >{} (requires {})",
			mem::align_of::<Self>(), mem::align_of::<U>());

		match self.push_inner(&v)
		{
		Ok(d) => {
			// SAFE: Destination address is valid
			unsafe { ptr::write( d.as_mut_ptr() as *mut U, v ); }
			Ok( () )
			},
		Err(_) => Err(v),
		}
	}

	// Get a raw pointer to the top of the stack
	fn top_raw(&self) -> Option<*mut T>
	{
		if self.next_ofs == 0
		{
			None
		}
		else
		{
			let len = self.data.as_ref().len();
			let meta = &self.data.as_ref()[len - self.next_ofs..];
			// SAFE: Internal consistency maintains the metadata validity
			Some( unsafe { super::make_fat_ptr( 
				meta[Self::meta_words()..].as_ptr() as usize,
				&meta[..Self::meta_words()]
				) } )
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
			let words = unsafe {
				let size = mem::size_of_val(&*ptr);
				ptr::drop_in_place(ptr);
				super::round_to_words(size)
				};
			self.next_ofs -= words+1;
		}
	}
}

impl<D: ::DataBuf> StackA<str,D>
{
	/// Push the contents of a string slice as an item onto the stack
	pub fn push_str(&mut self, v: &str) -> Result<(),()>
	{
		match self.push_inner(v)
		{
		Ok(d) => {
			unsafe { 
				ptr::copy( v.as_bytes().as_ptr(), d.as_mut_ptr() as *mut u8, v.len() );
			}
			Ok( () )
			},
		Err(_) => Err( () ),
		}
	}
}
impl<D: ::DataBuf, T: Clone> StackA<[T],D>
{
	/// Pushes a set of items (cloning out of the input slice)
	pub fn push_cloned(&mut self, v: &[T]) -> Result<(),()>
	{
		match self.push_inner(&v)
		{
		Ok(d) => {
			unsafe
			{
				let mut ptr = d.as_mut_ptr() as *mut T;
				for val in v
				{
					ptr::write(ptr, val.clone());
					ptr = ptr.offset(1);
				}
			}
			Ok( () )
			},
		Err(_) => Err( () ),
		}
	}
}

