//! Single DST stored inline

use core::{marker, mem, ops, ptr};

/// Stack-allocated dynamically sized type
///
/// `T` is the unsized type contained.
/// `D` is the buffer used to hold the unsized type (both data and metadata).
pub struct ValueA<T: ?Sized, D: ::DataBuf> {
    _pd: marker::PhantomData<T>,
    // Data contains the object data first, then padding, then the pointer information
    data: D,
}

impl<T: ?Sized, D: ::DataBuf> ValueA<T, D> {
    /// Construct a stack-based DST
    ///
    /// Returns Ok(dst) if the allocation was successful, or Err(val) if it failed
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
    #[cfg(feature = "unsize")]
    pub fn in_buffer<U: marker::Unsize<T>>(buffer: D, val: U) -> Result<ValueA<T, D>, U>
    where
        (U, D::Inner): crate::AlignmentValid,
    {
        Self::in_buffer_stable(buffer, val, |p| p)
    }

    /// Construct a stack-based DST (without needing `Unsize`)
    ///
    /// Returns `Ok(dst)` if the allocation was successful, or `Err(val)` if it failed
    pub fn new_stable<U, F: FnOnce(&U) -> &T>(val: U, get_ref: F) -> Result<ValueA<T, D>, U>
    where
        (U, D::Inner): crate::AlignmentValid,
        D: Default,
    {
        Self::in_buffer_stable(D::default(), val, get_ref)
    }

    /// Construct a stack-based DST (without needing `Unsize`)
    ///
    /// Returns `Ok(dst)` if the allocation was successful, or `Err(val)` if it failed
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
            let mut ptr: *const _ = crate::check_fat_pointer(&val, get_ref);
            let words = super::ptr_as_slice(&mut ptr);

            ValueA::new_raw(
                &words[1..],
                words[0] as *mut (),
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
    /// use stack_dst::ValueA;
    /// use core::fmt::Debug;
    /// let val = [1usize, 2, 3, 4];
    /// assert!( ValueA::<dyn Debug, [usize; 2]>::new(val).is_err() );
    /// let v = ValueA::<dyn Debug, [usize; 2]>::new_or_boxed(val);
    /// println!("v = {:?}", v);
    /// ```
    pub fn new_or_boxed<U>(val: U) -> ValueA<T, D>
    where
        U: marker::Unsize<T>,
        ::alloc::boxed::Box<U>: marker::Unsize<T>,
        D: Default,
    {
        Self::new(val).unwrap_or_else(|val| {
            Self::new(Box::new(val))
                .ok()
                .expect("Insufficient space for Box<T>")
        })
    }

    /// UNSAFE: `data` must point to `size` bytes, which shouldn't be freed if `Some` is returned
    pub unsafe fn new_raw(
        info: &[usize],
        data: *mut (),
        size: usize,
        buffer: D,
    ) -> Option<ValueA<T, D>> {
        if info.len() * mem::size_of::<usize>() + size > mem::size_of::<D>() {
            None
        } else {
            let mut rv = ValueA {
                _pd: marker::PhantomData,
                data: buffer,
            };
            assert!(info.len() + D::round_to_words(size) <= rv.data.as_ref().len());

            // Place pointer information at the end of the region
            // - Allows the data to be at the start for alignment purposes
            {
                let info_ofs = rv.data.as_ref().len() - info.len();
                let info_dst = &mut rv.data.as_mut()[info_ofs..];
                crate::store_metadata(info_dst, info);
            }

            let src_ptr = data as *const u8;
            let dataptr = rv.data.as_mut()[..].as_mut_ptr() as *mut u8;
            for i in 0..size {
                *dataptr.add(i) = *src_ptr.add(i);
            }
            Some(rv)
        }
    }

    #[cfg(false_)]
    #[cfg(feature = "unsize")]
    // TODO: A function to replace the contents (reusing a non-trivial buffer)
    pub fn replace<U>(&mut self, val: U) -> Result<(), U> {
        // Check size requirements (allow resizing)
        // If met, drop the existing item and move in the new item
    }

    /// Obtain raw pointer to the contained data
    unsafe fn as_ptr(&self) -> *mut T {
        let data = self.data.as_ref();
        let info_size = mem::size_of::<*mut T>() / mem::size_of::<usize>() - 1;
        let info_ofs = data.len() - info_size;
        super::make_fat_ptr(data[..].as_ptr() as usize, &data[info_ofs..])
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
        unsafe { &mut *self.as_ptr() }
    }
}
impl<T: ?Sized, D: ::DataBuf> ops::Drop for ValueA<T, D> {
    fn drop(&mut self) {
        unsafe { ptr::drop_in_place(&mut **self) }
    }
}

mod trait_impls;
