extern crate stack_dst;

type DstFifo<T> = stack_dst::FifoA<T, [usize; 8]>;

#[test]
// A trivial check that ensures that methods are correctly called
fn trivial_type() {
    let mut val = DstFifo::<dyn PartialEq<u32>>::new();
    val.push_back_stable(1234, |p| p).unwrap();
    val.push_back_stable(1233, |p| p).unwrap();
    assert!(*val.front().unwrap() == 1234);
    assert!(*val.front().unwrap() != 1233);
    val.pop_front();
    assert!(*val.front().unwrap() != 1234);
    assert!(*val.front().unwrap() == 1233);
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
            let _v = COUNT.fetch_add(1, Ordering::SeqCst);
            //assert!( _v == 0 );
        }
    }
    let input = [Sentinel(false), Sentinel(true)];

    let _ = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
        let mut stack = DstFifo::<[Sentinel]>::new();
        let _ = stack.push_cloned(&input);
    }));
    assert_eq!(COUNT.load(Ordering::SeqCst), 1);
}
