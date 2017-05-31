
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

