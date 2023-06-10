# uX2: A better [uX](https://github.com/rust-ux/uX)

[![Crates.io](https://img.shields.io/crates/v/ux2)](https://crates.io/crates/ux2)
[![docs](https://img.shields.io/crates/v/ux2?color=yellow&label=docs)](https://docs.rs/ux2)
[![codecov](https://codecov.io/gh/JonathanWoollett-Light/ux2/branch/master/graph/badge.svg?token=II1xtnbCDX)](https://codecov.io/gh/JonathanWoollett-Light/ux2)

#### Non-standard integer types like `u7`, `u9`, `u10`, `u63`, `i7`, `i9` etc.

When non-standard-width integers are required in an application, the norm is to use a larger container and make sure the value is within range after manipulation. uX aims to take care of this once and for all by:
 - Providing `u1`-`u127` and `i1`-`i127` types that should behave as similar as possible to the built in rust types
     - The methods of the defined types are the same as for the built in types (far from all methods are implemented at this point, but fill out an issue or create a PR if something essential for you is missing)
     - Overflow will panic in debug and wrap in release.
 - All lossless infallible conversions are possible by using `From`. 
 - All lossless fallible conversions are possible by using `TryFrom`.

The uX types take up as much space as the smallest integer type that can contain them.

## Why does this exist? Why use this over `ux`?

I noticed [uX](https://github.com/rust-ux/uX) doesn't seem to be actively maintained and the current code
could use some big changes.

So I did what any reasonable developer does and completely re-invented the wheel.

Behold uX2, slightly better in pretty much every way.

- More functionality, more closely matching the standard library, with optional support for `serde` and `num-traits`
- Better documentation.
- Better CI (e.g. automated changelog)
- Less code (1435 lines vs 2805 lines).
- Less memory (2.84kB vs 17.3kB).

I've already implemented some of the open issues from uX in this library e.g.
- https://github.com/rust-ux/uX/issues/55
- https://github.com/rust-ux/uX/issues/54
- https://github.com/rust-ux/uX/issues/53
- https://github.com/rust-ux/uX/issues/17
- https://github.com/rust-ux/uX/issues/19

Why didn't I just post a PR on uX?
1. Review: The current PRs don't seem to be getting reviewed, I wasn't really confident a PR which completely changes the entire library would be merged.
2. Control: If the maintainer/s of uX are inactive there is nothing I can do, I cannot get PRs merged or fix issue, if I have control I can do this.
