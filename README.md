# uX2: A better [uX](https://github.com/rust-ux/uX)

[![Crates.io](https://img.shields.io/crates/v/ux2)](https://crates.io/crates/ux2)
[![docs](https://img.shields.io/crates/v/ux2?color=yellow&label=docs)](https://docs.rs/ux2)
[![codecov](https://codecov.io/gh/JonathanWoollett-Light/ux2/branch/master/graph/badge.svg?token=II1xtnbCDX)](https://codecov.io/gh/JonathanWoollett-Light/ux2)

#### Non-standard integer types like `u7`, `u9`, `u10`, `u63`, `i7`, `i9` etc.

When non-standard-width integers are required in an application, the norm is to use a larger container and make sure the value is within range after manipulation. uX2 aims to take care of this once and for all by providing `u1`-`u127` and `i1`-`i127` types (depending on the enabled features) that offer safe arithmetic operations.

`<core::primitive::i32 as core::ops::Add<core::primitive::i32>>::add` can panic in `Debug` or overflow in `Release`, `<ux2::i32 as core::ops::Add<ux2::i32>>::add` cannot panic or overflow in `Debug` or `Release`, this is because it returns `ux2::i33`. This is applied for all operations and combinations of types in `ux2`. This allows for more thorough compile time type checking.

```rust
use rand::Rng;
let a = ux2::i4::try_from(3i8).unwrap();
let b = ux2::i8::from(rand::thread_rng().gen::<core::primitive::i8>());
let c: ux2::i9 = a + b;
let d: ux2::i4 = c % a;
let e: core::primitive::i8 = core::primitive::i8::from(d);
```

uX2 types take up as much space as the smallest integer type that can contain them.

## Features

The `8`, `16`, `32`, `64` and `128` features enable support up to the types of `i8`/`u8`, `i16`/`u16`, `i32`/`u32`, `i64`/`u64` and `i128`/`u128` respectively.

The compile times increase exponentially, 3s, 7s, 30s, 3m and 46m respectively.

<details>
  <summary>Click here for me details</summary>

    ```bash
    jonathan@jonathan-System-Product-Name:~/Projects/ux2/ux2$ cargo clean
    jonathan@jonathan-System-Product-Name:~/Projects/ux2/ux2$ cargo build --no-default-features --features 8
    Compiling proc-macro2 v1.0.56
    Compiling unicode-ident v1.0.8
    Compiling quote v1.0.26
    Compiling ux2-macros v0.7.0 (/home/jonathan/Projects/ux2/ux2-macros)
    Compiling ux2 v0.7.0 (/home/jonathan/Projects/ux2/ux2)
        Finished dev [unoptimized + debuginfo] target(s) in 3.00s
    jonathan@jonathan-System-Product-Name:~/Projects/ux2/ux2$ cargo clean
    jonathan@jonathan-System-Product-Name:~/Projects/ux2/ux2$ cargo build --no-default-features --features 16
    Compiling proc-macro2 v1.0.56
    Compiling quote v1.0.26
    Compiling unicode-ident v1.0.8
    Compiling ux2-macros v0.7.0 (/home/jonathan/Projects/ux2/ux2-macros)
    Compiling ux2 v0.7.0 (/home/jonathan/Projects/ux2/ux2)
        Finished dev [unoptimized + debuginfo] target(s) in 7.36s
    jonathan@jonathan-System-Product-Name:~/Projects/ux2/ux2$ cargo clean
    jonathan@jonathan-System-Product-Name:~/Projects/ux2/ux2$ cargo build --no-default-features --features 32
    Compiling proc-macro2 v1.0.56
    Compiling unicode-ident v1.0.8
    Compiling quote v1.0.26
    Compiling ux2-macros v0.7.0 (/home/jonathan/Projects/ux2/ux2-macros)
    Compiling ux2 v0.7.0 (/home/jonathan/Projects/ux2/ux2)
        Finished dev [unoptimized + debuginfo] target(s) in 29.96s
    jonathan@jonathan-System-Product-Name:~/Projects/ux2/ux2$ cargo clean
    jonathan@jonathan-System-Product-Name:~/Projects/ux2/ux2$ cargo build --no-default-features --features 64
    Compiling proc-macro2 v1.0.56
    Compiling unicode-ident v1.0.8
    Compiling quote v1.0.26
    Compiling ux2-macros v0.7.0 (/home/jonathan/Projects/ux2/ux2-macros)
    Compiling ux2 v0.7.0 (/home/jonathan/Projects/ux2/ux2)
        Finished dev [unoptimized + debuginfo] target(s) in 3m 26s
    jonathan@jonathan-System-Product-Name:~/Projects/ux2/ux2$ cargo clean
    jonathan@jonathan-System-Product-Name:~/Projects/ux2/ux2$ cargo build --no-default-features --features 128
    Compiling proc-macro2 v1.0.56
    Compiling unicode-ident v1.0.8
    Compiling quote v1.0.26
    Compiling ux2-macros v0.7.0 (/home/jonathan/Projects/ux2/ux2-macros)
    Compiling ux2 v0.7.0 (/home/jonathan/Projects/ux2/ux2)
        Finished dev [unoptimized + debuginfo] target(s) in 46m 22s
    ```
</details>

## Why does this exist? Why use this over `ux`?

I noticed [uX](https://github.com/rust-ux/uX) doesn't seem to be actively maintained and the current code
could use some big changes.

So I did what any reasonable developer does and completely re-invented the wheel.

Behold uX2, slightly better in almost every way.

- More functionality, with optional support for `serde`.
- Better documentation.
- Better CI (e.g. automated changelog)

I've already implemented some of the open issues from uX in this library e.g.
- https://github.com/rust-ux/uX/issues/55
- https://github.com/rust-ux/uX/issues/54
- https://github.com/rust-ux/uX/issues/53
- https://github.com/rust-ux/uX/issues/17

Why didn't I just post a PR on uX?
1. Review: The current PRs don't seem to be getting reviewed, I wasn't really confident a PR which completely changes the entire library would be merged.
2. Control: If the maintainer/s of uX are inactive there is nothing I can do, I cannot get PRs merged or fix issue, if I have control I can do this.
