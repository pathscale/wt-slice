# wt-slice

High-performance extensions for ordered [`slice`](https://doc.rust-lang.org/stable/std/primitive.slice.html)s.

This is a fork of [superslice](https://github.com/alkis/superslice-rs). It preserves superslice's lower-bound, upper-bound, equal-range, and permutation APIs while adding exact-search experiments for WorkTablesIndex.

Licensed under APACHE-2.

### Installation

Until the fork is published, use its Git repository:

```toml
[dependencies]
wt-slice = { git = "https://github.com/pathscale/wt-slice" }
```

and augment `slice`s by using its `Ext` trait:

```rust
use wt_slice::*;
```

Now you can enjoy high performance of common algorithms on slices:

- `lower_bound`
- `upper_bound`
- `equal_range`

### Why isn't this part of the standard library?

Worry not, work is on the way:

- [X] Make `binary_search` as fast as ~~`fast_binary_search`~~:  https://github.com/rust-lang/rust/pull/45333
- [ ] Add `lower_bound`, `upper_bound`, `equal_range` to std: https://github.com/rust-lang/rfcs/issues/2184
