//! Single DST stored inline

use core::{marker, mem, ops, ptr};

/// Stack-allocated dynamically sized type
///
/// `T` is the unsized type contained.
/// `D` is the buffer used to hold the unsized type (both data and metadata).
///
/// ```
/// # extern crate core;
/// # use stack_dst::ValueA;
/// # use core::fmt::Display;
/// let val = ValueA::<dyn Display, ::stack_dst::buffers::Ptr2>::new_stable(123456, |v| v as _).expect("Insufficient size");
/// assert_eq!( format!("{}", val), "123456" );
/// ```
pub struct ValueA<T: ?Sized, D: ::DataBuf> {
    _pd: marker::PhantomData<T>,
    // Data contains the object data first, then padding, then the pointer information
    data: D,
}

impl<T: ?Sized, D: ::DataBuf> ValueA<T, D> {
    /// Construct a stack-based DST
    ///
    /// Returns Ok(dst) if the allocation was successful, or Err(val) if it failed
    ///
    /// ```
    /// # extern crate core;
    /// # use stack_dst::ValueA;
    /// # use core::fmt::Display;
    /// let val = ValueA::<dyn Display, ::stack_dst::buffers::Ptr2>::new(1234).expect("Insufficient size");
    /// assert_eq!( format!("{}", val), "1234" );
    /// ```
    #[cfg(feature = "unsize")]
    pub fn new<U: marker::Unsize<T>>(val: U) -> Result<ValueA<T, D>, U>
    where
        (U, D::Inner): crate::AlignmentValid,
        D: Default,
    {
        Self::new_stable(val, |p| p)
    }

    /// Construct a stack-based DST using a pre-constructed buffer
    ///
    /// Returns `Ok(dst)` if the allocation was successful, or `Err(val)` if it failed
    ///
    /// ```
    /// # extern crate core;
    /// # use stack_dst::ValueA;
    /// # use core::fmt::Display;
    /// # use core::mem::MaybeUninit;
    /// let val = ValueA::<dyn Display, _>::in_buffer([MaybeUninit::new(0u64); 2], 1234).expect("Insufficient size");
    /// assert_eq!( format!("{}", val), "1234" );
    /// ```
    #[cfg(feature = "unsize")]
    pub fn in_buffer<U: marker::Unsize<T>>(buffer: D, val: U) -> Result<ValueA<T, D>, U>
    where
        (U, D::Inner): crate::AlignmentValid,
    {
        Self::in_buffer_stable(buffer, val, |p| p)
    }

    /// Construct a stack-based DST (without needing `Unsize`). The closure `get_ref`
    /// must just convert `&U` to `&U` (if the pointers don't match, an assertion triggers)
    ///
    /// Returns `Ok(dst)` if the allocation was successful, or `Err(val)` if it failed
    ///
    /// ```
    /// # extern crate core;
    /// # use stack_dst::ValueA;
    /// # use core::fmt::Display;
    /// let val = ValueA::<dyn Display, ::stack_dst::buffers::Ptr2>::new_stable(1234, |v| v as _).expect("Insufficient size");
    /// assert_eq!( format!("{}", val), "1234" );
    /// ```
    pub fn new_stable<U, F: FnOnce(&U) -> &T>(val: U, get_ref: F) -> Result<ValueA<T, D>, U>
    where
        (U, D::Inner): crate::AlignmentValid,
        D: Default,
    {
        Self::in_buffer_stable(D::default(), val, get_ref)
    }

    /// Construct a stack-based DST (without needing `Unsize`) using a provided buffer.
    /// See `new_stable` for requirements on the `get_ref` closure.
    ///
    /// Returns `Ok(dst)` if the allocation was successful, or `Err(val)` if it failed
    ///
    /// ```
    /// # extern crate core;
    /// # use stack_dst::ValueA;
    /// # use core::fmt::Display;
    /// # use core::mem::MaybeUninit;
    /// let val = ValueA::<dyn Display, _>::in_buffer_stable([MaybeUninit::new(0u64); 2], 1234, |v| v as _).expect("Insufficient size");
    /// assert_eq!( format!("{}", val), "1234" );
    /// ```
    pub fn in_buffer_stable<U, F: FnOnce(&U) -> &T>(
        buffer: D,
        val: U,
        get_ref: F,
    ) -> Result<ValueA<T, D>, U>
    where
        (U, D::Inner): crate::AlignmentValid,
    {
        <(U, D::Inner) as crate::AlignmentValid>::check();

        let rv = unsafe {
            let ptr: *const _ = crate::check_fat_pointer(&val, get_ref);
            let (raw_ptr, meta_len, meta) = super::decompose_pointer(ptr);

            ValueA::new_raw(
                &meta[..meta_len],
                raw_ptr as *mut _,
                mem::size_of::<U>(),
                buffer,
            )
        };
        match rv {
            Some(r) => {
                // Prevent the destructor from running, now that we've copied it away
                mem::forget(val);
                Ok(r)
            }
            None => Err(val),
        }
    }

