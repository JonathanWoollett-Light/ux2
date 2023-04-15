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
