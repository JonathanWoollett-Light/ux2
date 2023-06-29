use std::str::FromStr;

use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote};

#[proc_macro]
pub fn generate_types(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut items = item.into_iter();
    let max = match items.next() {
        Some(proc_macro::TokenTree::Literal(literal)) => {
            u8::from_str(&literal.to_string()).unwrap()
        }
        x => panic!("Missing max size, found: {x:?}"),
    };
    assert!(items.next().is_none());

    let output = (1..=max).map(|size| {
        let inner_size = std::cmp::max(size.next_power_of_two(),8);
        let inner_bytes = (inner_size / 8) as usize;
        let bits = size as u32;
        let bytes = match size % 8 {
            0 => size / 8,
            1..=7 => (size / 8) + 1,
            _ => unreachable!()
        } as usize;

        let unsigned_doc = format!(" The {size}-bit unsigned integer type.");
        let unsigned_inner_ident = format_ident!("u{inner_size}");
        let unsigned_inner_ident = quote! { core::primitive::#unsigned_inner_ident };
        let unsigned_ident = format_ident!("u{size}");
        let unsigned_max = || 2u128.pow(u32::from(size)) - 1;
        let unsigned_max_ident = match size {
            0..=7 => Literal::u8_suffixed(unsigned_max() as u8),
            8 => Literal::u8_suffixed(u8::MAX),
            9..=15 => Literal::u16_suffixed(unsigned_max() as u16),
            16 => Literal::u16_suffixed(u16::MAX),
            17..=31 => Literal::u32_suffixed(unsigned_max() as u32),
            32 => Literal::u32_suffixed(u32::MAX),
            33..=63 => Literal::u64_suffixed(unsigned_max() as u64),
            64 => Literal::u64_suffixed(u64::MAX),
            65..=127 => Literal::u128_suffixed(unsigned_max()),
            128 => Literal::u128_suffixed(u128::MAX),
            129.. => unreachable!()
        };
        let unsigned_min_ident = match size {
            0..=8 => Literal::u8_suffixed(0),
            9..=16 => Literal::u16_suffixed(0),
            17..=32 => Literal::u32_suffixed(0),
            33..=64 => Literal::u64_suffixed(0),
            65..=128 => Literal::u128_suffixed(0),
            129.. => unreachable!()
        };
        let unsigned_one = match size {
            0..=8 => Literal::u8_suffixed(1),
            9..=16 => Literal::u16_suffixed(1),
            17..=32 => Literal::u32_suffixed(1),
            33..=64 => Literal::u64_suffixed(1),
            65..=128 => Literal::u128_suffixed(1),
            129.. => unreachable!()
        };


        let unsigned_max_doc = format!(" assert_eq!({unsigned_ident}::MAX, {unsigned_ident}::try_from({unsigned_max_ident}).unwrap());");
        let unsigned_min_doc = format!(" assert_eq!({unsigned_ident}::MIN, {unsigned_ident}::try_from({unsigned_min_ident}).unwrap());");
        let unsigned_bits_doc = format!("assert_eq!({unsigned_ident}::BITS, {size});");
        let unsigned_wrapping_add_examples = format!(" assert_eq!({unsigned_ident}::MAX.wrapping_add({unsigned_ident}::try_from({unsigned_one}).unwrap()), {unsigned_ident}::MIN);");
        let unsigned_growing_add = if size == max {
            quote! {}
        }
        else {
            let unsigned_ident_plus_one = format_ident!("u{}",size + 1);
            // On `x86` `reg` does not support 8-bit registers, instead this is `reg_byte`.
            // See https://doc.rust-lang.org/reference/inline-assembly.html.
            // ```text
            // Architecture	Register    class	    Target feature	Allowed types
            // x86-32	                reg	        None	        i16, i32, f32
            // x86-64	                reg	        None	        i16, i32, f32, i64, f64
            // x86	                    reg_byte	None	        i8
            // ```
            let (add_inner, sub_inner) = if cfg!(target_arch="x86_64") && size < 8 {
                (
                    quote! {
                        std::arch::asm! {
                            "add {x}, {y}",
                            x = inout(reg_byte) x,
                            y = in(reg_byte) y,
                        }
                    },
                    quote! {
                        std::arch::asm! {
                            "sub {x}, {y}",
                            x = inout(reg_byte) x,
                            y = in(reg_byte) y,
                        }
                    }
                )
            }
            // Registers do not support i128s & u128s.
            else if size >= 64 {
                (
                    quote!{
                        x += y;
                    },
                    quote!{
                        x -= y;
                    }
                )
            }
            else {
                (
                    quote! {
                        std::arch::asm! {
                            "add {x}, {y}",
                            x = inout(reg) x,
                            y = in(reg) y,
                        }
                    },
                    quote! {
                        std::arch::asm! {
                            "sub {x}, {y}",
                            x = inout(reg) x,
                            y = in(reg) y,
                        }
                    }
                )
            };
            quote! {
                impl #unsigned_ident {
                    /// Growing addition. Compute `self + rhs` returning an output type guaranteed
                    /// to be able to contain the result.
                    /// 
                    /// Since `unchecked_math` is nightly only, this uses inline assembly.
                    pub fn growing_add(self, rhs: #unsigned_ident) -> #unsigned_ident_plus_one {
                        let mut x = <#unsigned_ident_plus_one>::from(self).0;
                        let y = <#unsigned_ident_plus_one>::from(rhs).0;
                        unsafe {
                            #add_inner
                        }
                        #unsigned_ident_plus_one(x)
                    }
                }
                impl #unsigned_ident_plus_one {
                    /// Growing subtraction. Compute `self - rhs` returning an output type
                    /// guaranteed to be able to contain the result.
                    /// 
                    /// Since `unchecked_math` is nightly only, this uses inline assembly.
                    pub fn growing_sub(self, rhs: #unsigned_ident) -> #unsigned_ident_plus_one {
                        let mut x = <#unsigned_ident_plus_one>::from(self).0;
                        let y = <#unsigned_ident_plus_one>::from(rhs).0;
                        unsafe {
                            #sub_inner
                        }
                        #unsigned_ident_plus_one(x)
                    }
                }
            }
        };

        let signed_doc = format!(" The {size}-bit signed integer type.");
        let signed_inner_ident = format_ident!("i{inner_size}");
        let signed_inner_ident = quote! { core::primitive::#signed_inner_ident };
        let signed_ident = format_ident!("i{size}");
        let signed_max = 2u128.pow(u32::from(size) - 1) - 1;
        let signed_max_ident = match size {
            0..=8 => Literal::i8_suffixed(signed_max as i8),
            9..=16 => Literal::i16_suffixed(signed_max as i16),
            17..=32 => Literal::i32_suffixed(signed_max as i32),
            33..=64 => Literal::i64_suffixed(signed_max as i64),
            65..=128 => Literal::i128_suffixed(signed_max as i128),
            129.. => unreachable!()
        };
        let signed_min_ident = match size {
            0..=8 => Literal::i8_suffixed(-(signed_max as i8)-1),
            9..=16 => Literal::i16_suffixed(-(signed_max as i16)-1),
            17..=32 => Literal::i32_suffixed(-(signed_max as i32)-1),
            33..=64 => Literal::i64_suffixed(-(signed_max as i64)-1),
            65..=128 => Literal::i128_suffixed(-(signed_max as i128)-1),
            129.. => unreachable!()
        };
        let signed_minus_one = match size {
            0..=8 => Literal::i8_suffixed(-1),
            9..=16 => Literal::i16_suffixed(-1),
            17..=32 => Literal::i32_suffixed(-1),
            33..=64 => Literal::i64_suffixed(-1),
            65..=128 => Literal::i128_suffixed(-1),
            129.. => unreachable!()
        };
        let signed_zero = match size {
            0..=8 => Literal::i8_suffixed(0),
            9..=16 => Literal::i16_suffixed(0),
            17..=32 => Literal::i32_suffixed(0),
            33..=64 => Literal::i64_suffixed(0),
            65..=128 => Literal::i128_suffixed(0),
            129.. => unreachable!()
        };
        let signed_one = match size {
            0..=8 => Literal::i8_suffixed(1),
            9..=16 => Literal::i16_suffixed(1),
            17..=32 => Literal::i32_suffixed(1),
            33..=64 => Literal::i64_suffixed(1),
            65..=128 => Literal::i128_suffixed(1),
            129.. => unreachable!()
        };

        let signed_max_doc = format!(" assert_eq!({signed_ident}::MAX, {signed_ident}::try_from({signed_max_ident}).unwrap());");
        let signed_min_doc = format!(" assert_eq!({signed_ident}::MIN, {signed_ident}::try_from({signed_min_ident}).unwrap());");
        let signed_bits_doc = format!(" assert_eq!({signed_ident}::BITS, {size});");
        let signed_wrapping_add_examples = format!(" assert_eq!({signed_ident}::MIN.wrapping_add({signed_ident}::try_from({signed_minus_one}).unwrap()), {signed_ident}::MAX);");
        // TODO This is not correct, correct this.
        let signed_abs_doc = format!(" The absolute value of `{signed_ident}::MIN` cannot be represented as an `{signed_ident}`, and attempting to calculate it will cause an overflow. This means that code in debug mode will trigger a panic on this case and optimized code will return `{signed_ident}::MIN` without a panic.");
        let signed_abs_doc_example_one = format!(" assert_eq!({signed_ident}::MAX.abs(), {signed_ident}::MAX);");
        let signed_abs_doc_example_two = if size == 1 {
            String::from(" // `i1` cannot contain `1`.")
        }else {
            format!(" assert_eq!(({signed_ident}::MIN + {signed_ident}::try_from(1i8).unwrap()).abs(), {signed_ident}::MAX);")
        };
        let signed_growing_ops = if size == max {
            quote! {}
        }
        else {
            let signed_ident_plus_one = format_ident!("i{}",size + 1);
            // On `x86` `reg` does not support 8-bit registers, instead this is `reg_byte`.
            // See https://doc.rust-lang.org/reference/inline-assembly.html.
            // ```text
            // Architecture	Register    class	    Target feature	Allowed types
            // x86-32	                reg	        None	        i16, i32, f32
            // x86-64	                reg	        None	        i16, i32, f32, i64, f64
            // x86	                    reg_byte	None	        i8
            // ```
            let (add_inner,sub_inner) = if cfg!(target_arch="x86_64") && size < 8 {
                (
                    quote! {
                        std::arch::asm! {
                            "add {x}, {y}",
                            x = inout(reg_byte) x,
                            y = in(reg_byte) y,
                        }
                    },
                    quote! {
                        std::arch::asm! {
                            "sub {x}, {y}",
                            x = inout(reg_byte) x,
                            y = in(reg_byte) y,
                        }
                    }
                )
            }
            // Registers do not support i128s & u128s.
            else if size >= 64 {
                (
                    quote!{
                        x += y;
                    },
                    quote!{
                        x -= y;
                    }
                )
            }
            else {
                (
                    quote! {
                        std::arch::asm! {
                            "add {x}, {y}",
                            x = inout(reg) x,
                            y = in(reg) y,
                        }
                    },
                    quote! {
                        std::arch::asm! {
                            "sub {x}, {y}",
                            x = inout(reg) x,
                            y = in(reg) y,
                        }
                    }
                )
            };
            quote! {
                /// Growing addition. Compute `self + rhs` returning an output type guaranteed to be
                /// able to contain the result.
                /// 
                /// Since `unchecked_math` is nightly only, this uses inline assembly.
                pub fn growing_add(self, rhs: #signed_ident) -> #signed_ident_plus_one {
                    let mut x = <#signed_ident_plus_one>::from(self).0;
                    let y = <#signed_ident_plus_one>::from(rhs).0;
                    unsafe {
                        #add_inner
                    }
                    #signed_ident_plus_one(x)
                }

                /// Growing subtraction. Compute `self - rhs` returning an output type guaranteed to
                /// be able to contain the result.
                /// 
                /// Since `unchecked_math` is nightly only, this uses inline assembly.
                pub fn growing_sub(self, rhs: #signed_ident) -> #signed_ident_plus_one {
                    let mut x = <#signed_ident_plus_one>::from(self).0;
                    let y = <#signed_ident_plus_one>::from(rhs).0;
                    unsafe {
                        #sub_inner
                    }
                    #signed_ident_plus_one(x)
                }
            }
        };

        let unsigned_from_implementations = {
            let primitive_from = [8,16,32,64,128].into_iter().map(|s| {
                let from_ident = format_ident!("u{s}");
                let from_ident = quote! { core::primitive::#from_ident };
                if size >= s {
                    quote! {
                        impl From<#from_ident> for #unsigned_ident {
                            fn from(x: #from_ident) -> Self {
                                Self(<#unsigned_inner_ident>::from(x))
                            }
                        }
                    }
                }
                else {
                    quote! {
                        impl TryFrom<#from_ident> for #unsigned_ident {
                            type Error = TryFromIntError;
                            fn try_from(x: #from_ident) -> Result<Self, Self::Error> {
                                let y = <#unsigned_inner_ident>::try_from(x).map_err(|_|TryFromIntError)?;
                                if (Self::MIN.0..=Self::MAX.0).contains(&y) {
                                    Ok(Self(y))
                                }
                                else {
                                    Err(TryFromIntError)
                                }
                            }
                        }
                    }
                }
            });

            let smaller = (1..size).map(|s| {
                let from_unsigned_ident = format_ident!("u{s}");
                let from_signed_ident = format_ident!("i{s}");
                quote! {
                    impl From<#from_unsigned_ident> for #unsigned_ident {
                        fn from(x: #from_unsigned_ident) -> #unsigned_ident {
                            Self(x.0 as #unsigned_inner_ident)
                        }
                    }
                    impl TryFrom<#from_signed_ident> for #unsigned_ident {
                        type Error = TryFromIntError;
                        fn try_from(x: #from_signed_ident) -> Result<#unsigned_ident,Self::Error> {
                            Ok(Self(#unsigned_inner_ident::try_from(x.0).map_err(|_|TryFromIntError)?))
                        }
                    }
                }
            });
            let same = std::iter::once({
                quote! {
                    impl TryFrom<#signed_ident> for #unsigned_ident {
                        type Error = TryFromIntError;
                        fn try_from(x: #signed_ident) -> Result<#unsigned_ident,Self::Error> {
                            Ok(Self(#unsigned_inner_ident::try_from(x.0).map_err(|_|TryFromIntError)?))
                        }
                    }
                }
            });
            let bigger = ((size+1)..=max).map(|s| {
                let from_unsigned_ident = format_ident!("u{s}");
                let from_signed_ident = format_ident!("i{s}");
                quote! {
                    impl TryFrom<#from_unsigned_ident> for #unsigned_ident {
                        type Error = TryFromIntError;
                        fn try_from(x: #from_unsigned_ident) -> Result<#unsigned_ident,Self::Error> {
                            let y = #unsigned_inner_ident::try_from(x.0).map_err(|_|TryFromIntError)?;
                            if (#unsigned_ident::MIN.0..=#unsigned_ident::MAX.0).contains(&y) {
                                Ok(Self(y))
                            }
                            else {
                                Err(TryFromIntError)
                            }
                        }
                    }
                    impl TryFrom<#from_signed_ident> for #unsigned_ident {
                        type Error = TryFromIntError;
                        fn try_from(x: #from_signed_ident) -> Result<#unsigned_ident,Self::Error> {
                            let y = #unsigned_inner_ident::try_from(x.0).map_err(|_|TryFromIntError)?;
                            if (#unsigned_ident::MIN.0..=#unsigned_ident::MAX.0).contains(&y) {
                                Ok(Self(y))
                            }
                            else {
                                Err(TryFromIntError)
                            }
                        }
                    }
                }
            });
            primitive_from.chain(smaller).chain(same).chain(bigger).collect::<TokenStream>()
        };
        let signed_from_implementations = {
            let primitive_from = [8,16,32,64,128].into_iter().map(|s| {
                let from_ident = format_ident!("i{s}");
                let from_ident = quote! { core::primitive::#from_ident };
                if size >= s {
                    quote! {
                        impl From<#from_ident> for #signed_ident {
                            fn from(x: #from_ident) -> Self {
                                Self(<#signed_inner_ident>::from(x))
                            }
                        }
                    }
                }
                else {
                    quote! {
                        impl TryFrom<#from_ident> for #signed_ident {
                            type Error = TryFromIntError;
                            fn try_from(x: #from_ident) -> Result<Self, Self::Error> {
                                let y = <#signed_inner_ident>::try_from(x).map_err(|_|TryFromIntError)?;
                                if (Self::MIN.0..=Self::MAX.0).contains(&y) {
                                    Ok(Self(y))
                                }
                                else {
                                    Err(TryFromIntError)
                                }
                            }
                        }
                    }
                }
            });

            let smaller = (1..size).map(|s| {
                let from_unsigned_ident = format_ident!("u{s}");
                let from_signed_ident = format_ident!("i{s}");
                quote! {
                    impl From<#from_unsigned_ident> for #signed_ident {
                        fn from(x: #from_unsigned_ident) -> #signed_ident {
                            Self(x.0 as #signed_inner_ident)
                        }
                    }
                    impl From<#from_signed_ident> for #signed_ident {
                        fn from(x: #from_signed_ident) -> #signed_ident {
                            Self(x.0 as #signed_inner_ident)
                        }
                    }
                }
            });

            let same = std::iter::once({
                quote! {
                    impl TryFrom<#unsigned_ident> for #signed_ident {
                        type Error = TryFromIntError;
                        fn try_from(x: #unsigned_ident) -> Result<#signed_ident,Self::Error> {
                            Ok(Self(#signed_inner_ident::try_from(x.0).map_err(|_|TryFromIntError)?))
                        }
                    }
                }
            });

            let bigger = ((size+1)..=max).map(|s| {
                let from_unsigned_ident = format_ident!("u{s}");
                let from_signed_ident = format_ident!("i{s}");
                quote! {
                    impl TryFrom<#from_unsigned_ident> for #signed_ident {
                        type Error = TryFromIntError;
                        fn try_from(x: #from_unsigned_ident) -> Result<#signed_ident,Self::Error> {
                            let y = #signed_inner_ident::try_from(x.0).map_err(|_|TryFromIntError)?;
                            if (#signed_ident::MIN.0..=#signed_ident::MAX.0).contains(&y) {
                                Ok(Self(y))
                            }
                            else {
                                Err(TryFromIntError)
                            }
                        }
                    }
                    impl TryFrom<#from_signed_ident> for #signed_ident {
                        type Error = TryFromIntError;
                        fn try_from(x: #from_signed_ident) -> Result<#signed_ident,Self::Error> {
                            let y = #signed_inner_ident::try_from(x.0).map_err(|_|TryFromIntError)?;
                            if (#signed_ident::MIN.0..=#signed_ident::MAX.0).contains(&y) {
                                Ok(Self(y))
                            }
                            else {
                                Err(TryFromIntError)
                            }
                        }
                    }
                }
            });
            primitive_from.chain(smaller).chain(same).chain(bigger).collect::<TokenStream>()
        };
        let unsigned_mask = match size {
            8 | 16 | 32 | 64 | 128 => quote! { #unsigned_min_ident },
            _ => quote! { (#unsigned_one << #size) }
        };
        let signed_mask = match size {
            8 | 16 | 32 | 64 | 128 => quote! { #signed_zero },
            _ => quote! { (#signed_one << #size) }
        };

        quote! {
            #[doc=#unsigned_doc]
            #[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
            #[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash)]
            pub struct #unsigned_ident(#unsigned_inner_ident);
            impl #unsigned_ident {
                fn mask(self) -> Self {
                    Self(self.0 & #unsigned_mask.overflowing_sub(#unsigned_one).0)
                }

                /// The smallest value that can be represented by this integer type.
                /// 
                /// # Examples
                /// 
                /// Basic usage:
                /// ```
                /// # use ux2::*;
                #[doc=#unsigned_min_doc]
                /// ```
                pub const MIN: #unsigned_ident = #unsigned_ident(#unsigned_min_ident);
                /// The largest value that can be represented by this integer type.
                /// 
                /// # Examples
                /// 
                /// Basic usage:
                /// ```
                /// # use ux2::*;
                #[doc=#unsigned_max_doc]
                /// ```
                pub const MAX: #unsigned_ident = #unsigned_ident(#unsigned_max_ident);
                /// The size of this integer type in bits.
                /// 
                /// # Examples
                /// 
                /// ```
                /// # use ux2::*;
                #[doc=#unsigned_bits_doc]
                /// ```
                pub const BITS: core::primitive::u32 = #bits;

                /// Create a native endian integer value from its representation as a byte
                /// array in big endian.
                pub fn from_be_bytes(bytes: [core::primitive::u8; #bytes]) -> Self {
                    let mut arr = [0u8; #inner_bytes];
                    unsafe {
                        arr.as_mut_ptr().copy_from_nonoverlapping(&bytes[0],#bytes);
                    }
                    Self(#unsigned_inner_ident::from_be_bytes(arr))
                }
                /// Create a native endian integer value from its representation as a byte
                /// array in little endian.
                pub fn from_le_bytes(bytes: [core::primitive::u8; #bytes]) -> Self {
                    let mut arr = [0u8; #inner_bytes];
                    unsafe {
                        arr.as_mut_ptr().copy_from_nonoverlapping(&bytes[0],#bytes);
                    }
                    Self(#unsigned_inner_ident::from_le_bytes(arr))
                }
                /// Create a native endian integer value from its memory representation as a
                /// byte array in native endianness.
                ///
                /// As the target platform’s native endianness is used, portable code likely
                /// wants to use `from_be_bytes` or `from_le_bytes`, as appropriate instead.
                pub fn from_ne_bytes(bytes: [core::primitive::u8; #bytes]) -> Self {
                    // https://doc.rust-lang.org/reference/conditional-compilation.html#target_endian
                    #[cfg(target_endian="big")]
                    let ret = Self::from_be_bytes(bytes);
                    #[cfg(target_endian="little")]
                    let ret = Self::from_le_bytes(bytes);
                    ret
                }

                /// Return the memory representation of this integer as a byte array in
                /// big-endian (network) byte order.
                pub fn to_be_bytes(self) -> [core::primitive::u8; #bytes] {
                    let mut arr = self.0.to_be_bytes();
                    array_rsplit_array_mut::<_,#inner_bytes,#bytes>(&mut arr).1.clone()
                }
                /// Return the memory representation of this integer as a byte array in
                /// little-endian byte order.
                pub fn to_le_bytes(self) -> [core::primitive::u8; #bytes] {
                    let mut arr = self.0.to_le_bytes();
                    array_split_array_mut::<_,#inner_bytes,#bytes>(&mut arr).0.clone()
                }
                /// Return the memory representation of this integer as a byte array in
                /// native byte order.
                ///
                /// As the target platform’s native endianness is used, portable code should
                /// use `to_be_bytes` or `to_le_bytes`, as appropriate, instead.
                pub fn to_ne_bytes(self) -> [core::primitive::u8; #bytes] {
                    // https://doc.rust-lang.org/reference/conditional-compilation.html#target_endian
                    #[cfg(target_endian="big")]
                    let ret = self.to_be_bytes();
                    #[cfg(target_endian="little")]
                    let ret = self.to_be_bytes();
                    ret
                }

                /// Converts a string slice in a given base to an integer.
                ///
                /// The string is expected to be an optional + sign followed by digits.
                /// Leading and trailing whitespace represent an error. Digits are a subset
                /// of these characters, depending on radix:
                /// 
                /// - `0-9`
                /// - `a-z`
                /// - `A-Z`
                /// 
                /// # Panics
                /// 
                /// This function panics if `radix` is not in the range from 2 to 36.
                pub fn from_str_radix(src: &str, radix: core::primitive::u32) -> Result<#unsigned_ident, ParseIntError> {
                    let x = #unsigned_inner_ident::from_str_radix(src,radix).map_err(|_|ParseIntError)?;
                    if (#unsigned_ident::MIN.0..#unsigned_ident::MAX.0).contains(&x) {
                        Ok(Self(x))
                    }
                    else {
                        Err(ParseIntError)
                    }
                }

                /// Raises self to the power of `exp`, using exponentiation by squaring.
                pub fn pow(self, exp: core::primitive::u32) -> #unsigned_ident {
                    let x = self.0.pow(exp);
                    debug_assert!((#unsigned_ident::MIN.0..#unsigned_ident::MAX.0).contains(&x));
                    Self(x)
                }

                /// Checked integer addition. Computes `self + rhs`, returning `None` if overflow occurred.
                pub fn checked_add(self, rhs: #unsigned_ident) -> Option<#unsigned_ident> {
                    match self.0.checked_add(rhs.0) {
                        Some(x) if (#unsigned_ident::MIN.0..#unsigned_ident::MAX.0).contains(&x) => Some(Self(x)),
                        _ => None
                    }
                }
                /// Checked integer division. Computes `self / rhs`, returning `None` if `rhs == 0`.
                pub fn checked_div(self, rhs: #unsigned_ident) -> Option<#unsigned_ident> {
                    match self.0.checked_div(rhs.0) {
                        Some(x) if (#unsigned_ident::MIN.0..#unsigned_ident::MAX.0).contains(&x) => Some(Self(x)),
                        _ => None
                    }
                }
                /// Checked integer multiplication. Computes `self * rhs`, returning `None` if overflow occurred.
                pub fn checked_mul(self, rhs: #unsigned_ident) -> Option<#unsigned_ident> {
                    match self.0.checked_mul(rhs.0) {
                        Some(x) if (#unsigned_ident::MIN.0..#unsigned_ident::MAX.0).contains(&x) => Some(Self(x)),
                        _ => None
                    }
                }
                /// Checked integer subtraction. Computes `self - rhs`, returning `None` if overflow occurred.
                pub fn checked_sub(self, rhs: #unsigned_ident) -> Option<#unsigned_ident> {
                    match self.0.checked_sub(rhs.0) {
                        Some(x) if (#unsigned_ident::MIN.0..#unsigned_ident::MAX.0).contains(&x) => Some(Self(x)),
                        _ => None
                    }
                }
                /// Wrapping (modular) addition. Computes `self + rhs`, wrapping around at
                /// the boundary of the type.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                /// # use ux2::*;
                #[doc=#unsigned_wrapping_add_examples]
                /// ```
                pub fn wrapping_add(self, rhs: #unsigned_ident) -> #unsigned_ident {
                    Self(self.0.overflowing_add(rhs.0).0).mask()
                }

                /// Wrapping (modular) subtraction. Computes `self - rhs`, wrapping around
                /// at the boundary of the type.
                pub fn wrapping_sub(self, rhs: #unsigned_ident) -> #unsigned_ident {
                    Self(self.0.overflowing_sub(rhs.0).0).mask()
                }
            }

            #unsigned_growing_add

            impl std::ops::Add<&#unsigned_ident> for &#unsigned_ident {
                type Output = #unsigned_ident;
                fn add(self, rhs: &#unsigned_ident) -> Self::Output {
                    let x = self.0 + rhs.0;
                    debug_assert!((#unsigned_ident::MIN.0..#unsigned_ident::MAX.0).contains(&x));
                    #unsigned_ident(x)
                }
            }
            impl std::ops::Add<&#unsigned_ident> for #unsigned_ident {
                type Output = #unsigned_ident;
                fn add(self, rhs: &#unsigned_ident) -> Self::Output {
                    let x = self.0 + rhs.0;
                    debug_assert!((#unsigned_ident::MIN.0..#unsigned_ident::MAX.0).contains(&x));
                    #unsigned_ident(x)
                }
            }
            impl<'a> std::ops::Add<#unsigned_ident> for &'a #unsigned_ident {
                type Output = #unsigned_ident;
                fn add(self, rhs: #unsigned_ident) -> Self::Output {
                    let x = self.0 + rhs.0;
                    debug_assert!((#unsigned_ident::MIN.0..#unsigned_ident::MAX.0).contains(&x));
                    #unsigned_ident(x)
                }
            }
            impl std::ops::Add<#unsigned_ident> for #unsigned_ident {
                type Output = #unsigned_ident;
                fn add(self, rhs: #unsigned_ident) -> Self::Output {
                    let x = self.0 + rhs.0;
                    debug_assert!((#unsigned_ident::MIN.0..#unsigned_ident::MAX.0).contains(&x));
                    #unsigned_ident(x)
                }
            }

            impl std::ops::Sub<&#unsigned_ident> for &#unsigned_ident {
                type Output = #unsigned_ident;
                fn sub(self, rhs: &#unsigned_ident) -> Self::Output {
                    let x = self.0 - rhs.0;
                    debug_assert!((#unsigned_ident::MIN.0..#unsigned_ident::MAX.0).contains(&x));
                    #unsigned_ident(x)
                }
            }
            impl std::ops::Sub<&#unsigned_ident> for #unsigned_ident {
                type Output = #unsigned_ident;
                fn sub(self, rhs: &#unsigned_ident) -> Self::Output {
                    let x = self.0 - rhs.0;
                    debug_assert!((#unsigned_ident::MIN.0..#unsigned_ident::MAX.0).contains(&x));
                    #unsigned_ident(x)
                }
            }
            impl<'a> std::ops::Sub<#unsigned_ident> for &'a #unsigned_ident {
                type Output = #unsigned_ident;
                fn sub(self, rhs: #unsigned_ident) -> Self::Output {
                    let x = self.0 - rhs.0;
                    debug_assert!((#unsigned_ident::MIN.0..#unsigned_ident::MAX.0).contains(&x));
                    #unsigned_ident(x)
                }
            }
            impl std::ops::Sub<#unsigned_ident> for #unsigned_ident {
                type Output = #unsigned_ident;
                fn sub(self, rhs: #unsigned_ident) -> Self::Output {
                    let x = self.0 - rhs.0;
                    debug_assert!((#unsigned_ident::MIN.0..#unsigned_ident::MAX.0).contains(&x));
                    #unsigned_ident(x)
                }
            }

            impl std::ops::Mul<&#unsigned_ident> for &#unsigned_ident {
                type Output = #unsigned_ident;
                fn mul(self, rhs: &#unsigned_ident) -> Self::Output {
                    let x = self.0 * rhs.0;
                    debug_assert!((#unsigned_ident::MIN.0..#unsigned_ident::MAX.0).contains(&x));
                    #unsigned_ident(x)
                }
            }
            impl std::ops::Mul<&#unsigned_ident> for #unsigned_ident {
                type Output = #unsigned_ident;
                fn mul(self, rhs: &#unsigned_ident) -> Self::Output {
                    let x = self.0 * rhs.0;
                    debug_assert!((#unsigned_ident::MIN.0..#unsigned_ident::MAX.0).contains(&x));
                    #unsigned_ident(x)
                }
            }
            impl<'a> std::ops::Mul<#unsigned_ident> for &'a #unsigned_ident {
                type Output = #unsigned_ident;
                fn mul(self, rhs: #unsigned_ident) -> Self::Output {
                    let x = self.0 * rhs.0;
                    debug_assert!((#unsigned_ident::MIN.0..#unsigned_ident::MAX.0).contains(&x));
                    #unsigned_ident(x)
                }
            }
            impl std::ops::Mul<#unsigned_ident> for #unsigned_ident {
                type Output = #unsigned_ident;
                fn mul(self, rhs: #unsigned_ident) -> Self::Output {
                    let x = self.0 * rhs.0;
                    debug_assert!((#unsigned_ident::MIN.0..#unsigned_ident::MAX.0).contains(&x));
                    #unsigned_ident(x)
                }
            }

            impl std::ops::Div<&#unsigned_ident> for &#unsigned_ident {
                type Output = #unsigned_ident;
                fn div(self, rhs: &#unsigned_ident) -> Self::Output {
                    let x = self.0 / rhs.0;
                    debug_assert!((#unsigned_ident::MIN.0..#unsigned_ident::MAX.0).contains(&x));
                    #unsigned_ident(x)
                }
            }
            impl std::ops::Div<&#unsigned_ident> for #unsigned_ident {
                type Output = #unsigned_ident;
                fn div(self, rhs: &#unsigned_ident) -> Self::Output {
                    let x = self.0 / rhs.0;
                    debug_assert!((#unsigned_ident::MIN.0..#unsigned_ident::MAX.0).contains(&x));
                    #unsigned_ident(x)
                }
            }
            impl<'a> std::ops::Div<#unsigned_ident> for &'a #unsigned_ident {
                type Output = #unsigned_ident;
                fn div(self, rhs: #unsigned_ident) -> Self::Output {
                    let x = self.0 / rhs.0;
                    debug_assert!((#unsigned_ident::MIN.0..#unsigned_ident::MAX.0).contains(&x));
                    #unsigned_ident(x)
                }
            }
            impl std::ops::Div<#unsigned_ident> for #unsigned_ident {
                type Output = #unsigned_ident;
                fn div(self, rhs: #unsigned_ident) -> Self::Output {
                    let x = self.0 / rhs.0;
                    debug_assert!((#unsigned_ident::MIN.0..#unsigned_ident::MAX.0).contains(&x));
                    #unsigned_ident(x)
                }
            }

            impl std::ops::Rem<&#unsigned_ident> for &#unsigned_ident {
                type Output = #unsigned_ident;
                fn rem(self, rhs: &#unsigned_ident) -> Self::Output {
                    let x = self.0 % rhs.0;
                    debug_assert!((#unsigned_ident::MIN.0..#unsigned_ident::MAX.0).contains(&x));
                    #unsigned_ident(x)
                }
            }
            impl std::ops::Rem<&#unsigned_ident> for #unsigned_ident {
                type Output = #unsigned_ident;
                fn rem(self, rhs: &#unsigned_ident) -> Self::Output {
                    let x = self.0 / rhs.0;
                    debug_assert!((#unsigned_ident::MIN.0..#unsigned_ident::MAX.0).contains(&x));
                    #unsigned_ident(x)
                }
            }
            impl<'a> std::ops::Rem<#unsigned_ident> for &'a #unsigned_ident {
                type Output = #unsigned_ident;
                fn rem(self, rhs: #unsigned_ident) -> Self::Output {
                    let x = self.0 % rhs.0;
                    debug_assert!((#unsigned_ident::MIN.0..#unsigned_ident::MAX.0).contains(&x));
                    #unsigned_ident(x)
                }
            }
            impl std::ops::Rem<#unsigned_ident> for #unsigned_ident {
                type Output = #unsigned_ident;
                fn rem(self, rhs: #unsigned_ident) -> Self::Output {
                    let x = self.0 % rhs.0;
                    debug_assert!((#unsigned_ident::MIN.0..#unsigned_ident::MAX.0).contains(&x));
                    #unsigned_ident(x)
                }
            }

            impl std::fmt::Binary for #unsigned_ident {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    std::fmt::Binary::fmt(&self.0, f)
                }
            }
            impl std::fmt::LowerHex for #unsigned_ident {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    std::fmt::LowerHex::fmt(&self.0, f)
                }
            }
            impl std::fmt::UpperHex for #unsigned_ident {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    std::fmt::UpperHex::fmt(&self.0, f)
                }
            }
            impl std::fmt::Octal for #unsigned_ident {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    std::fmt::Octal::fmt(&self.0, f)
                }
            }

            impl std::ops::BitAnd<&#unsigned_ident> for &#unsigned_ident {
                type Output = <#unsigned_ident as std::ops::BitAnd<#unsigned_ident>>::Output;

                fn bitand(self, rhs: &#unsigned_ident) -> Self::Output {
                    #unsigned_ident(self.0 & rhs.0)
                }
            }
            impl std::ops::BitAnd<&#unsigned_ident> for #unsigned_ident {
                type Output = <#unsigned_ident as std::ops::BitAnd<#unsigned_ident>>::Output;

                fn bitand(self, rhs: &#unsigned_ident) -> Self::Output {
                    #unsigned_ident(self.0 & rhs.0)
                }
            }
            impl<'a> std::ops::BitAnd<#unsigned_ident> for &'a #unsigned_ident {
                type Output = <#unsigned_ident as std::ops::BitAnd<#unsigned_ident>>::Output;

                fn bitand(self, rhs: #unsigned_ident) -> Self::Output {
                    #unsigned_ident(self.0 & rhs.0)
                }
            }
            impl std::ops::BitAnd<#unsigned_ident> for #unsigned_ident {
                type Output = Self;

                fn bitand(self, rhs: Self) -> Self::Output {
                    #unsigned_ident(self.0 & rhs.0)
                }
            }

            impl std::ops::BitOr<&#unsigned_ident> for &#unsigned_ident {
                type Output = <#unsigned_ident as std::ops::BitOr<#unsigned_ident>>::Output;

                fn bitor(self, rhs: &#unsigned_ident) -> Self::Output {
                    #unsigned_ident(self.0 | rhs.0)
                }
            }
            impl std::ops::BitOr<&#unsigned_ident> for #unsigned_ident {
                type Output = <#unsigned_ident as std::ops::BitOr<#unsigned_ident>>::Output;

                fn bitor(self, rhs: &#unsigned_ident) -> Self::Output {
                    #unsigned_ident(self.0 | rhs.0)
                }
            }
            impl<'a> std::ops::BitOr<#unsigned_ident> for &'a #unsigned_ident {
                type Output = <#unsigned_ident as std::ops::BitOr<#unsigned_ident>>::Output;

                fn bitor(self, rhs: #unsigned_ident) -> Self::Output {
                    #unsigned_ident(self.0 | rhs.0)
                }
            }
            impl std::ops::BitOr<#unsigned_ident> for #unsigned_ident {
                type Output = Self;

                fn bitor(self, rhs: Self) -> Self::Output {
                    #unsigned_ident(self.0 | rhs.0)
                }
            }

            impl std::ops::BitXor<&#unsigned_ident> for &#unsigned_ident {
                type Output = <#unsigned_ident as std::ops::BitXor<#unsigned_ident>>::Output;

                fn bitxor(self, rhs: &#unsigned_ident) -> Self::Output {
                    #unsigned_ident(self.0 ^ rhs.0)
                }
            }
            impl std::ops::BitXor<&#unsigned_ident> for #unsigned_ident {
                type Output = <#unsigned_ident as std::ops::BitXor<#unsigned_ident>>::Output;

                fn bitxor(self, rhs: &#unsigned_ident) -> Self::Output {
                    #unsigned_ident(self.0 ^ rhs.0)
                }
            }
            impl<'a> std::ops::BitXor<#unsigned_ident> for &'a #unsigned_ident {
                type Output = <#unsigned_ident as std::ops::BitXor<#unsigned_ident>>::Output;

                fn bitxor(self, rhs: #unsigned_ident) -> Self::Output {
                    #unsigned_ident(self.0 ^ rhs.0)
                }
            }
            impl std::ops::BitXor<#unsigned_ident> for #unsigned_ident {
                type Output = Self;

                fn bitxor(self, rhs: Self) -> Self::Output {
                    #unsigned_ident(self.0 ^ rhs.0)
                }
            }

            impl std::ops::Not for #unsigned_ident {
                type Output = Self;

                fn not(self) -> Self::Output {
                    #unsigned_ident(self.0)
                }
            }
            impl std::ops::Not for &#unsigned_ident {
                type Output = <#unsigned_ident as std::ops::Not>::Output;

                fn not(self) -> Self::Output {
                    #unsigned_ident(self.0)
                }
            }

            #unsigned_from_implementations

            #[cfg(feature="num-traits")]
            impl num_traits::identities::One for #unsigned_ident {
                fn one() -> Self {
                    use num_traits::identities::One;
                    Self(#unsigned_inner_ident::one())
                }
            }
            #[cfg(feature="num-traits")]
            impl num_traits::identities::Zero for #unsigned_ident {
                fn zero() -> Self {
                    use num_traits::identities::Zero;
                    Self(#unsigned_inner_ident::zero())
                }
                fn is_zero(&self) -> bool {
                    *self == Self::zero()
                }
            }
            #[cfg(feature="num-traits")]
            impl num_traits::Num for #unsigned_ident {
                type FromStrRadixErr = ParseIntError;

                fn from_str_radix(
                    str: &str,
                    radix: core::primitive::u32,
                ) -> Result<Self, Self::FromStrRadixErr> {
                    Self::from_str_radix(str,radix)
                }
            }

            #[doc=#signed_doc]
            #[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
            #[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash)]
            pub struct #signed_ident(#signed_inner_ident);
            impl #signed_ident {
                fn mask(self) -> Self {
                    let x = if self.0 & (#signed_one << (#size - 1)) == 0 {
                        self.0 & #signed_mask.overflowing_sub(#signed_one).0
                    }
                    else {
                        self.0 | !#signed_mask.overflowing_sub(#signed_one).0
                    };
                    Self(x)
                }

                /// The smallest value that can be represented by this integer type.
                /// 
                /// # Examples
                /// 
                /// Basic usage:
                /// ```
                /// # use ux2::*;
                #[doc=#signed_min_doc]
                /// ```
                pub const MIN: #signed_ident = #signed_ident(#signed_min_ident);
                /// The largest value that can be represented by this integer type.
                /// 
                /// # Examples
                /// 
                /// Basic usage:
                /// ```
                /// # use ux2::*;
                #[doc=#signed_max_doc]
                /// ```
                pub const MAX: #signed_ident = #signed_ident(#signed_max_ident);
                /// The size of this integer type in bits.
                /// 
                /// # Examples
                /// 
                /// ```
                /// # use ux2::*;
                #[doc=#signed_bits_doc]
                /// ```
                pub const BITS: core::primitive::u32 = #bits;
                /// Computes the absolute value of self.
                /// 
                /// # Overflow behavior
                /// 
                #[doc=#signed_abs_doc]
                ///
                /// # Examples
                /// 
                /// Basic usage:
                /// ```
                /// # use ux2::*;
                #[doc=#signed_abs_doc_example_one]
                #[doc=#signed_abs_doc_example_two]
                /// ```
                pub const fn abs(self) -> #signed_ident {
                    Self(self.0.abs())
                }

                /// Create a native endian integer value from its representation as a byte
                /// array in big endian.
                pub fn from_be_bytes(bytes: [core::primitive::u8; #bytes]) -> Self {
                    let mut arr = [0u8; #inner_bytes];
                    unsafe {
                        arr.as_mut_ptr().copy_from_nonoverlapping(&bytes[0],#bytes);
                    }
                    Self(#signed_inner_ident::from_be_bytes(arr))
                }
                /// Create a native endian integer value from its representation as a byte
                /// array in little endian.
                pub fn from_le_bytes(bytes: [core::primitive::u8; #bytes]) -> Self {
                    let mut arr = [0u8; #inner_bytes];
                    unsafe {
                        arr.as_mut_ptr().copy_from_nonoverlapping(&bytes[0],#bytes);
                    }
                    Self(#signed_inner_ident::from_le_bytes(arr))
                }
                /// Create a native endian integer value from its memory representation as a
                /// byte array in native endianness.
                ///
                /// As the target platform’s native endianness is used, portable code likely
                /// wants to use `from_be_bytes` or `from_le_bytes`, as appropriate instead.
                pub fn from_ne_bytes(bytes: [core::primitive::u8; #bytes]) -> Self {
                    // https://doc.rust-lang.org/reference/conditional-compilation.html#target_endian
                    #[cfg(target_endian="big")]
                    let ret = Self::from_be_bytes(bytes);
                    #[cfg(target_endian="little")]
                    let ret = Self::from_le_bytes(bytes);
                    ret
                }

                /// Return the memory representation of this integer as a byte array in
                /// big-endian (network) byte order.
                /// 
                /// Unused bits are undefined.
                pub fn to_be_bytes(self) -> [core::primitive::u8; #bytes] {
                    let mut arr = self.0.to_be_bytes();
                    array_rsplit_array_mut::<_,#inner_bytes,#bytes>(&mut arr).1.clone()
                }
                /// Return the memory representation of this integer as a byte array in
                /// little-endian byte order.
                /// 
                /// Unused bits are undefined.
                pub fn to_le_bytes(self) -> [core::primitive::u8; #bytes] {
                    let mut arr = self.0.to_le_bytes();
                    array_split_array_mut::<_,#inner_bytes,#bytes>(&mut arr).0.clone()
                }
                /// Return the memory representation of this integer as a byte array in
                /// native byte order.
                ///
                /// As the target platform’s native endianness is used, portable code should
                /// use `to_be_bytes` or `to_le_bytes`, as appropriate, instead.
                /// 
                /// Unused bits are undefined.
                pub fn to_ne_bytes(self) -> [core::primitive::u8; #bytes] {
                    // https://doc.rust-lang.org/reference/conditional-compilation.html#target_endian
                    #[cfg(target_endian="big")]
                    let ret = self.to_be_bytes();
                    #[cfg(target_endian="little")]
                    let ret = self.to_be_bytes();
                    ret
                }

                /// Converts a string slice in a given base to an integer.
                ///
                /// The string is expected to be an optional + sign followed by digits.
                /// Leading and trailing whitespace represent an error. Digits are a subset
                /// of these characters, depending on radix:
                /// 
                /// - `0-9`
                /// - `a-z`
                /// - `A-Z`
                /// 
                /// # Panics
                /// 
                /// This function panics if `radix` is not in the range from 2 to 36.
                pub fn from_str_radix(src: &str, radix: core::primitive::u32) -> Result<#signed_ident, ParseIntError> {
                    let x = #signed_inner_ident::from_str_radix(src,radix).map_err(|_|ParseIntError)?;
                    if (#signed_ident::MIN.0..#signed_ident::MAX.0).contains(&x) {
                        Ok(Self(x))
                    }
                    else {
                        Err(ParseIntError)
                    }
                }

                /// Raises self to the power of `exp`, using exponentiation by squaring.
                pub fn pow(self, exp: core::primitive::u32) -> #signed_ident {
                    let x = self.0.pow(exp);
                    debug_assert!((#signed_ident::MIN.0..#signed_ident::MAX.0).contains(&x));
                    Self(x)
                }

                /// Checked integer addition. Computes `self + rhs`, returning `None` if overflow occurred.
                pub fn checked_add(self, rhs: #signed_ident) -> Option<#signed_ident> {
                    match self.0.checked_add(rhs.0) {
                        Some(x) if (#signed_ident::MIN.0..#signed_ident::MAX.0).contains(&x) => Some(Self(x)),
                        _ => None
                    }
                }
                /// Checked integer division. Computes `self / rhs`, returning `None` if `rhs == 0`.
                pub fn checked_div(self, rhs: #signed_ident) -> Option<#signed_ident> {
                    match self.0.checked_div(rhs.0) {
                        Some(x) if (#signed_ident::MIN.0..#signed_ident::MAX.0).contains(&x) => Some(Self(x)),
                        _ => None
                    }
                }
                /// Checked integer multiplication. Computes `self * rhs`, returning `None` if overflow occurred.
                pub fn checked_mul(self, rhs: #signed_ident) -> Option<#signed_ident> {
                    match self.0.checked_mul(rhs.0) {
                        Some(x) if (#signed_ident::MIN.0..#signed_ident::MAX.0).contains(&x) => Some(Self(x)),
                        _ => None
                    }
                }
                /// Checked integer subtraction. Computes `self - rhs`, returning `None` if overflow occurred.
                pub fn checked_sub(self, rhs: #signed_ident) -> Option<#signed_ident> {
                    match self.0.checked_sub(rhs.0) {
                        Some(x) if (#signed_ident::MIN.0..#signed_ident::MAX.0).contains(&x) => Some(Self(x)),
                        _ => None
                    }
                }

                /// Wrapping (modular) addition. Computes `self + rhs`, wrapping around at
                /// the boundary of the type.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                /// # use ux2::*;
                #[doc=#signed_wrapping_add_examples]
                /// ```
                pub fn wrapping_add(self, rhs: #signed_ident) -> #signed_ident {
                    Self(self.0.overflowing_add(rhs.0).0).mask()
                }

                /// Wrapping (modular) subtraction. Computes `self - rhs`, wrapping around
                /// at the boundary of the type.
                pub fn wrapping_sub(self, rhs: #signed_ident) -> #signed_ident {
                    Self(self.0.overflowing_sub(rhs.0).0).mask()
                }

                #signed_growing_ops
            }

            impl std::ops::Add<&#signed_ident> for &#signed_ident {
                type Output = #signed_ident;
                fn add(self, rhs: &#signed_ident) -> Self::Output {
                    let x = self.0 + rhs.0;
                    debug_assert!((#signed_ident::MIN.0..#signed_ident::MAX.0).contains(&x));
                    #signed_ident(x)
                }
            }
            impl std::ops::Add<&#signed_ident> for #signed_ident {
                type Output = #signed_ident;
                fn add(self, rhs: &#signed_ident) -> Self::Output {
                    let x = self.0 + rhs.0;
                    debug_assert!((#signed_ident::MIN.0..#signed_ident::MAX.0).contains(&x));
                    #signed_ident(x)
                }
            }
            impl<'a> std::ops::Add<#signed_ident> for &'a #signed_ident {
                type Output = #signed_ident;
                fn add(self, rhs: #signed_ident) -> Self::Output {
                    let x = self.0 + rhs.0;
                    debug_assert!((#signed_ident::MIN.0..#signed_ident::MAX.0).contains(&x));
                    #signed_ident(x)
                }
            }
            impl std::ops::Add<#signed_ident> for #signed_ident {
                type Output = #signed_ident;
                fn add(self, rhs: #signed_ident) -> Self::Output {
                    let x = self.0 + rhs.0;
                    debug_assert!((#signed_ident::MIN.0..#signed_ident::MAX.0).contains(&x));
                    #signed_ident(x)
                }
            }

            impl std::ops::Sub<&#signed_ident> for &#signed_ident {
                type Output = #signed_ident;
                fn sub(self, rhs: &#signed_ident) -> Self::Output {
                    let x = self.0 - rhs.0;
                    debug_assert!((#signed_ident::MIN.0..#signed_ident::MAX.0).contains(&x));
                    #signed_ident(x)
                }
            }
            impl std::ops::Sub<&#signed_ident> for #signed_ident {
                type Output = #signed_ident;
                fn sub(self, rhs: &#signed_ident) -> Self::Output {
                    let x = self.0 - rhs.0;
                    debug_assert!((#signed_ident::MIN.0..#signed_ident::MAX.0).contains(&x));
                    #signed_ident(x)
                }
            }
            impl<'a> std::ops::Sub<#signed_ident> for &'a #signed_ident {
                type Output = #signed_ident;
                fn sub(self, rhs: #signed_ident) -> Self::Output {
                    let x = self.0 - rhs.0;
                    debug_assert!((#signed_ident::MIN.0..#signed_ident::MAX.0).contains(&x));
                    #signed_ident(x)
                }
            }
            impl std::ops::Sub<#signed_ident> for #signed_ident {
                type Output = #signed_ident;
                fn sub(self, rhs: #signed_ident) -> Self::Output {
                    let x = self.0 - rhs.0;
                    debug_assert!((#signed_ident::MIN.0..#signed_ident::MAX.0).contains(&x));
                    #signed_ident(x)
                }
            }

            impl std::ops::Mul<&#signed_ident> for &#signed_ident {
                type Output = #signed_ident;
                fn mul(self, rhs: &#signed_ident) -> Self::Output {
                    let x = self.0 * rhs.0;
                    debug_assert!((#signed_ident::MIN.0..#signed_ident::MAX.0).contains(&x));
                    #signed_ident(x)
                }
            }
            impl std::ops::Mul<&#signed_ident> for #signed_ident {
                type Output = #signed_ident;
                fn mul(self, rhs: &#signed_ident) -> Self::Output {
                    let x = self.0 * rhs.0;
                    debug_assert!((#signed_ident::MIN.0..#signed_ident::MAX.0).contains(&x));
                    #signed_ident(x)
                }
            }
            impl<'a> std::ops::Mul<#signed_ident> for &'a #signed_ident {
                type Output = #signed_ident;
                fn mul(self, rhs: #signed_ident) -> Self::Output {
                    let x = self.0 * rhs.0;
                    debug_assert!((#signed_ident::MIN.0..#signed_ident::MAX.0).contains(&x));
                    #signed_ident(x)
                }
            }
            impl std::ops::Mul<#signed_ident> for #signed_ident {
                type Output = #signed_ident;
                fn mul(self, rhs: #signed_ident) -> Self::Output {
                    let x = self.0 * rhs.0;
                    debug_assert!((#signed_ident::MIN.0..#signed_ident::MAX.0).contains(&x));
                    #signed_ident(x)
                }
            }

            impl std::ops::Div<&#signed_ident> for &#signed_ident {
                type Output = #signed_ident;
                fn div(self, rhs: &#signed_ident) -> Self::Output {
                    let x = self.0 / rhs.0;
                    debug_assert!((#signed_ident::MIN.0..#signed_ident::MAX.0).contains(&x));
                    #signed_ident(x)
                }
            }
            impl std::ops::Div<&#signed_ident> for #signed_ident {
                type Output = #signed_ident;
                fn div(self, rhs: &#signed_ident) -> Self::Output {
                    let x = self.0 / rhs.0;
                    debug_assert!((#signed_ident::MIN.0..#signed_ident::MAX.0).contains(&x));
                    #signed_ident(x)
                }
            }
            impl<'a> std::ops::Div<#signed_ident> for &'a #signed_ident {
                type Output = #signed_ident;
                fn div(self, rhs: #signed_ident) -> Self::Output {
                    let x = self.0 / rhs.0;
                    debug_assert!((#signed_ident::MIN.0..#signed_ident::MAX.0).contains(&x));
                    #signed_ident(x)
                }
            }
            impl std::ops::Div<#signed_ident> for #signed_ident {
                type Output = #signed_ident;
                fn div(self, rhs: #signed_ident) -> Self::Output {
                    let x = self.0 / rhs.0;
                    debug_assert!((#signed_ident::MIN.0..#signed_ident::MAX.0).contains(&x));
                    #signed_ident(x)
                }
            }

            impl std::ops::Rem<&#signed_ident> for &#signed_ident {
                type Output = #signed_ident;
                fn rem(self, rhs: &#signed_ident) -> Self::Output {
                    let x = self.0 % rhs.0;
                    debug_assert!((#signed_ident::MIN.0..#signed_ident::MAX.0).contains(&x));
                    #signed_ident(x)
                }
            }
            impl std::ops::Rem<&#signed_ident> for #signed_ident {
                type Output = #signed_ident;
                fn rem(self, rhs: &#signed_ident) -> Self::Output {
                    let x = self.0 % rhs.0;
                    debug_assert!((#signed_ident::MIN.0..#signed_ident::MAX.0).contains(&x));
                    #signed_ident(x)
                }
            }
            impl<'a> std::ops::Rem<#signed_ident> for &'a #signed_ident {
                type Output = #signed_ident;
                fn rem(self, rhs: #signed_ident) -> Self::Output {
                    let x = self.0 % rhs.0;
                    debug_assert!((#signed_ident::MIN.0..#signed_ident::MAX.0).contains(&x));
                    #signed_ident(x)
                }
            }
            impl std::ops::Rem<#signed_ident> for #signed_ident {
                type Output = #signed_ident;
                fn rem(self, rhs: #signed_ident) -> Self::Output {
                    let x = self.0 % rhs.0;
                    debug_assert!((#signed_ident::MIN.0..#signed_ident::MAX.0).contains(&x));
                    #signed_ident(x)
                }
            }

            impl std::fmt::Binary for #signed_ident {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    std::fmt::Binary::fmt(&self.0, f)
                }
            }
            impl std::fmt::LowerHex for #signed_ident {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    std::fmt::LowerHex::fmt(&self.0, f)
                }
            }
            impl std::fmt::UpperHex for #signed_ident {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    std::fmt::UpperHex::fmt(&self.0, f)
                }
            }
            impl std::fmt::Octal for #signed_ident {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    std::fmt::Octal::fmt(&self.0, f)
                }
            }

            impl std::ops::BitAnd<&#signed_ident> for &#signed_ident {
                type Output = <#signed_ident as std::ops::BitAnd<#signed_ident>>::Output;

                fn bitand(self, rhs: &#signed_ident) -> Self::Output {
                    #signed_ident(self.0 & rhs.0)
                }
            }
            impl std::ops::BitAnd<&#signed_ident> for #signed_ident {
                type Output = <#signed_ident as std::ops::BitAnd<#signed_ident>>::Output;

                fn bitand(self, rhs: &#signed_ident) -> Self::Output {
                    #signed_ident(self.0 & rhs.0)
                }
            }
            impl<'a> std::ops::BitAnd<#signed_ident> for &'a #signed_ident {
                type Output = <#signed_ident as std::ops::BitAnd<#signed_ident>>::Output;

                fn bitand(self, rhs: #signed_ident) -> Self::Output {
                    #signed_ident(self.0 & rhs.0)
                }
            }
            impl std::ops::BitAnd<#signed_ident> for #signed_ident {
                type Output = Self;

                fn bitand(self, rhs: Self) -> Self::Output {
                    #signed_ident(self.0 & rhs.0)
                }
            }

            impl std::ops::BitOr<&#signed_ident> for &#signed_ident {
                type Output = <#signed_ident as std::ops::BitOr<#signed_ident>>::Output;

                fn bitor(self, rhs: &#signed_ident) -> Self::Output {
                    #signed_ident(self.0 | rhs.0)
                }
            }
            impl std::ops::BitOr<&#signed_ident> for #signed_ident {
                type Output = <#signed_ident as std::ops::BitOr<#signed_ident>>::Output;

                fn bitor(self, rhs: &#signed_ident) -> Self::Output {
                    #signed_ident(self.0 | rhs.0)
                }
            }
            impl<'a> std::ops::BitOr<#signed_ident> for &'a #signed_ident {
                type Output = <#signed_ident as std::ops::BitOr<#signed_ident>>::Output;

                fn bitor(self, rhs: #signed_ident) -> Self::Output {
                    #signed_ident(self.0 | rhs.0)
                }
            }
            impl std::ops::BitOr<#signed_ident> for #signed_ident {
                type Output = Self;

                fn bitor(self, rhs: Self) -> Self::Output {
                    #signed_ident(self.0 | rhs.0)
                }
            }

            impl std::ops::BitXor<&#signed_ident> for &#signed_ident {
                type Output = <#signed_ident as std::ops::BitXor<#signed_ident>>::Output;

                fn bitxor(self, rhs: &#signed_ident) -> Self::Output {
                    #signed_ident(self.0 ^ rhs.0)
                }
            }
            impl std::ops::BitXor<&#signed_ident> for #signed_ident {
                type Output = <#signed_ident as std::ops::BitXor<#signed_ident>>::Output;

                fn bitxor(self, rhs: &#signed_ident) -> Self::Output {
                    #signed_ident(self.0 ^ rhs.0)
                }
            }
            impl<'a> std::ops::BitXor<#signed_ident> for &'a #signed_ident {
                type Output = <#signed_ident as std::ops::BitXor<#signed_ident>>::Output;

                fn bitxor(self, rhs: #signed_ident) -> Self::Output {
                    #signed_ident(self.0 ^ rhs.0)
                }
            }
            impl std::ops::BitXor<#signed_ident> for #signed_ident {
                type Output = Self;

                fn bitxor(self, rhs: Self) -> Self::Output {
                    #signed_ident(self.0 ^ rhs.0)
                }
            }

            impl std::ops::Neg for #signed_ident {
                type Output = Self;
                fn neg(self) -> Self::Output {
                    assert_ne!(self,Self::MIN);
                    #signed_ident(-self.0)
                }
            }

            impl std::ops::Not for #signed_ident {
                type Output = Self;

                fn not(self) -> Self::Output {
                    #signed_ident(self.0)
                }
            }
            impl std::ops::Not for &#signed_ident {
                type Output = <#signed_ident as std::ops::Not>::Output;

                fn not(self) -> Self::Output {
                    #signed_ident(self.0)
                }
            }

            #signed_from_implementations

            #[cfg(feature="num-traits")]
            impl num_traits::identities::One for #signed_ident {
                fn one() -> Self {
                    use num_traits::identities::One;
                    Self(#signed_inner_ident::one())
                }
            }
            #[cfg(feature="num-traits")]
            impl num_traits::identities::Zero for #signed_ident {
                fn zero() -> Self {
                    use num_traits::identities::Zero;
                    Self(#signed_inner_ident::zero())
                }
                fn is_zero(&self) -> bool {
                    *self == Self::zero()
                }
            }
            #[cfg(feature="num-traits")]
            impl num_traits::Num for #signed_ident {
                type FromStrRadixErr = ParseIntError;

                fn from_str_radix(
                    str: &str,
                    radix: core::primitive::u32,
                ) -> Result<Self, Self::FromStrRadixErr> {
                    Self::from_str_radix(str,radix)
                }
            }
        }
    }).collect::<proc_macro2::TokenStream>();

    output.into()
}
