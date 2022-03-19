# stack_dst

Inline (aka stack-allocated) dynamically-sized types, and collections of dyanmically-sized types using the same logic

# Overview
This crate provides ways of storing DSTs directly within an allocation.

# Basic usage
This crate covers two primary usecases
- `Value` allows storing (and returning) a single DST within a fixed-size allocation
- `Stack` and `Fifo` allow heterogeneous collections without needing to box each object.

# Example

## Unboxed closure
One of the most obvious uses is to allow returning capturing closures without having to box them. In the example below, the closure
takes ownership of `value`, and is then returned using a `Value`
```rust
use stack_dst::ValueA;

// The closure is stored in two 64-bit integers (one for the vtable, the other for the value)
fn make_closure(value: u64) -> ValueA<dyn Fn()->String, [u64; 2]> {
    if value < 0x10000 {
        ValueA::new_stable(move || format!("Hello there! value={}", value), |v| v as _).ok().expect("Closure doesn't fit")
    }
    else {
        ValueA::new_stable(move || format!("Hello there! value={:#x}", value), |v| v as _).ok().expect("Closure doesn't fit")
    }
}
let closure = make_closure(12);
assert_eq!( closure(), "Hello there! value=12" );
```

# Status
- Works for most test cases
- miri is happy with it
- Not rigourously tested across platforms

# Minimum rust version
- Uses `MaybeUninit`, so requires at least 1.36

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
