#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]
// https://doc.rust-lang.org/rustdoc/unstable-features.html#doc_auto_cfg-automatically-generate-doccfg

//! # uX2: A better [uX](https://github.com/rust-ux/uX)
//!
//! [![Crates.io](https://img.shields.io/crates/v/ux2)](https://crates.io/crates/ux2)
//! [![docs](https://img.shields.io/crates/v/ux2?color=yellow&label=docs)](https://docs.rs/ux2)
//! [![codecov](https://codecov.io/gh/JonathanWoollett-Light/ux2/branch/master/graph/badge.svg?token=II1xtnbCDX)](https://codecov.io/gh/JonathanWoollett-Light/ux2)
//!
//! #### Non-standard integer types like `u7`, `u9`, `u10`, `u63`, `i7`, `i9` etc.
//!
//! When non-standard-width integers are required in an application, the norm is to use a larger
//! container and make sure the value is within range after manipulation. uX2 aims to take care of
//! this once and for all by providing `u1`-`u127` and `i1`-`i127` types (depending on the enabled
//! features) that offer safe arithmetic operations.
//!
//! `<core::primitive::i32 as core::ops::Add<core::primitive::i32>>::add` can panic in `Debug` or
//! overflow in `Release`, `<ux2::i32 as core::ops::Add<ux2::i32>>::add` cannot panic or overflow in
//! `Debug` or `Release`, this is because it returns `ux2::i33`. This is applied for all operations
//! and combinations of types in `ux2`. This allows for more thorough compile time type checking.
//!
//! ```ignore
//! use rand::Rng;
//! let a = ux2::i4::try_from(3i8).unwrap();
//! let b = ux2::i8::from(rand::thread_rng().gen::<core::primitive::i8>());
//! let c: ux2::i9 = a + b;
//! let d: ux2::i4 = c % a;
//! let e: core::primitive::i8 = core::primitive::i8::from(d);
//! ```
//!
//! uX2 types take up as much space as the smallest integer type that can contain them.
//!
//! ## Why does this exist? Why use this over `ux`?
//!
//! I noticed [uX](https://github.com/rust-ux/uX) doesn't seem to be actively maintained and the current code
//! could use some big changes.
//!
//! So I did what any reasonable developer does and completely re-invented the wheel.
//!
//! Behold uX2, slightly better in almost every way.
//!
//! - More functionality, with optional support for `serde`.
//! - Better documentation.
//! - Better CI (e.g. automated changelog)
//!
//! I've already implemented some of the open issues from uX in this library e.g.
//! - <https://github.com/rust-ux/uX/issues/55>
//! - <https://github.com/rust-ux/uX/issues/54>
//! - <https://github.com/rust-ux/uX/issues/53>
//! - <https://github.com/rust-ux/uX/issues/17>
//!
//! Why didn't I just post a PR on uX?
//! 1. Review: The current PRs don't seem to be getting reviewed, I wasn't really confident a PR
//! which completely changes the entire library would be merged. 2. Control: If the maintainer/s of
//! uX are inactive there is nothing I can do, I cannot get PRs merged or fix issue, if I have
//! control I can do this.
//!
//! ## Features
//!
//! The `8`, `16`, `32`, `64` and `128` features enable support up to the types of `i8`/`u8`,
//! `i16`/`u16`, `i32`/`u32`, `i64`/`u64` and `i128`/`u128` respectively.
//!
//! The compile times increase exponentially, 3s, 7s, 30s, 3m and 46m respectively.

#[cfg(feature = "128")]
ux2_macros::generate_types!(128);
#[cfg(all(feature = "64", not(feature = "128")))]
ux2_macros::generate_types!(64);
#[cfg(all(feature = "32", not(feature = "64")))]
ux2_macros::generate_types!(32);
#[cfg(all(feature = "16", not(feature = "32")))]
ux2_macros::generate_types!(16);
#[cfg(all(feature = "8", not(feature = "16")))]
ux2_macros::generate_types!(8);

/// A mimic of [`std::num::TryFromIntError`] that can be constructed on stable.
#[derive(Debug, Eq, PartialEq)]
pub struct TryFromIntError;
impl core::fmt::Display for TryFromIntError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Failed `TryFrom`.")
    }
}
#[cfg(feature = "std")]
impl std::error::Error for TryFromIntError {}

/// A mimic of [`std::num::ParseIntError`] that can be constructed on stable.
#[derive(Debug, Eq, PartialEq)]
pub struct ParseIntError;
impl core::fmt::Display for ParseIntError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Failed `TryFrom`.")
    }
}
#[cfg(feature = "std")]
impl std::error::Error for ParseIntError {}

/// https://doc.rust-lang.org/std/primitive.array.html#method.split_array_mut
fn array_split_array_mut<T, const N: usize, const M: usize>(
    array: &mut [T; N],
) -> (&mut [T; M], &mut [T]) {
    slice_split_array_mut::<_, M>(&mut array[..])
}

/// https://doc.rust-lang.org/std/primitive.array.html#method.rsplit_array_mut
fn array_rsplit_array_mut<T, const N: usize, const M: usize>(
    array: &mut [T; N],
) -> (&mut [T], &mut [T; M]) {
    slice_rsplit_array_mut::<_, M>(&mut array[..])
}

/// https://doc.rust-lang.org/std/primitive.slice.html#method.rsplit_array_mut
fn slice_rsplit_array_mut<T, const N: usize>(slice: &mut [T]) -> (&mut [T], &mut [T; N]) {
    assert!(N <= slice.len());
    let (a, b) = slice.split_at_mut(slice.len() - N);
    // SAFETY: b points to [T; N]? Yes it's [T] of length N (checked by split_at_mut)
    unsafe { (a, &mut *(b.as_mut_ptr() as *mut [T; N])) }
}

/// https://doc.rust-lang.org/std/primitive.slice.html#method.split_array_mut
fn slice_split_array_mut<T, const N: usize>(slice: &mut [T]) -> (&mut [T; N], &mut [T]) {
    let (a, b) = slice.split_at_mut(N);
    // SAFETY: a points to [T; N]? Yes it's [T] of length N (checked by split_at_mut)
    unsafe { (&mut *(a.as_mut_ptr() as *mut [T; N]), b) }
}
