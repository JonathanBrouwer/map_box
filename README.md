# Map Box

[![github](https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github)](https://github.com/JonathanBrouwer/map_box)
&ensp;[![crates-io](https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust)](https://crates.io/crates/map_box)
&ensp;[![docs-rs](https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs)](https://docs.rs/map_box)

Map the value in a Box, re-using the allocation when possible.

For example, this code will not re-allocate.

```rust
use map_box::Map;

let b = Box::new(42u64);
let b = b.map(|v| v as i64);
```

The signature of `map` is:
```rust
impl<T1> Box<T1> {
    fn map<T2>(self, f: impl FnMut(T1) -> T2) -> Box<T2>;
}
```


## Limitations

If the alignment requirements of the type changes, even if the alignment becomes lower, the Box needs to be reallocated.
This is because:
1. [alloc::dealloc](https://doc.rust-lang.org/stable/std/alloc/trait.GlobalAlloc.html#tymethod.dealloc) requires the layout to be identical to the layout that the allocation was made with
2. [alloc::realloc](https://doc.rust-lang.org/stable/std/alloc/trait.GlobalAlloc.html#method.realloc) only takes a new size, it cannot change the layout of the allocation downwards
