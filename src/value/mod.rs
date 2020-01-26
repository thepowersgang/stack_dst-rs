//! Single DST stored inline

use std::{marker, mem, ops, ptr};

/// 8 data words, plus one metadata
pub const DEFAULT_SIZE: usize = 8 + 1;

/// Stack-allocated DST (using a default size)
pub type Value<T /*: ?Sized*/> = ValueA<T, [usize; DEFAULT_SIZE]>;

/// Stack-allocated dynamically sized type
///
/// `T` is the unsized type contained.
/// `D` is the buffer used to hold the unsized type (both data and metadata).
pub struct ValueA<T: ?Sized, D: ::DataBuf> {
    // Force alignment to be 8 bytes (for types that contain u64s)
    _align: [u64; 0],
    _pd: marker::PhantomData<T>,
    // Data contains the object data first, then padding, then the pointer information
    data: D,
}

impl<T: ?Sized, D: ::DataBuf> ValueA<T, D> {
    /// Construct a stack-based DST
    ///
    /// Returns Ok(dst) if the allocation was successful, or Err(val) if it failed
    #[cfg(feature = "unsize")]
    pub fn new<U: marker::Unsize<T>>(val: U) -> Result<ValueA<T, D>, U> {
        Self::new_stable(val, |p| p)
    }

    /// Construct a stack-based DST (without needing `Unsize`)
    ///
    /// Returns Ok(dst) if the allocation was successful, or Err(val) if it failed
    pub fn new_stable<U, F: FnOnce(&U) -> &T>(val: U, get_ref: F) -> Result<ValueA<T, D>, U> {
        let rv = unsafe {
            let mut ptr: *const T = get_ref(&val);
            let words = super::ptr_as_slice(&mut ptr);
            assert!(
                words[0] == &val as *const _ as usize,
                "BUG: Pointer layout is not (data_ptr, info...)"
            );
            // - Ensure that Self is aligned same as data requires
            assert!(
                mem::align_of::<U>() <= mem::align_of::<Self>(),
                "TODO: Enforce alignment >{} (requires {})",
                mem::align_of::<Self>(),
                mem::align_of::<U>()
            );

            ValueA::new_raw(&words[1..], words[0] as *mut (), mem::size_of::<U>())
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
    {
        Self::new(val).unwrap_or_else(|val| {
            Self::new(Box::new(val))
                .ok()
                .expect("Insufficient space for Box<T>")
        })
    }

    /// UNSAFE: `data` must point to `size` bytes, which shouldn't be freed if `Some` is returned
    pub unsafe fn new_raw(info: &[usize], data: *mut (), size: usize) -> Option<ValueA<T, D>> {
        if info.len() * mem::size_of::<usize>() + size > mem::size_of::<D>() {
            None
        } else {
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
                for (d, v) in Iterator::zip(info_dst.iter_mut(), info.iter()) {
                    *d = *v;
                }
            }

            let src_ptr = data as *const u8;
            let dataptr = rv.data.as_mut()[..].as_mut_ptr() as *mut u8;
            for i in 0..size {
                *dataptr.offset(i as isize) = *src_ptr.offset(i as isize);
            }
            Some(rv)
        }
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