    #[cfg(all(feature = "alloc", feature = "unsize"))]
    /// Construct a stack-based DST, falling back on boxing if the value doesn't fit
    ///
    /// ```
    /// # extern crate core;
    /// # use stack_dst::ValueA;
    /// # use core::fmt::Debug;
    /// let val = [1usize, 2, 3, 4];
    /// assert!( ValueA::<dyn Debug, ::stack_dst::buffers::Ptr2>::new(val).is_err() );
    /// let v = ValueA::<dyn Debug, ::stack_dst::buffers::Ptr2>::new_or_boxed(val);
    /// println!("v = {:?}", v);
    /// ```
    pub fn new_or_boxed<U>(val: U) -> ValueA<T, D>
    where
        (U, D::Inner): crate::AlignmentValid,
        U: marker::Unsize<T>,
        ::alloc::boxed::Box<U>: marker::Unsize<T>,
        D: Default,
    {
        Self::new(val).unwrap_or_else(|val| {
            Self::new::<::alloc::boxed::Box<_>>(::alloc::boxed::Box::new(val))
                .ok()
                .expect("Insufficient space for Box<T>")
        })
    }

    /// UNSAFE: `data` must point to `size` bytes, which shouldn't be freed if `Some` is returned
    pub unsafe fn new_raw(
        info: &[usize],
        data: *mut (),
        size: usize,
        mut buffer: D,
        ) -> Option<ValueA<T, D>>
    {
        let req_words = D::round_to_words(mem::size_of_val(info)) + D::round_to_words(size);
        if let Err(_) = buffer.extend( req_words ) {
            return None;
        }

        let mut rv = mem::ManuallyDrop::new(ValueA::<T,D> {
            _pd: marker::PhantomData,
            data: buffer,
        });
        rv.write_value(data, size, info);
        Some(mem::ManuallyDrop::into_inner(rv))
    }

    unsafe fn write_value(&mut self, data: *const (), size: usize, info: &[usize])
    {
        let info_words = D::round_to_words(mem::size_of_val(info));
        let req_words = info_words + D::round_to_words(size);
        let buf = self.data.as_mut();
        assert!(req_words <= buf.len());

        // Place pointer information at the end of the region
        // - Allows the data to be at the start for alignment purposes
        {
            let info_ofs = buf.len() - info_words;
            let info_dst = &mut buf[info_ofs..];
            crate::store_metadata(info_dst, info);
        }

        ptr::copy_nonoverlapping(data as *const u8, buf.as_mut_ptr() as *mut u8, size);
    }

    /// Replace the contents without dropping the backing allocation
    /// 
    /// ```
    /// # extern crate core;
    /// # use stack_dst::ValueA;
    /// # use core::fmt::Display;
    /// let mut value = ValueA::<dyn Display, ::stack_dst::buffers::Ptr2>::new_stable(1234, |v| v).unwrap();
    /// assert_eq!(format!("{}", value), "1234");
    /// value.replace_stable(1.234, |v| v).unwrap();
    /// assert_eq!(format!("{}", value), "1.234");
    /// ```
    pub fn replace_stable<U>(&mut self, val: U, get_ref: impl Fn(&U) -> &T) -> Result<(), U>
    where
        (U, D::Inner): crate::AlignmentValid,
    {
        <(U, D::Inner) as crate::AlignmentValid>::check();

        let size = mem::size_of::<U>();
        let (raw_ptr, meta_len, meta) = super::decompose_pointer( crate::check_fat_pointer(&val, get_ref) );
        let info = &meta[..meta_len];

        // Check size requirements (allow resizing)
        let req_words = D::round_to_words(mem::size_of_val(info)) + D::round_to_words(size);
        if let Err(_) = self.data.extend(req_words) {
            return Err(val);
        }
        // If met, drop the existing item and move in the new item
        unsafe {
            ptr::drop_in_place::<T>(&mut **self);
            self.write_value(raw_ptr, mem::size_of::<U>(), info);
        }
        Ok( () )
    }
    #[cfg(feature = "unsize")]
    /// Replace the contents without dropping the backing allocation
    /// 
    /// ```
    /// # extern crate core;
    /// # use stack_dst::ValueA;
    /// # use core::fmt::Display;
    /// let mut value = ValueA::<dyn Display, ::stack_dst::buffers::Ptr2>::new(1234).unwrap();
    /// assert_eq!(format!("{}", value), "1234");
    /// value.replace(1.234).unwrap();
    /// assert_eq!(format!("{}", value), "1.234");
    /// ```
    pub fn replace<U>(&mut self, val: U) -> Result<(), U>
    where
        (U, D::Inner): crate::AlignmentValid,
        U: marker::Unsize<T>,
    {
        self.replace_stable(val, |v| v)
    }

