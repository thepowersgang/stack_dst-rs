//
// Implementation of the `DataBuf` trait
//
use core::mem::MaybeUninit;

/// Trait used to represent a data buffer, typically you'll passs a `[usize; N]` array.
///
/// Can also provide a `Vec<T>` (if the `alloc` feature is enabled) which will grow as-needed
///
/// UNSAFE: Used by the internal unsafe code, must confor to the following rules
/// - The `as_ref`/`as_mut` methods must return pointers to the same data
/// - The pointer returned by `as_mut` must be stable until either a call to `extend` or the
///   value is moved (i.e. `let a = foo.as_mut().as_ptr(); let b = foo.as_mut().as_ptr(); assert!(a == b)` always holds.)
/// - `extend` must not change any contained data (but may extend with unspecified values)
pub unsafe trait DataBuf {
    /// Inner type of the buffer
    type Inner: Pod;

    /// Get the buffer slice as an immutable borrow
    fn as_ref(&self) -> &[MaybeUninit<Self::Inner>];
    /// Get the buffer slice as a mutable borrow
    fn as_mut(&mut self) -> &mut [MaybeUninit<Self::Inner>];

    /// Extend the buffer (fallible)
    fn extend(&mut self, len: usize) -> Result<(), ()>;

    /// Convert a byte count to a word count (rounding up)
    fn round_to_words(bytes: usize) -> usize {
        crate::round_to_words::<Self::Inner>(bytes)
    }
}

/// Trait that indicates that a type is valid for any bit pattern
pub unsafe trait Pod: Copy {
    /// Construct a new instance (sames as `Default::default`)
    fn default() -> Self;
}
macro_rules! impl_pod {
    ( $($t:ty),* ) => {
        $( unsafe impl Pod for $t { fn default() -> Self { 0 } } )*
    }
}
impl_pod! { u8, u16, u32, u64, u128, usize }

unsafe impl<T, U> DataBuf for &mut T
where
    U: Pod,
    T: DataBuf<Inner = U>,
{
    type Inner = T::Inner;
    fn as_ref(&self) -> &[MaybeUninit<Self::Inner>] {
        (**self).as_ref()
    }
    fn as_mut(&mut self) -> &mut [MaybeUninit<Self::Inner>] {
        (**self).as_mut()
    }
    fn extend(&mut self, len: usize) -> Result<(), ()> {
        (**self).extend(len)
    }
}

#[cfg(not(feature = "const_generics"))]
macro_rules! impl_databuf_array {
    ( $($n:expr),* ) => {
        $(unsafe impl<T: Pod> DataBuf for [MaybeUninit<T>; $n] {
            type Inner = T;
            fn as_ref(&self) -> &[MaybeUninit<Self::Inner>] {
                self
            }
            fn as_mut(&mut self) -> &mut [MaybeUninit<Self::Inner>] {
                self
            }
            fn extend(&mut self, len: usize) -> Result<(), ()> {
                if len > $n {
                    Err( () )
                }
                else {
                    Ok( () )
                }
            }
        })*
    }
}
#[cfg(not(feature = "const_generics"))]
impl_databuf_array! {
     0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
    10,11,12,13,14,15,16,17,18,19,
    20,21,22,23,24,25,26,27,28,29,
    30,31,
    32,48,
    64,96,
    128,192,
    256
}
/// Array-specific impl
#[cfg(feature = "const_generics")]
unsafe impl<T: Pod, const N: usize> DataBuf for [MaybeUninit<T>; N] {
    type Inner = T;
    fn as_ref(&self) -> &[MaybeUninit<Self::Inner>] {
        self
    }
    fn as_mut(&mut self) -> &mut [MaybeUninit<Self::Inner>] {
        self
    }
    fn extend(&mut self, len: usize) -> Result<(), ()> {
        if len > N {
            Err(())
        } else {
            Ok(())
        }
    }
}

/// Vector backed structures, can be used to auto-grow the allocation
///
/// ```
/// let mut buf = ::stack_dst::Fifo::<str, Vec<::std::mem::MaybeUninit<u8>>>::new();
/// buf.push_back_str("Hello world!");
/// buf.push_back_str("This is a very long string");
/// buf.push_back_str("The buffer should keep growing as it needs to");
/// for line in buf.iter() {
///   println!("{}", line);
/// }
/// ```
#[cfg(feature = "alloc")]
unsafe impl<T: Pod> crate::DataBuf for ::alloc::vec::Vec<MaybeUninit<T>> {
    type Inner = T;
    fn as_ref(&self) -> &[MaybeUninit<Self::Inner>] {
        self
    }
    fn as_mut(&mut self) -> &mut [MaybeUninit<Self::Inner>] {
        self
    }
    fn extend(&mut self, len: usize) -> Result<(), ()> {
        if len > self.len() {
            self.resize(len, MaybeUninit::uninit());
            let cap = self.capacity();
            self.resize(cap, MaybeUninit::uninit());
        }
        Ok(())
    }
}
