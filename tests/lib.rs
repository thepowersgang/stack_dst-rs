
extern crate stack_dst;

use stack_dst::StackDST;

#[test]
// A trivial check that ensures that methods are correctly called
fn trivial_type()
{
	let val = StackDST::<PartialEq<u32>>::new( 1234u32 ).unwrap();
	assert!( *val == 1234 );
	assert!( *val != 1233 );
}

#[test]
// Create an instance with a Drop implementation, and ensure the drop handler fires when destructed
// This also ensures that lifetimes are correctly handled
fn ensure_drop()
{
	use std::cell::Cell;
	#[derive(Debug)]
	struct Struct<'a>(&'a Cell<bool>);
	impl<'a> Drop for Struct<'a> { fn drop(&mut self) { self.0.set(true); } }
	
	let flag = Cell::new(false);
	let val: StackDST<::std::fmt::Debug> = StackDST::new( Struct(&flag) ).unwrap();
	assert!(flag.get() == false);
	drop(val);
	assert!(flag.get() == true);
}

#[test]
fn many_instances()
{
	trait TestTrait {
		fn get_value(&self) -> u32;
	}
	
	#[inline(never)]
	fn instance_one() -> StackDST<TestTrait> {
		struct OneStruct(u32);
		impl TestTrait for OneStruct {
			fn get_value(&self) -> u32 { self.0 }
		}
		StackDST::new( OneStruct(12345) ).unwrap()
	}
	
	#[inline(never)]
	fn instance_two() -> StackDST<TestTrait> {
		struct TwoStruct;
		impl TestTrait for TwoStruct {
			fn get_value(&self) -> u32 { 54321 }
		}
		StackDST::new(TwoStruct).unwrap()
	}
	
	let i1 = instance_one();
	let i2 = instance_two();
	assert_eq!(i1.get_value(), 12345);
	assert_eq!(i2.get_value(), 54321);
}


#[test]
fn closure()
{
	let v1 = 1234u64;
	let c: StackDST<Fn()->String> = StackDST::new(|| format!("{}", v1)).unwrap();
	assert_eq!(c(), "1234");
}
