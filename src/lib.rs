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
//! let dst = ValueA::<dyn Any, [usize; 2]>::new_stable(1234u64, |p| p as _)
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
//! fn make_closure(value: u64) -> ValueA<dyn FnMut()->String, [usize; 3]> {
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
//! let v: ValueA<dyn Any, [u32; 6]> = ValueA::new_stable(123i128, |p| p as _).unwrap();
//! ```
//! This works, because the backing buffer has sufficient alignment
//! ```rust
//! # use stack_dst::ValueA;
//! # use std::any::Any;
//! let v: ValueA<dyn Any, [u128; 2]> = ValueA::new_stable(123i128, |p| p as _).unwrap();
//! ```
//!
//! # Features
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

#[cfg(feature = "alloc")]
extern crate alloc;

mod data_buf;
pub use self::data_buf::DataBuf;
pub use self::data_buf::Pod;

pub use list::FifoA;
pub use stack::StackA;
pub use value::ValueA;

/// Implementation of the FIFO list structure
pub mod list;
/// Implementation of the LIFO stack structure
pub mod stack;
/// Implementation of the single-value structure
pub mod value;

#[cfg(feature = "const_generics")]
/// A single LIFO stack of DSTs
pub type Stack<T /*: ?Sized*/, const N: usize /* = 16*/> = StackA<T, [usize; N]>;
#[cfg(feature = "const_generics")]
/// A single dynamically-sized value
pub type Value<T /*: ?Sized*/, const N: usize /* = {8+1}*/> = ValueA<T, [usize; N]>;
#[cfg(feature = "const_generics")]
/// A FIFO queue of DSTs
pub type Fifo<T /*: ?Sized*/, const N: usize /* = {8+1}*/> = FifoA<T, [usize; N]>;

/// Obtain mutable access to a pointer's words
fn ptr_as_slice<T: ?Sized>(ptr: &mut *const T) -> &mut [usize] {
    let addr = *ptr as *const u8 as usize;
    let rv = mem_as_slice(ptr);
    assert!(
        rv[0] == addr,
        "BUG: Pointer layout is not (data_ptr, info...)"
    );
    rv
}
fn mem_as_slice<T>(ptr: &mut T) -> &mut [usize] {
    assert!(mem::size_of::<T>() % mem::size_of::<usize>() == 0);
    let words = mem::size_of::<T>() / mem::size_of::<usize>();
    // SAFE: Points to valid memory (a raw pointer)
    unsafe { slice::from_raw_parts_mut(ptr as *mut _ as *mut usize, words) }
}

/// Re-construct a fat pointer
unsafe fn make_fat_ptr<T: ?Sized, W: Copy>(data_ptr: usize, meta_vals: &[W]) -> *mut T {
    // I'd love to use a union, but can't get the right array size for it.
    let mut rv = mem::MaybeUninit::<*mut T>::uninit();
    {
        let s = mem_as_slice(&mut rv);
        s[0] = data_ptr;
        ptr::copy(
            meta_vals.as_ptr() as *const u8,
            s[1..].as_mut_ptr() as *mut u8,
            (s.len() - 1) * mem::size_of::<usize>(),
        );
    }
    let rv = rv.assume_init();
    assert_eq!(rv as *const (), data_ptr as *const ());
    rv
}
/// Write metadata (abstraction around `ptr::copy`)
fn store_metadata<W: Copy>(dst: &mut [W], meta_words: &[usize]) {
    let n_bytes = meta_words.len() * mem::size_of::<usize>();
    assert!(n_bytes <= dst.len() * mem::size_of::<W>());
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

unsafe fn list_push_cloned<T: Clone, W: Copy + Default>(meta: &mut [W], data: &mut [W], v: &[T]) {
    // Prepare the slot with zeros (as if it's an empty slice)
    // The length is updated as each item is written
    // - This ensures that there's no drop issues during write
    // - If a panic occurs, the drop will pop a bunch of empty lists after this one
    assert!(mem::size_of_val(v) <= mem::size_of_val(data));
    crate::store_metadata(meta, &[0]);
    for v in data.iter_mut() {
        *v = Default::default();
    }

    let mut ptr = data.as_mut_ptr() as *mut T;
    for (i, val) in v.iter().enumerate() {
        ptr::write(ptr, val.clone());
        crate::store_metadata(meta, &[1 + i]);
        ptr = ptr.offset(1);
    }
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
