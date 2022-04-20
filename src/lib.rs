//! Support for storing dynamically-sized types within fixed-size allocations
//!
//! - The `Value` type provides a fixed size (7 word in the current version) buffer in which a trait object
//!   or array can be stored, without resorting to a heap allocation.
//! - The `Fifo` and `Stack` types provide collection types (first-in-first-out and last-in-first-out).
//!
//! # Examples
//! ## An unboxed any
//! As a quick example - The following wraps a 64-bit integer up in an inline DST using the Any trait.
//!
//! ```rust
//! # use std::any::Any;
//! # use stack_dst::ValueA;
//! #
//! let dst = ValueA::<dyn Any, ::stack_dst::buffers::Ptr2>::new_stable(1234u64, |p| p as _)
//!     .ok().expect("Integer did not fit in allocation");
//! println!("dst as u64 = {:?}", dst.downcast_ref::<u64>());
//! println!("dst as i8 = {:?}", dst.downcast_ref::<i8>());
//! ```
//!
//! ## Stack-allocated closure!
//! The following snippet shows how small (`'static`) closures can be returned using this crate
//!
//! ```rust
//! # use stack_dst::ValueA;
//! #
//! fn make_closure(value: u64) -> ValueA<dyn FnMut()->String, ::stack_dst::array_buf![u64; U2]> {
//!     ValueA::new_stable(move || format!("Hello there! value={}", value), |p| p as _)
//!         .ok().expect("Closure doesn't fit")
//! }
//! let mut closure = make_closure(666);
//! assert_eq!( (&mut *closure)(), "Hello there! value=666" );
//! ```
//!
//! ## Custom allocation sizes/types
//! If you need larger alignment, you can use a different type for the backing array. (Note, that metadata uses at least one slot in the array)
//!
//! This code panics, because i128 requires 8/16 byte alignment (usually)
//! ```should_panic
//! # use stack_dst::ValueA;
//! # use std::any::Any;
//! let v: ValueA<dyn Any, ::stack_dst::buffers::U8_32> = ValueA::new_stable(123i128, |p| p as _).unwrap();
//! ```
//! This works, because the backing buffer has sufficient alignment
//! ```rust
//! # use stack_dst::ValueA;
//! # use std::any::Any;
//! let v: ValueA<dyn Any, ::stack_dst::array_buf![u128; U2]> = ValueA::new_stable(123i128, |p| p as _).unwrap();
//! ```
//!
//! # Feature flags
//! ## `alloc` (default)
//! Provides the `StackDstA::new_or_boxed` method (if `unsize` feature is active too)
//! ## `const_generics` (default)
//! Uses value/constant generics to provide a slightly nicer API
//! ## `unsize` (optional)
//! Uses the nightly feature `unsize` to provide a more egonomic API (no need for the `|p| p` closures)
// //! ## `full_const_generics` (optional)
// //! Uses extended const generics to give compile time alignment errors
//!
#![cfg_attr(feature = "unsize", feature(unsize))] // needed for Unsize
#![cfg_attr(
    feature = "full_const_generics",
    feature(generic_const_exprs)
)]
#![cfg_attr(feature = "full_const_generics", allow(incomplete_features))]
#![no_std]
#![deny(missing_docs)]

use core::{mem, ptr, slice};
use ::core::mem::MaybeUninit;

// Internal helper
type BufSlice<T> = [MaybeUninit<T>];

#[cfg(miri)]
#[macro_use]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

extern crate generic_array;

mod data_buf;
pub use self::data_buf::DataBuf;
pub use self::data_buf::Pod;

pub use fifo::FifoA;
pub use stack::StackA;
pub use value::ValueA;

/// Shorthand for defining a array buffer
/// 
/// The array size must be a typenum unsigned integer (e.g `U8`)
/// E.g. `array_buf![u8; U32]` expands to `::stack_dst::buffers::ArrayBuf<u8, ::stack_dst::buffers::n::::U32>`
#[macro_export]
macro_rules! array_buf {
    ($t:ty; $n:ident) => { $crate::buffers::ArrayBuf<$t, $crate::buffers::n::$n> }
}

/// Type aliases for common buffer sizes and types
/// 
/// Some useful suggestions
/// - [Ptr8] is the semi-standard buffer for holding a single object (a good balance of space used)
/// - [Ptr2] is suitable for storing a single pointer and its vtable
pub mod buffers {
    use ::core::mem::MaybeUninit;

