extern crate stack_dst;

use stack_dst::Value;

#[test]
// A trivial check that ensures that methods are correctly called
fn trivial_type() {
    let val = Value::<dyn PartialEq<u32>, 2>::new_stable(1234u32, |p| p).unwrap();
    assert!(*val == 1234);
    assert!(*val != 1233);
}

#[test]
// Create an instance with a Drop implementation, and ensure the drop handler fires when destructed
// This also ensures that lifetimes are correctly handled
fn ensure_drop() {
    use std::cell::Cell;
    #[derive(Debug)]
    struct Struct<'a>(&'a Cell<bool>);
    impl<'a> Drop for Struct<'a> {
        fn drop(&mut self) {
            self.0.set(true);
        }
    }

    let flag = Cell::new(false);
    let val = Value::<dyn std::fmt::Debug, 2>::new_stable(Struct(&flag), |p| p).unwrap();
    assert!(flag.get() == false);
    drop(val);
    assert!(flag.get() == true);
}

#[test]
fn many_instances() {
    trait TestTrait {
        fn get_value(&self) -> u32;
    }

    #[inline(never)]
    fn instance_one() -> Value<dyn TestTrait, 2> {
        #[derive(Debug)]
        struct OneStruct(u32);
        impl TestTrait for OneStruct {
            fn get_value(&self) -> u32 {
                self.0
            }
        }
        Value::new_stable(OneStruct(12345), |p| p as _).unwrap()
    }

    #[inline(never)]
    fn instance_two() -> Value<dyn TestTrait, 2> {
        #[derive(Debug)]
        struct TwoStruct;
        impl TestTrait for TwoStruct {
            fn get_value(&self) -> u32 {
                54321
            }
        }
        Value::new_stable(TwoStruct, |p| p as _).unwrap()
    }

    let i1 = instance_one();
    let i2 = instance_two();
    assert_eq!(i1.get_value(), 12345);
    assert_eq!(i2.get_value(), 54321);
}

#[test]
fn closure() {
    let v1 = 1234u64;
    let c: Value<dyn Fn() -> String, 8> = Value::new_stable(|| format!("{}", v1), |p| p as _)
        .map_err(|_| "Oops")
        .unwrap();
    assert_eq!(c(), "1234");
}

#[test]
fn oversize() {
    use std::any::Any;
    const MAX_SIZE_PTRS: usize = 8;
    assert!(Value::<dyn Any, 9>::new_stable([0usize; MAX_SIZE_PTRS], |p| p).is_ok());
    assert!(Value::<dyn Any, 9>::new_stable([0usize; MAX_SIZE_PTRS + 1], |p| p).is_err());
}

#[test]
fn option() {
    use std::any::Any;
    assert!(Some(Value::<dyn Any, 3>::new_stable("foo", |p| p).unwrap()).is_some());
}


#[test] #[should_panic]
fn stable_closure_different_pointer() {
    use std::fmt::Debug;
	static BIG_VALUE: [i32; 4] = [0,0,0,0];
	// Type confusion via a different pointer
	let _ = Value::<dyn Debug, 9>::new_stable(123, |_| &BIG_VALUE as &dyn Debug);
}
#[test] #[should_panic]
fn stable_closure_subset() {
    use std::fmt::Debug;
	let _ = Value::<dyn Debug, 9>::new_stable( (1,2), |v| &v.0 as &dyn Debug);
}
