//
// Implementation of the `DataBuf` trait
//

/// Trait used to represent a data buffer, typically you'll passs a `[usize; N]` array.
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

/// Fallback for when const generics aren't available
#[cfg(not(feature = "const_generics"))]
impl<T: Copy + Default + AsMut<[usize]> + AsRef<[usize]>> DataBuf for T {
    type Inner = usize;
    fn default() -> Self {
        Default::default()
    }
    fn extend(&mut self, _: usize) -> Result<(), ()> {
        Err(())
    }
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

/// Vector backed structures
#[cfg(all(feature = "const_generics", feature = "alloc"))]
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
