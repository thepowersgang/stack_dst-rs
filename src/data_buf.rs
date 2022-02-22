//
// Implementation of the `DataBuf` trait
//

/// Trait used to represent a data buffer, typically you'll passs a `[usize; N]` array.
///
/// Can also provide a `Vec<T>` (if the `alloc` feature is enabled) which will grow as-needed
pub trait DataBuf: AsMut<[Self::Inner]> + AsRef<[Self::Inner]> {
    /// Inner type of the buffer
    type Inner: Pod;
    /// Create a default/new buffer
    fn default() -> Self;
    /// Extend the buffer (fallible)
    fn extend(&mut self, len: usize) -> Result<(), ()>;

    /// Convert a byte count to a word count (rounding up)
    fn round_to_words(bytes: usize) -> usize {
        crate::round_to_words::<Self::Inner>(bytes)
    }
}

/// Trait that indicates that a type is valid for any bit pattern
pub unsafe trait Pod: Copy + Default {}
macro_rules! impl_pod {
    ( $($t:ty),* ) => {
        $( unsafe impl Pod for $t {} )*
    }
}
impl_pod! { u8, u16, u32, u64, u128, usize }

#[cfg(not(feature = "const_generics"))]
macro_rules! impl_databuf_array {
    ( $($n:expr),* ) => {
        $(impl<T: Pod> DataBuf for [T; $n] {
            type Inner = T;
            fn default() -> Self {
                [Default::default(); $n]
            }
            fn extend(&mut self, _: usize) -> Result<(), ()> {
                Err(())
            }
        })
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
impl<T: Pod, const N: usize> DataBuf for [T; N] {
    type Inner = T;
    fn default() -> Self {
        [Default::default(); N]
    }
    fn extend(&mut self, _: usize) -> Result<(), ()> {
        Err(())
    }
}

/// Vector backed structures, can be used to auto-grow the allocation
///
/// ```
/// let mut buf = ::stack_dst::FifoA::<str, Vec<u8>>::new();
/// buf.push_back_str("Hello world!");
/// buf.push_back_str("This is a very long string");
/// buf.push_back_str("The buffer should keep growing as it needs to");
/// for line in buf.iter() {
///   println!("{}", line);
/// }
/// ```
#[cfg(all(feature = "alloc"))]
impl<T: Pod> crate::DataBuf for ::alloc::vec::Vec<T> {
    type Inner = T;
    fn default() -> Self {
        ::alloc::vec::Vec::new()
    }
    fn extend(&mut self, len: usize) -> Result<(), ()> {
        if len > self.len() {
            self.resize(len, Default::default());
            let cap = self.capacity();
            self.resize(cap, Default::default());
        }
        Ok(())
    }
}
