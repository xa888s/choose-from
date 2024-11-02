# Choose-from
[![Documentation](https://docs.rs/choose-from/badge.svg)](https://docs.rs/choose-from)

Simple Rust library for enforcing values are chosen from a set of values, using const generics,
lifetimes, and more. Please see the [docs](https://docs.rs/choose-from) for more information.

Example usage:
```rust
use choose_from::select_from_fixed;
let choices = ["Hi", "how", "are ya?"];

let chosen = select_from_fixed(choices).with(|[first, second, third]| {
    // the provided choices allow inspection of the values
    assert_eq!(*first, "Hi");
    
    // this is our selection
    [first, third]
});

assert_eq!(chosen, ["Hi", "are ya?"]);
```