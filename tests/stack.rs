use std::any::Any;

extern crate stack_dst;

type DstStack<T> = stack_dst::StackA<T, [usize; 8]>;

#[test]
// A trivial check that ensures that methods are correctly called
fn trivial_type()
{
	let mut val = DstStack::<PartialEq<u32>>::new();
	val.push(1234).unwrap();
	val.push(1233).unwrap();
	assert!( *val.top().unwrap() != 1234 );
	assert!( *val.top().unwrap() == 1233 );
	val.pop();
	assert!( *val.top().unwrap() == 1234 );
	assert!( *val.top().unwrap() != 1233 );
}

#[test]
fn strings()
{
	let mut stack: DstStack<str> = DstStack::new();
	stack.push_str("\n").unwrap();
	stack.push_str("World").unwrap();
	stack.push_str(" ").unwrap();
	stack.push_str("Hello").unwrap();

	assert_eq!( stack.top(), Some("Hello") ); stack.pop();
	assert_eq!( stack.top(), Some(" ") );     stack.pop();
	assert_eq!( stack.top(), Some("World") ); stack.pop();
	stack.push_str("World").unwrap();
	stack.push_str("Cruel").unwrap();
	assert_eq!( stack.top(), Some("Cruel") ); stack.pop();
	assert_eq!( stack.top(), Some("World") ); stack.pop();
	assert_eq!( stack.top(), Some("\n") );    stack.pop();
	assert_eq!( stack.top(), None );
}

#[test]
fn slices()
{
	let mut stack: DstStack<[u8]> = DstStack::new();

	stack.push_cloned(b"123").unwrap();
	stack.push_cloned(b"").unwrap();
	stack.push_cloned(b"abcd").unwrap();
	assert_eq!( stack.top(), Some(b"abcd" as &[_]) ); stack.pop();
	assert_eq!( stack.top(), Some(b"" as &[_]) ); stack.pop();
	assert_eq!( stack.top(), Some(b"123" as &[_]) );    stack.pop();
	assert_eq!( stack.top(), None );
}

#[test]
fn limits()
{
	let mut val = stack_dst::StackA::<Any, [usize; 2]>::new();
	// Pushing when full
	val.push(1usize).unwrap();
	assert!(val.push(2usize).is_err());

	// Popping past empty (should stay empty)
	val.pop();
	assert!(val.is_empty());
	val.pop();
	assert!(val.is_empty());


	// Zero-sized types
	val.push( () ).unwrap();
	val.push( () ).unwrap();
	assert!( val.push( () ).is_err() );
	val.pop();

	// Pushing a value when there is space, but no enough space for the entire value
	assert!(val.push(1usize).is_err());
	val.push( () ).unwrap();
}

#[test]
fn destructors()
{
	struct DropWatch(::std::rc::Rc<::std::cell::Cell<usize>>);
	impl ::std::ops::Drop for DropWatch {
		fn drop(&mut self) {
			self.0.set( self.0.get() + 1 );
		}
	}

	let v: ::std::rc::Rc<::std::cell::Cell<_>> = Default::default();

	let mut stack = ::stack_dst::StackA::<Any, [usize; 8]>::new();
	// Successful pushes shouldn't call destructors
	stack.push( DropWatch(v.clone()) ).ok().unwrap();
	assert_eq!(v.get(), 0);
	stack.push( DropWatch(v.clone()) ).ok().unwrap();
	assert_eq!(v.get(), 0);
	stack.push( DropWatch(v.clone()) ).ok().unwrap();
	assert_eq!(v.get(), 0);
	stack.push( DropWatch(v.clone()) ).ok().unwrap();
	assert_eq!(v.get(), 0);
	// Failed push should return the value (which will be dropped)
	assert!( stack.push( DropWatch(v.clone()) ).is_err() );
	assert_eq!(v.get(), 1);

	// Pop a value, drop increases
	stack.pop();
	assert_eq!(v.get(), 2);
	// Drop the entire stack, the rest are dropped
	drop(stack);
	assert_eq!(v.get(), 2+3);
}