    /// A re-export of `typenum` for shorter names
    pub use ::generic_array::typenum as n;
    /// A buffer backing onto an array (used to provide default)
    pub struct ArrayBuf<T, N>
    where
        N: ::generic_array::ArrayLength<MaybeUninit<T>>,
    {
        inner: ::generic_array::GenericArray<MaybeUninit<T>, N>,
    }
    impl<T, N> AsRef<crate::BufSlice<T>> for ArrayBuf<T, N>
    where
        N: ::generic_array::ArrayLength<MaybeUninit<T>>,
    {
        fn as_ref(&self) -> &crate::BufSlice<T> {
            &self.inner
        }
    }
    impl<T, N> AsMut<crate::BufSlice<T>> for ArrayBuf<T, N>
    where
        N: ::generic_array::ArrayLength<MaybeUninit<T>>,
    {
        fn as_mut(&mut self) -> &mut crate::BufSlice<T> {
            &mut self.inner
        }
    }
    impl<T, N> ::core::default::Default for ArrayBuf<T, N>
    where
        N: ::generic_array::ArrayLength<MaybeUninit<T>>,
    {
        fn default() -> Self {
            ArrayBuf {
                // `unwarp` won't fail, lengths match
                inner: ::generic_array::GenericArray::from_exact_iter( (0 .. N::USIZE).map(|_| MaybeUninit::uninit()) ).unwrap(),
            }
        }
    }
    unsafe impl<T,N> crate::DataBuf for ArrayBuf<T, N>
    where
        T: crate::Pod,
        N: ::generic_array::ArrayLength<MaybeUninit<T>>,
    {
        type Inner = T;
        fn extend(&mut self, len: usize) -> Result<(), ()> {
            if len > N::USIZE {
                Err( () )
            }
            else {
                Ok( () )
            }
        }
    }

    /// 8 pointers (32/64 bytes, with pointer alignment)
    pub type Ptr8 = ArrayBuf<*const (), n::U8>;
    /// 64 bytes, 64-bit alignment
    pub type U64_8 = ArrayBuf<u64, n::U8>;
    /// 32 bytes, 8-bit alignment
    pub type U8_32 = ArrayBuf<u8, n::U32>;

    /// 16 bytes, 64-bit alignment
    pub type U64_2 = ArrayBuf<u64, n::U2>;
    
    /// 16 pointers (64/128 bytes, with pointer alignment)
    pub type Ptr16 = ArrayBuf<*const (), n::U16>;
    
    /// Two pointers, useful for wrapping a pointer along with a vtable
    pub type Ptr2 = ArrayBuf<*const (), n::U2>;
    /// One pointer, can only store the vtable
    pub type Ptr1 = ArrayBuf<*const (), n::U1>;

    /// Dyanamically allocated buffer with 8-byte alignment
    #[cfg(feature="alloc")]
    pub type U64Vec = ::alloc::vec::Vec<MaybeUninit<u64>>;
    /// Dyanamically allocated buffer with 1-byte alignment
    #[cfg(feature="alloc")]
    pub type U8Vec = ::alloc::vec::Vec<MaybeUninit<u8>>;
    /// Dyanamically allocated buffer with pointer alignment
    #[cfg(feature="alloc")]
    pub type PtrVec = ::alloc::vec::Vec<MaybeUninit<*const ()>>;
}

/// Implementation of the FIFO list structure
pub mod fifo;
/// Implementation of the LIFO stack structure
pub mod stack;
/// Implementation of the single-value structure
pub mod value;

#[cfg(feature = "const_generics")]
/// A single LIFO stack of DSTs
pub type Stack<T /*: ?Sized*/, const N: usize /* = 16*/> = StackA<T, [MaybeUninit<usize>; N]>;
#[cfg(feature = "const_generics")]
/// A single dynamically-sized value
pub type Value<T /*: ?Sized*/, const N: usize /* = {8+1}*/> = ValueA<T, [MaybeUninit<usize>; N]>;
#[cfg(feature = "const_generics")]
/// A FIFO queue of DSTs
pub type Fifo<T /*: ?Sized*/, const N: usize /* = {8+1}*/> = FifoA<T, [MaybeUninit<usize>; N]>;

fn decompose_pointer<T: ?Sized>(mut ptr: *const T) -> (*const (), usize, [usize; 3]) {
    let addr = ptr as *const ();
    let rv = mem_as_slice(&mut ptr);
    let mut vals = [0; 3];
    assert!(
        rv[0] == addr as usize,
        "BUG: Pointer layout is not (data_ptr, info...)"
    );
    vals[..rv.len()-1].copy_from_slice(&rv[1..]);
    (addr, rv.len()-1, vals,)
}

fn mem_as_slice<T>(ptr: &mut T) -> &mut [usize] {
    assert!(mem::size_of::<T>() % mem::size_of::<usize>() == 0);
    assert!(mem::align_of::<T>() % mem::align_of::<usize>() == 0);
    let words = mem::size_of::<T>() / mem::size_of::<usize>();
    // SAFE: Points to valid memory (a raw pointer)
    unsafe { slice::from_raw_parts_mut(ptr as *mut _ as *mut usize, words) }
}