    /// Obtain raw pointer to the contained data
    unsafe fn as_ptr(&self) -> *mut T {
        let data = self.data.as_ref();
        let info_size = mem::size_of::<*mut T>() / mem::size_of::<usize>() - 1;
        let info_ofs = data.len() - D::round_to_words(info_size * mem::size_of::<usize>());
        let (data, meta) = data.split_at(info_ofs);
        super::make_fat_ptr(data.as_ptr() as *mut (), meta)
    }

    /// Obtain raw pointer to the contained data
    unsafe fn as_ptr_mut(&mut self) -> *mut T {
        let data = self.data.as_mut();
        let info_size = mem::size_of::<*mut T>() / mem::size_of::<usize>() - 1;
        let info_ofs = data.len() - D::round_to_words(info_size * mem::size_of::<usize>());
        let (data, meta) = data.split_at_mut(info_ofs);
        super::make_fat_ptr(data.as_mut_ptr() as *mut (), meta)
    }
}
/// Specialisations for `str` (allowing storage of strings with single-byte alignment)
impl<D: ::DataBuf> ValueA<str, D> {
    /// Create a new empty string with a default buffer
    pub fn empty_str() -> Result<Self,()>
    where
        D: Default,
    {
        Self::empty_str_in_buffer(Default::default())
    }
    /// Create a new empty string with a provided buffer
    pub fn empty_str_in_buffer(buffer: D) -> Result<Self,()>
    {
        let rv = unsafe {
            let (raw_ptr, meta_len, meta) = super::decompose_pointer("");

            ValueA::new_raw(
                &meta[..meta_len],
                raw_ptr as *mut (),
                0,
                buffer,
            )
        };
        match rv {
            Some(r) => Ok(r),
            None => Err(()),
        }
    }
    /// Construct from a `str` using a default-constructed buffer
    /// ```
    /// # extern crate core;
    /// # use stack_dst::ValueA;
    /// # use core::fmt::Display;
    /// let val = ValueA::<str, stack_dst::buffers::U8_32>::new_str("Hello, World").expect("Insufficient size");
    /// assert_eq!( &val[..], "Hello, World" );
    /// ```
    pub fn new_str(v: &str) -> Result<Self,&str>
    where
        D: Default
    {
        Self::new_str_in_buffer(Default::default(), v)
    }
    /// Construct from a `str` using a provided buffer
    ///
    /// ```
    /// # extern crate core;
    /// # use stack_dst::ValueA;
    /// # use core::fmt::Display;
    /// # use core::mem::MaybeUninit;
    /// let val = ValueA::new_str_in_buffer([MaybeUninit::new(0u8); 32], "Hello, World").expect("Insufficient size");
    /// assert_eq!( &val[..], "Hello, World" );
    /// ```
    pub fn new_str_in_buffer(buffer: D, val: &str) -> Result<Self,&str>
    {
        let rv = unsafe {
            let (raw_ptr, meta_len, meta) = super::decompose_pointer(val);

            ValueA::new_raw(
                &meta[..meta_len],
                raw_ptr as *mut (),
                mem::size_of_val(val),
                buffer,
            )
        };
        match rv {
            Some(r) => Ok(r),
            None => Err(val),
        }
    }

    /// Add a string to the end of a string
    /// 
    /// ```
    /// # use stack_dst::ValueA;
    /// let mut s = ValueA::<str, stack_dst::buffers::Ptr8>::new_str("Foo").unwrap();
    /// s.append_str("Bar").unwrap();
    /// assert_eq!(&s[..], "FooBar");
    /// ```
    pub fn append_str(&mut self, val: &str) -> Result<(),()> {
        let info_words = D::round_to_words(mem::size_of::<usize>());
        
        let ofs = self.len();

        // Check/expand sufficient space
        let req_words = D::round_to_words( ofs + val.len() ) + info_words;
        if let Err(_) = self.data.extend(req_words) {
            return Err(());
        }

        // Get the metadata slot
        let data = self.data.as_mut();
        let info_ofs = data.len() - info_words;

        unsafe {
            ptr::copy_nonoverlapping(val.as_ptr(), (data.as_mut_ptr() as *mut u8).offset(ofs as isize), val.len());
            crate::store_metadata(&mut data[info_ofs..], &[ofs + val.len()]);
        }

        Ok(())
    }

