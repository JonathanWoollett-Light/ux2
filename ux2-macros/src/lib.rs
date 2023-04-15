use std::str::FromStr;

use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote};

#[proc_macro]
pub fn generate_types(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let max = match item.into_iter().next() {
        Some(proc_macro::TokenTree::Literal(literal)) => {
            u8::from_str(&literal.to_string()).unwrap()
        }
        _ => panic!("bad"),
    };

    let output = (1..=max).map(|size| {
        match size {
            8 => quote! {
                pub use i8;
                pub use u8;
            },
            16 => quote! {
                pub use i16;
                pub use u16;
            },
            32 => quote! {
                pub use i32;
                pub use u32;
            },
            64 => quote! {
                pub use i64;
                pub use u64;
            },
            128 => quote! {
                pub use i128;
                pub use u128;
            },
            _ => {
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
                let unsigned_ident = format_ident!("u{size}");
                let unsigned_max = 2u128.pow(u32::from(size)) - 1;
                let unsigned_max_ident = match size {
                    0..=8 => Literal::u8_suffixed(unsigned_max as u8),
                    9..=16 => Literal::u16_suffixed(unsigned_max as u16),
                    17..=32 => Literal::u32_suffixed(unsigned_max as u32),
                    33..=64 => Literal::u64_suffixed(unsigned_max as u64),
                    65..=128 => Literal::u128_suffixed(unsigned_max),
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
                let unsigned_max_doc = format!(" assert_eq!({unsigned_ident}::MAX, {unsigned_ident}::try_from({unsigned_max_ident}).unwrap());");
                let unsigned_min_doc = format!(" assert_eq!({unsigned_ident}::MIN, {unsigned_ident}::try_from({unsigned_min_ident}).unwrap());");
                let unsigned_bits_doc = format!("assert_eq!({unsigned_ident}::BITS, {size});");

                let signed_doc = format!(" The {size}-bit signed integer type.");
                let signed_inner_ident = format_ident!("i{inner_size}");
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
                let signed_max_doc = format!(" assert_eq!({signed_ident}::MAX, {signed_ident}::try_from({signed_max_ident}).unwrap());");
                let signed_min_doc = format!(" assert_eq!({signed_ident}::MIN, {signed_ident}::try_from({signed_min_ident}).unwrap());");
                let signed_bits_doc = format!(" assert_eq!({signed_ident}::BITS, {size});");
                // TODO This is not correct, correct this.
                let signed_abs_doc = format!(" The absolute value of `{signed_ident}::MIN` cannot be represented as an `{signed_ident}`, and attempting to calculate it will cause an overflow. This means that code in debug mode will trigger a panic on this case and optimized code will return `{signed_ident}::MIN` without a panic.");
                let signed_abs_doc_example_one = format!(" assert_eq!({signed_ident}::MAX.abs(), {signed_ident}::MAX);");
                let signed_abs_doc_example_two = if size == 1 {
                    String::from(" // `i1` cannot contain `1`.")
                }else {
                    format!(" assert_eq!(({signed_ident}::MIN + {signed_ident}::try_from(1i8).unwrap()).abs(), {signed_ident}::MAX);")
                };

                let unsigned_from_implementations = {
                    let smaller = (1..size).map(|s| {
                        let from_unsigned_ident = format_ident!("u{s}");
                        let from_signed_ident = format_ident!("i{s}");
                        if s == 8 || s == 16 || s == 32 || s == 64 || s == 128 {
                            quote! {
                                impl From<#from_unsigned_ident> for #unsigned_ident {
                                    fn from(x: #from_unsigned_ident) -> #unsigned_ident {
                                        Self(x as #unsigned_inner_ident)
                                    }
                                }
                                impl TryFrom<#from_signed_ident> for #unsigned_ident {
                                    type Error = TryFromIntError;
                                    fn try_from(x: #from_signed_ident) -> Result<#unsigned_ident,Self::Error> {
                                        Ok(Self(#unsigned_inner_ident::try_from(x).map_err(|_|TryFromIntError)?))
                                    }
                                }
                            }
                        }
                        else {
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
                        if s == 8 || s == 16 || s == 32 || s == 64 || s == 128 {
                            quote! {
                                impl TryFrom<#from_unsigned_ident> for #unsigned_ident {
                                    type Error = TryFromIntError;
                                    fn try_from(x: #from_unsigned_ident) -> Result<#unsigned_ident,Self::Error> {
                                        let y = #unsigned_inner_ident::try_from(x).map_err(|_|TryFromIntError)?;
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
                                        let y = #unsigned_inner_ident::try_from(x).map_err(|_|TryFromIntError)?;
                                        if (#unsigned_ident::MIN.0..=#unsigned_ident::MAX.0).contains(&y) {
                                            Ok(Self(y))
                                        }
                                        else {
                                            Err(TryFromIntError)
                                        }
                                    }
                                }
                            }
                        }
                        else {
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
                        }
                    });
                    smaller.chain(same).chain(bigger).collect::<TokenStream>()
                };
                let signed_from_implementations = {
                    let smaller = (1..size).map(|s| {
                        let from_unsigned_ident = format_ident!("u{s}");
                        let from_signed_ident = format_ident!("i{s}");
                        if s == 8 || s == 16 || s == 32 || s == 64 || s == 128 {
                            quote! {
                                impl From<#from_unsigned_ident> for #signed_ident {
                                    fn from(x: #from_unsigned_ident) -> #signed_ident {
                                        Self(x as #signed_inner_ident)
                                    }
                                }
                                impl From<#from_signed_ident> for #signed_ident {
                                    fn from(x: #from_signed_ident) -> #signed_ident {
                                        Self(x as #signed_inner_ident)
                                    }
                                }
                            }
                        }
                        else {
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
                        if s == 8 || s == 16 || s == 32 || s == 64 || s == 128 {
                            quote! {
                                impl TryFrom<#from_unsigned_ident> for #signed_ident {
                                    type Error = TryFromIntError;
                                    fn try_from(x: #from_unsigned_ident) -> Result<#signed_ident,Self::Error> {
                                        let y = #signed_inner_ident::try_from(x).map_err(|_|TryFromIntError)?;
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
                                        let y = #signed_inner_ident::try_from(x).map_err(|_|TryFromIntError)?;
                                        if (#signed_ident::MIN.0..=#signed_ident::MAX.0).contains(&y) {
                                            Ok(Self(y))
                                        }
                                        else {
                                            Err(TryFromIntError)
                                        }
                                    }
                                }
                            }
                        }
                        else {
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
                        }
                    });
                    smaller.chain(same).chain(bigger).collect::<TokenStream>()
                };
                quote! {
                    #[doc=#unsigned_doc]
                    #[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash)]
                    pub struct #unsigned_ident(#unsigned_inner_ident);
                    impl #unsigned_ident {
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
                        pub const BITS: u32 = #bits;

                        /// Create a native endian integer value from its representation as a byte
                        /// array in big endian.
                        pub fn from_be_bytes(bytes: [u8; #bytes]) -> Self {
                            let mut arr = [0u8; #inner_bytes];
                            unsafe {
                                (&mut arr[1] as *mut u8).copy_from_nonoverlapping(&bytes[0],#bytes);
                            }
                            Self(#unsigned_inner_ident::from_be_bytes(arr))
                        }
                        /// Create a native endian integer value from its representation as a byte
                        /// array in little endian.
                        pub fn from_le_bytes(bytes: [u8; #bytes]) -> Self {
                            let mut arr = [0u8; #inner_bytes];
                            unsafe {
                                (&mut arr[0] as *mut u8).copy_from_nonoverlapping(&bytes[0],#bytes);
                            }
                            Self(#unsigned_inner_ident::from_le_bytes(arr))
                        }
                        /// Create a native endian integer value from its memory representation as a
                        /// byte array in native endianness.
                        ///
                        /// As the target platform’s native endianness is used, portable code likely
                        /// wants to use `from_be_bytes` or `from_le_bytes`, as appropriate instead.
                        pub fn from_ne_bytes(bytes: [u8; #bytes]) -> Self {
                            // https://doc.rust-lang.org/reference/conditional-compilation.html#target_endian
                            #[cfg(target_endian="big")]
                            let ret = Self::from_be_bytes(bytes);
                            #[cfg(target_endian="little")]
                            let ret = Self::from_le_bytes(bytes);
                            ret
                        }

                        /// Return the memory representation of this integer as a byte array in
                        /// big-endian (network) byte order.
                        pub fn to_be_bytes(self) -> [u8; #bytes] {
                            let mut arr = self.0.to_be_bytes();
                            array_rsplit_array_mut::<_,#inner_bytes,#bytes>(&mut arr).1.clone()
                        }
                        /// Return the memory representation of this integer as a byte array in
                        /// little-endian byte order.
                        pub fn to_le_bytes(self) -> [u8; #bytes] {
                            let mut arr = self.0.to_le_bytes();
                            array_split_array_mut::<_,#inner_bytes,#bytes>(&mut arr).0.clone()
                        }
                        /// Return the memory representation of this integer as a byte array in
                        /// native byte order.
                        ///
                        /// As the target platform’s native endianness is used, portable code should
                        /// use `to_be_bytes` or `to_le_bytes`, as appropriate, instead.
                        pub fn to_ne_bytes(self) -> [u8; #bytes] {
                            // https://doc.rust-lang.org/reference/conditional-compilation.html#target_endian
                            #[cfg(target_endian="big")]
                            let ret = self.to_be_bytes();
                            #[cfg(target_endian="little")]
                            let ret = self.to_be_bytes();
                            ret
                        }
                    }

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

                    #[doc=#signed_doc]
                    #[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash)]
                    pub struct #signed_ident(#signed_inner_ident);
                    impl #signed_ident {
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
                        pub const BITS: u32 = #bits;
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
                        pub fn from_be_bytes(bytes: [u8; #bytes]) -> Self {
                            let mut arr = [0u8; #inner_bytes];
                            unsafe {
                                (&mut arr[1] as *mut u8).copy_from_nonoverlapping(&bytes[0],#bytes);
                            }
                            Self(#signed_inner_ident::from_be_bytes(arr))
                        }
                        /// Create a native endian integer value from its representation as a byte
                        /// array in little endian.
                        pub fn from_le_bytes(bytes: [u8; #bytes]) -> Self {
                            let mut arr = [0u8; #inner_bytes];
                            unsafe {
                                (&mut arr[0] as *mut u8).copy_from_nonoverlapping(&bytes[0],#bytes);
                            }
                            Self(#signed_inner_ident::from_le_bytes(arr))
                        }
                        /// Create a native endian integer value from its memory representation as a
                        /// byte array in native endianness.
                        ///
                        /// As the target platform’s native endianness is used, portable code likely
                        /// wants to use `from_be_bytes` or `from_le_bytes`, as appropriate instead.
                        pub fn from_ne_bytes(bytes: [u8; #bytes]) -> Self {
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
                        pub fn to_be_bytes(self) -> [u8; #bytes] {
                            let mut arr = self.0.to_be_bytes();
                            array_rsplit_array_mut::<_,#inner_bytes,#bytes>(&mut arr).1.clone()
                        }
                        /// Return the memory representation of this integer as a byte array in
                        /// little-endian byte order.
                        /// 
                        /// Unused bits are undefined.
                        pub fn to_le_bytes(self) -> [u8; #bytes] {
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
                        pub fn to_ne_bytes(self) -> [u8; #bytes] {
                            // https://doc.rust-lang.org/reference/conditional-compilation.html#target_endian
                            #[cfg(target_endian="big")]
                            let ret = self.to_be_bytes();
                            #[cfg(target_endian="little")]
                            let ret = self.to_be_bytes();
                            ret
                        }
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
                }
            }
        }
    }).collect::<proc_macro2::TokenStream>();

    output.into()
}
