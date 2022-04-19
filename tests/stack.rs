use std::any::Any;

extern crate stack_dst;

type DstStack<T> = stack_dst::StackA<T, ::stack_dst::buffers::Ptr8>;

#[test]
// A trivial check that ensures that methods are correctly called
fn trivial_type() {
    let mut val = DstStack::<dyn PartialEq<u32>>::new();
    val.push_stable(1234, |p| p).unwrap();
    val.push_stable(1233, |p| p).unwrap();
    assert!(*val.top().unwrap() != 1234);
    assert!(*val.top().unwrap() == 1233);
    val.pop();
    assert!(*val.top().unwrap() == 1234);
    assert!(*val.top().unwrap() != 1233);
}

#[test]
fn strings() {
    let mut stack: DstStack<str> = DstStack::new();
    stack.push_str("\n").unwrap();
    stack.push_str("World").unwrap();
    stack.push_str(" ").unwrap();
    stack.push_str("Hello").unwrap();

    assert_eq!(stack.top(), Some("Hello"));
    stack.pop();
    assert_eq!(stack.top(), Some(" "));
    stack.pop();
    assert_eq!(stack.top(), Some("World"));
    stack.pop();
    stack.push_str("World").unwrap();
    stack.push_str("Cruel").unwrap();
    assert_eq!(stack.top(), Some("Cruel"));
    stack.pop();
    assert_eq!(stack.top(), Some("World"));
    stack.pop();
    assert_eq!(stack.top(), Some("\n"));
    stack.pop();
    assert_eq!(stack.top(), None);
}

#[test]
fn slices() {
    let mut stack: DstStack<[u8]> = DstStack::new();

    stack.push_cloned(b"123").unwrap();
    stack.push_cloned(b"").unwrap();
    stack.push_cloned(b"abcd").unwrap();
    assert_eq!(stack.top(), Some(b"abcd" as &[_]));
    stack.pop();
    assert_eq!(stack.top(), Some(b"" as &[_]));
    stack.pop();
    assert_eq!(stack.top(), Some(b"123" as &[_]));
    stack.pop();
    assert_eq!(stack.top(), None);
}

#[test]
fn limits() {
    let mut val = stack_dst::StackA::<dyn Any, ::stack_dst::buffers::Ptr2>::new();
    // Pushing when full
    val.push_stable(1usize, |p| p).unwrap();
    assert!(val.push_stable(2usize, |p| p).is_err());

    // Popping past empty (should stay empty)
    val.pop();
    assert!(val.is_empty());
    val.pop();
    assert!(val.is_empty());

    // Zero-sized types
    val.push_stable((), |p| p).unwrap();
    val.push_stable((), |p| p).unwrap();
    assert!(val.push_stable((), |p| p).is_err());
    val.pop();

    // Pushing a value when there is space, but no enough space for the entire value
    assert!(val.push_stable(1usize, |p| p).is_err());
    val.push_stable((), |p| p).unwrap();
}

#[test]
fn destructors() {
    struct DropWatch(::std::rc::Rc<::std::cell::Cell<usize>>);
    impl ::std::ops::Drop for DropWatch {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }

    let v: ::std::rc::Rc<::std::cell::Cell<_>> = Default::default();

    let mut stack = ::stack_dst::StackA::<dyn Any, ::stack_dst::buffers::Ptr8>::new();
    // Successful pushes shouldn't call destructors
    stack.push_stable(DropWatch(v.clone()), |p| p).ok().unwrap();
    assert_eq!(v.get(), 0);
    stack.push_stable(DropWatch(v.clone()), |p| p).ok().unwrap();
    assert_eq!(v.get(), 0);
    stack.push_stable(DropWatch(v.clone()), |p| p).ok().unwrap();
    assert_eq!(v.get(), 0);
    stack.push_stable(DropWatch(v.clone()), |p| p).ok().unwrap();
    assert_eq!(v.get(), 0);
    // Failed push should return the value (which will be dropped)
    assert!(stack.push_stable(DropWatch(v.clone()), |p| p).is_err());
    assert_eq!(v.get(), 1);

    // Pop a value, drop increases
    stack.pop();
    assert_eq!(v.get(), 2);
    // Drop the entire stack, the rest are dropped
    drop(stack);
    assert_eq!(v.get(), 2 + 3);
}

#[test]
fn slice_push_panic_safety() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNT: AtomicUsize = AtomicUsize::new(0);
    struct Sentinel(bool);
    impl Clone for Sentinel {
        fn clone(&self) -> Self {
            if self.0 {
                panic!();
            } else {
                Sentinel(self.0)
            }
        }
    }
    impl Drop for Sentinel {
        fn drop(&mut self) {
            COUNT.fetch_add(1, Ordering::SeqCst);
        }
    }
    let input = [Sentinel(false), Sentinel(true)];

    let _ = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
        let mut stack = ::stack_dst::StackA::<[Sentinel], ::stack_dst::buffers::Ptr8>::new();
        let _ = stack.push_cloned(&input);
    }));
    assert_eq!(COUNT.load(Ordering::SeqCst), 1);
}

#[test]
// Check that panic safety is maintained, even if the datatype isn't aligned to usize
fn slice_push_panic_safety_unaligned() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNT: AtomicUsize = AtomicUsize::new(0);
    struct Sentinel(bool);
    impl Clone for Sentinel {
        fn clone(&self) -> Self {
            if ! self.0 {
                panic!();
            } else {
                Sentinel(self.0)
            }
        }
    }
    impl Drop for Sentinel {
        fn drop(&mut self) {
            COUNT.fetch_add(1, Ordering::SeqCst);
        }
    }
    let input = [
        // 1 good followed by one bad
        Sentinel(true), Sentinel(false)
        ];

    let _ = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
        let mut stack = ::stack_dst::StackA::<[Sentinel], _>::with_buffer([::std::mem::MaybeUninit::new(0xFFu8); 32]);
        let _ = stack.push_cloned(&input);
    }));
    assert_eq!(COUNT.load(Ordering::SeqCst), 1);
}

#[cfg(not(feature="full_const_generics"))]
mod unaligned {
    use std::any::Any;
    use stack_dst::StackA;
    type Buf8_16 = ::stack_dst::buffers::ArrayBuf<u8, ::stack_dst::buffers::n::U16>;
    #[test] #[should_panic]
    fn push_stable() {
        let mut stack = StackA::<dyn Any, Buf8_16>::new();
        let _ = stack.push_stable(123u32, |v| v as _);
    }
    #[test] #[should_panic]
    #[cfg(feature = "unsize")]
    fn push() {
        let mut stack = StackA::<dyn Any, Buf8_16>::new();
        let _ = stack.push(123u32);
    }
    #[test] #[should_panic]
    fn push_cloned() {
        let mut stack = StackA::<[u32], Buf8_16>::new();
        let _ = stack.push_cloned(&[123u32]);
    }
    #[test] #[should_panic]
    fn push_copied() {
        let mut stack = StackA::<[u32], Buf8_16>::new();
        let _ = stack.push_copied(&[123u32]);
    }
    #[test] #[should_panic]
    fn push_from_iter() {
        let mut stack = StackA::<[u32], Buf8_16>::new();
        let _ = stack.push_from_iter(0..1);
    }
}