/// Re-construct a fat pointer
unsafe fn make_fat_ptr<T: ?Sized, W: Pod>(data_ptr: *mut (), meta_vals: &BufSlice<W>) -> *mut T {
    #[repr(C)]
    #[derive(Copy,Clone)]
    struct Raw {
        ptr: *const (),
        meta: [usize; 4],
    }
    union Inner<T: ?Sized> {
        ptr: *mut T,
        raw: Raw,
    }
    let mut rv = Inner { raw: Raw { ptr: data_ptr, meta: [0; 4] } };
    assert!(meta_vals.len() * mem::size_of::<W>() % mem::size_of::<usize>() == 0);
    assert!(meta_vals.len() * mem::size_of::<W>() <= 4 * mem::size_of::<usize>());
    ptr::copy(
        meta_vals.as_ptr() as *const u8,
        rv.raw.meta.as_mut_ptr() as *mut u8,
        meta_vals.len() * mem::size_of::<W>()
        );
    let rv = rv.ptr;
    assert_eq!(rv as *const (), data_ptr as *const ());
    rv
}
/// Write metadata (abstraction around `ptr::copy`)
fn store_metadata<W: Pod>(dst: &mut BufSlice<W>, meta_words: &[usize]) {
    let n_bytes = meta_words.len() * mem::size_of::<usize>();
    assert!(n_bytes <= dst.len() * mem::size_of::<W>(),
        "nbytes [{}] <= dst.len() [{}] * sizeof [{}]", n_bytes, dst.len(), mem::size_of::<W>());
    unsafe {
        ptr::copy(
            meta_words.as_ptr() as *const u8,
            dst.as_mut_ptr() as *mut u8,
            n_bytes,
        );
    }
}

fn round_to_words<T>(len: usize) -> usize {
    (len + mem::size_of::<T>() - 1) / mem::size_of::<T>()
}

/// Calls a provided function to get a fat pointer version of `v` (and checks that the returned pointer is sane)
fn check_fat_pointer<U, T: ?Sized>(v: &U, get_ref: impl FnOnce(&U) -> &T) -> &T {
    let ptr: &T = get_ref(v);
    assert_eq!(
        ptr as *const _ as *const u8, v as *const _ as *const u8,
        "MISUSE: Closure returned different pointer"
    );
    assert_eq!(
        mem::size_of_val(ptr),
        mem::size_of::<U>(),
        "MISUSE: Closure returned a subset pointer"
    );
    ptr
}

/// Push items to a list using a generator function to get the items
/// - `meta`  - Metadata slot (must be 1 usize long)
/// - `data`  - Data slot, must be at least `count * sizeof(T)` long
/// - `count` - Number of items to insert
/// - `gen`   - Generator function (is passed the current index)
/// - `reset_slot` - A slot updated with `reset_value` when a panic happens before push is complete
/// - `reset_value` - Value used in `reset_slot`
///
/// This provides a panic-safe push as long as `reset_slot` and `reset_value` undo the allocation operation
unsafe fn list_push_gen<T, W: Pod>(meta: &mut BufSlice<W>, data: &mut BufSlice<W>, count: usize, mut gen: impl FnMut(usize)->T, reset_slot: &mut usize, reset_value: usize)
{
    /// Helper to drop/zero all pushed items, and reset data structure state if there's a panic
    struct PanicState<'a, T>(*mut T, usize, &'a mut usize, usize);
    impl<'a, T> ::core::ops::Drop for PanicState<'a, T> {
        fn drop(&mut self) {
            if self.0.is_null() {
                return ;
            }
            // Reset the state of the data structure (leaking items)
            *self.2 = self.3;
            // Drop all partially-populated items
            unsafe {
                while self.1 != 0 {
                    ptr::drop_in_place(&mut *self.0);
                    ptr::write_bytes(self.0 as *mut u8, 0, mem::size_of::<T>());
                    self.0 = self.0.offset(1);
                    self.1 -= 1;
                }
            }
        }
    }

    let mut ptr = data.as_mut_ptr() as *mut T;
    let mut clr = PanicState(ptr, 0, reset_slot, reset_value);
    for i in 0 .. count {
        let val = gen(i);
        ptr::write(ptr, val);
        ptr = ptr.offset(1);
        clr.1 += 1;
    }
    clr.0 = ptr::null_mut();    // Prevent drops and prevent reset
    // Save the length once everything has been written
    crate::store_metadata(meta, &[count]);
}

/// Marker trait used to check alignment
pub unsafe trait AlignmentValid {
    #[doc(hidden)]
    fn check();
}
#[cfg(feature = "full_const_generics")]
unsafe impl<S, L> AlignmentValid for (S, L)
where
    [(); mem::align_of::<L>() - mem::align_of::<S>()]: Sized,
{
    fn check() {}
}
#[cfg(not(feature = "full_const_generics"))]
unsafe impl<S, L> AlignmentValid for (S, L) {
    fn check() {
        assert!(
            mem::align_of::<S>() <= mem::align_of::<L>(),
            "TODO: Enforce alignment >{} (requires {})",
            mem::align_of::<L>(),
            mem::align_of::<S>()
        );
    }
}

/*
#[cfg(doctest)]
#[doc=include_str!("../README.md")]
pub mod readme {
}
*/
