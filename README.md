# SeqDiff - Diff between two sequences

Functions to get correspondence between two sequences like diff,
based on Myers' algorithm.

doc: https://docs.rs/seqdiff  
crates.io: https://crates.io/crates/seqdiff

## Examples

```rust
use seqdiff;

let (a2b, b2a) = seqdiff::diff(&[1, 2, 3], &[1, 3]);
assert_eq!(a2b, vec![Some(0), None, Some(1)]);
assert_eq!(b2a, vec![Some(0), Some(2)]);
```

See the [doc](https://docs.rs/seqdiff) for more info.