    /// Resize the string (discarding trailing data)
    /// 
    /// ```
    /// # use stack_dst::ValueA;
    /// let mut s = ValueA::<str, stack_dst::buffers::Ptr8>::new_str("FooBar").unwrap();
    /// s.truncate(3);
    /// assert_eq!(&s[..], "Foo");
    /// ```
    pub fn truncate(&mut self, len: usize) {
        if len < self.len() {
            let _ = &self[..][len..];    // Index to force a panic if the index isn't char-aligned

            let info_words = D::round_to_words(mem::size_of::<usize>());
            let data = self.data.as_mut();
            let info_ofs = data.len() - info_words;
            crate::store_metadata(&mut data[info_ofs..], &[len]);
        }
    }
}
/// Specialisation for slices (acting like an `ArrayVec`)
impl<I, D: ::DataBuf> ValueA<[I], D>
where
    (I, D::Inner): crate::AlignmentValid,
{
    /// Create a new zero-sized slice (will error only if the metadata doesn't fit)
    pub fn empty_slice() -> Result<Self,()>
    where
        D: Default
    {
        Self::empty_slice_with_buffer(Default::default())
    }
    /// Create a new zero-sized slice in the provided buffer (will error only if the metadata doesn't fit)
    pub fn empty_slice_with_buffer(mut buffer: D) -> Result<Self,()>
    {
        <(I, D::Inner) as crate::AlignmentValid>::check();

        let info_words = D::round_to_words(mem::size_of::<usize>());
        let req_words = info_words + 0;
        if let Err(_) = buffer.extend( req_words ) {
            return Err( () );
        }
        assert!(req_words <= buffer.as_ref().len());

        let mut rv = ValueA {
            _pd: marker::PhantomData,
            data: buffer,
        };

        let data = rv.data.as_mut();
        let info_ofs = data.len() - info_words;
        let (_data_dst, info_dst) = data.split_at_mut(info_ofs);

        crate::store_metadata(info_dst, &[0]);
        Ok( rv )
    }

    /// Append an item to the end of the slice (similar to `Vec::push`)
    pub fn append(&mut self, v: I) -> Result<(), I> {
        let info_words = D::round_to_words(mem::size_of::<usize>());

        let ofs = self.len();

        // Check/expand sufficient space
        let req_words = D::round_to_words( (ofs + 1) * mem::size_of::<I>() ) + info_words;
        if let Err(_) = self.data.extend(req_words) {
            return Err(v);
        }
        let data = self.data.as_mut();
        assert!(req_words <= data.len());
        // Write the new value
        // SAFE: Alignment is checked, pointer is in-bounds
        unsafe {
            let data_ptr = (data.as_ptr() as *mut I).offset( ofs as isize );
            ptr::write(data_ptr, v);
        }
        // Only update item count after the write
        let info_ofs = data.len() - info_words;
        crate::store_metadata(&mut data[info_ofs..], &[ofs + 1]);

        Ok( () )
    }
    /// Inline append an item (See Self::append)
    pub fn appended(mut self, v: I) -> Result<Self, (Self, I)> {
        match self.append(v)
        {
        Ok(_) => Ok(self),
        Err(v) => Err( (self, v) ),
        }
    }

    /// Extend a slice with an iterator
    pub fn extend<It: Iterator<Item=I>>(&mut self, mut iter: It) -> Result<(), (I, It)>
    {
        while let Some(v) = iter.next()
        {
            match self.append(v)
            {
            Ok(_) => {},
            Err(v) => return Err( (v, iter) ),
            }
        }
        Ok( () )
    }
    /// Helper to extend during construction (see Self::extend)
    pub fn extended<It: Iterator<Item=I>>(mut self, iter: It) -> Result<Self, (Self, I, It)> {
        match self.extend(iter)
        {
        Ok(_) => Ok(self),
        Err( (v, iter) ) => Err( (self, v, iter) ),
        }
    }

    /// Remove the last item from the slice
    pub fn pop(&mut self) -> Option<I> {
        if self.len() > 0 {
            let ofs = self.len() - 1;
            let data = self.data.as_mut();
            let info_words = D::round_to_words(mem::size_of::<usize>());
            let info_ofs = data.len() - info_words;
            unsafe {
                crate::store_metadata(&mut data[info_ofs..], &[ofs]);
                Some( ptr::read((data.as_ptr() as *const I).offset(ofs as isize) ) )
            }
        }
        else {
            None
        }
    }
}
impl<T: ?Sized, D: ::DataBuf> ops::Deref for ValueA<T, D> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.as_ptr() }
    }
}
impl<T: ?Sized, D: ::DataBuf> ops::DerefMut for ValueA<T, D> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.as_ptr_mut() }
    }
}
impl<T: ?Sized, D: ::DataBuf> ops::Drop for ValueA<T, D> {
    fn drop(&mut self) {
        unsafe { ptr::drop_in_place(&mut **self) }
    }
}

mod trait_impls;
