use core::str::FromStr;

use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote};

#[cfg(target_pointer_width = "16")]
const TARGET_POINTER_WIDTH: u8 = 16;
#[cfg(target_pointer_width = "32")]
const TARGET_POINTER_WIDTH: u8 = 32;
#[cfg(target_pointer_width = "64")]
const TARGET_POINTER_WIDTH: u8 = 64;

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
        let inner_size = core::cmp::max(size.next_power_of_two(),8);
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

        let unsigned_add_ops = (1..max).filter(|rhs| core::cmp::max(rhs,&size) < &max).map(|s| {
            let rhs_ident = format_ident!("u{s}");
            let out_size = core::cmp::max(size,s) + 1;
            let out_ident = format_ident!("u{out_size}");

            quote! {
                impl core::ops::Add<&#rhs_ident> for &#unsigned_ident {
                    type Output = #out_ident;
                    fn add(self, rhs: &#rhs_ident) -> Self::Output {
                        let mut x = <#out_ident>::from(*self).0;
                        let y = <#out_ident>::from(*rhs).0;
                        x += y;
                        #out_ident(x)
                    }
                }
                impl core::ops::Add<&#rhs_ident> for #unsigned_ident {
                    type Output = #out_ident;
                    fn add(self, rhs: &#rhs_ident) -> Self::Output {
                        let mut x = <#out_ident>::from(self).0;
                        let y = <#out_ident>::from(*rhs).0;
                        x += y;
                        #out_ident(x)
                    }
                }
                impl<'a> core::ops::Add<#rhs_ident> for &'a #unsigned_ident {
                    type Output = #out_ident;
                    fn add(self, rhs: #rhs_ident) -> Self::Output {
                        let mut x = <#out_ident>::from(*self).0;
                        let y = <#out_ident>::from(rhs).0;
                        x += y;
                        #out_ident(x)
                    }
                }
                impl core::ops::Add<#rhs_ident> for #unsigned_ident {
                    type Output = #out_ident;
                    fn add(self, rhs: #rhs_ident) -> Self::Output {
                        let mut x = <#out_ident>::from(self).0;
                        let y = <#out_ident>::from(rhs).0;
                        x += y;
                        #out_ident(x)
                    }
                }
            }
        }).collect::<TokenStream>();

        let unsigned_mul_ops = (1..max).filter(|rhs| rhs+size <= max).map(|rhs|{
            let out_size = size+rhs;
            let rhs_ident = format_ident!("u{rhs}");
            let out_ident = format_ident!("u{out_size}");

            quote! {
                impl core::ops::Mul<&#rhs_ident> for &#unsigned_ident {
                    type Output = #out_ident;
                    fn mul(self, rhs: &#rhs_ident) -> Self::Output {
                        let mut x = <#out_ident>::from(*self).0;
                        let y = <#out_ident>::from(*rhs).0;
                        let z = x*y;
                        #out_ident(z)
                    }
                }
                impl core::ops::Mul<&#rhs_ident> for #unsigned_ident {
                    type Output = #out_ident;
                    fn mul(self, rhs: &#rhs_ident) -> Self::Output {
                        let mut x = <#out_ident>::from(self).0;
                        let y = <#out_ident>::from(*rhs).0;
                        let z = x*y;
                        #out_ident(z)
                    }
                }
                impl<'a> core::ops::Mul<#rhs_ident> for &'a #unsigned_ident {
                    type Output = #out_ident;
                    fn mul(self, rhs: #rhs_ident) -> Self::Output {
                        let mut x = <#out_ident>::from(*self).0;
                        let y = <#out_ident>::from(rhs).0;
                        let z = x*y;
                        #out_ident(z)
                    }
                }
                impl core::ops::Mul<#rhs_ident> for #unsigned_ident {
                    type Output = #out_ident;
                    fn mul(self, rhs: #rhs_ident) -> Self::Output {
                        let mut x = <#out_ident>::from(self).0;
                        let y = <#out_ident>::from(rhs).0;
                        let z = x*y;
                        #out_ident(z)
                    }
                }
            }
        }).collect::<TokenStream>();

        let unsigned_sub_ops = (1..max).filter(|rhs| core::cmp::max(rhs,&size) < &max).map(|s| {
            let rhs_ident = format_ident!("u{s}");
            let out_size = core::cmp::max(size,s)+1;
            let out_ident = format_ident!("i{out_size}");

            quote! {
                impl core::ops::Sub<&#rhs_ident> for &#unsigned_ident {
                    type Output = #out_ident;
                    fn sub(self, rhs: &#rhs_ident) -> Self::Output {
                        let mut x = <#out_ident>::from(*self).0;
                        let y = <#out_ident>::from(*rhs).0;
                        x -= y;
                        #out_ident(x)
                    }
                }
                impl core::ops::Sub<&#rhs_ident> for #unsigned_ident {
                    type Output = #out_ident;
                    fn sub(self, rhs: &#rhs_ident) -> Self::Output {
                        let mut x = <#out_ident>::from(self).0;
                        let y = <#out_ident>::from(*rhs).0;
                        x -= y;
                        #out_ident(x)
                    }
                }
                impl<'a> core::ops::Sub<#rhs_ident> for &'a #unsigned_ident {
                    type Output = #out_ident;
                    fn sub(self, rhs: #rhs_ident) -> Self::Output {
                        let mut x = <#out_ident>::from(*self).0;
                        let y = <#out_ident>::from(rhs).0;
                        x -= y;
                        #out_ident(x)
                    }
                }
                impl core::ops::Sub<#rhs_ident> for #unsigned_ident {
                    type Output = #out_ident;
                    fn sub(self, rhs: #rhs_ident) -> Self::Output {
                        let mut x = <#out_ident>::from(self).0;
                        let y = <#out_ident>::from(rhs).0;
                        x -= y;
                        #out_ident(x)
                    }
                }
            }
        }).collect::<TokenStream>();

        let unsigned_rem_ops = (1..max).filter(|rhs|rhs < &max).map(|s|{
            let rhs_ident = format_ident!("u{s}");
            let out_size = core::cmp::min(size,s);
            let out_ident = format_ident!("u{out_size}");
            let max_ident = core::cmp::max(size,s);
            let max_ident = format_ident!("u{max_ident}");
            quote! {
                impl core::ops::Rem<&#rhs_ident> for &#unsigned_ident {
                    type Output = #out_ident;
                    fn rem(self, rhs: &#rhs_ident) -> Self::Output {
                        let x = <#max_ident>::from(*self).0;
                        let y = <#max_ident>::from(*rhs).0;
                        #out_ident::try_from(x % y).unwrap()
                    }
                }
                impl core::ops::Rem<&#rhs_ident> for #unsigned_ident {
                    type Output = #out_ident;
                    fn rem(self, rhs: &#rhs_ident) -> Self::Output {
                        let x = <#max_ident>::from(self).0;
                        let y = <#max_ident>::from(*rhs).0;
                        #out_ident::try_from(x % y).unwrap()
                    }
                }
                impl<'a> core::ops::Rem<#rhs_ident> for &'a #unsigned_ident {
                    type Output = #out_ident;
                    fn rem(self, rhs: #rhs_ident) -> Self::Output {
                        let x = <#max_ident>::from(*self).0;
                        let y = <#max_ident>::from(rhs).0;
                        #out_ident::try_from(x % y).unwrap()
                    }
                }
                impl core::ops::Rem<#rhs_ident> for #unsigned_ident {
                    type Output = #out_ident;
                    fn rem(self, rhs: #rhs_ident) -> Self::Output {
                        let x = <#max_ident>::from(self).0;
                        let y = <#max_ident>::from(rhs).0;
                        #out_ident::try_from(x % y).unwrap()
                    }
                }
            }
        }).collect::<TokenStream>();

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

        let signed_add_ops = (1..max).filter(|rhs| core::cmp::max(rhs,&size) < &max).map(|s| {
            let signed_rhs = format_ident!("i{s}");
            let output_size = core::cmp::max(size,s) + 1;
            let signed_ident_plus_one = format_ident!("i{output_size}");

            quote! {
                impl core::ops::Add<&#signed_rhs> for &#signed_ident {
                    type Output = #signed_ident_plus_one;
                    fn add(self, rhs: &#signed_rhs) -> Self::Output {
                        let mut x = <#signed_ident_plus_one>::from(*self).0;
                        let y = <#signed_ident_plus_one>::from(*rhs).0;
                        x += y;
                        #signed_ident_plus_one(x)
                    }
                }
                impl core::ops::Add<&#signed_rhs> for #signed_ident {
                    type Output = #signed_ident_plus_one;
                    fn add(self, rhs: &#signed_rhs) -> Self::Output {
                        let mut x = <#signed_ident_plus_one>::from(self).0;
                        let y = <#signed_ident_plus_one>::from(*rhs).0;
                        x += y;
                        #signed_ident_plus_one(x)
                    }
                }
                impl<'a> core::ops::Add<#signed_rhs> for &'a #signed_ident {
                    type Output = #signed_ident_plus_one;
                    fn add(self, rhs: #signed_rhs) -> Self::Output {
                        let mut x = <#signed_ident_plus_one>::from(*self).0;
                        let y = <#signed_ident_plus_one>::from(rhs).0;
                        x += y;
                        #signed_ident_plus_one(x)
                    }
                }
                impl core::ops::Add<#signed_rhs> for #signed_ident {
                    type Output = #signed_ident_plus_one;
                    fn add(self, rhs: #signed_rhs) -> Self::Output {
                        let mut x = <#signed_ident_plus_one>::from(self).0;
                        let y = <#signed_ident_plus_one>::from(rhs).0;
                        x += y;
                        #signed_ident_plus_one(x)
                    }
                }
            }
        }).collect::<TokenStream>();

        let signed_mul_ops = (1..max).filter(|rhs| rhs+size <= max).map(|rhs|{
            let out = size+rhs;
            let rhs_ident = format_ident!("i{rhs}");
            let out_ident = format_ident!("i{out}");

            quote! {
                impl core::ops::Mul<&#rhs_ident> for &#signed_ident {
                    type Output = #out_ident;
                    fn mul(self, rhs: &#rhs_ident) -> Self::Output {
                        let mut x = <#out_ident>::from(*self).0;
                        let y = <#out_ident>::from(*rhs).0;
                        let z = x*y;
                        #out_ident(z)
                    }
                }
                impl core::ops::Mul<&#rhs_ident> for #signed_ident {
                    type Output = #out_ident;
                    fn mul(self, rhs: &#rhs_ident) -> Self::Output {
                        let mut x = <#out_ident>::from(self).0;
                        let y = <#out_ident>::from(*rhs).0;
                        let z = x*y;
                        #out_ident(z)
                    }
                }
                impl<'a> core::ops::Mul<#rhs_ident> for &'a #signed_ident {
                    type Output = #out_ident;
                    fn mul(self, rhs: #rhs_ident) -> Self::Output {
                        let mut x = <#out_ident>::from(*self).0;
                        let y = <#out_ident>::from(rhs).0;
                        let z = x*y;
                        #out_ident(z)
                    }
                }
                impl core::ops::Mul<#rhs_ident> for #signed_ident {
                    type Output = #out_ident;
                    fn mul(self, rhs: #rhs_ident) -> Self::Output {
                        let mut x = <#out_ident>::from(self).0;
                        let y = <#out_ident>::from(rhs).0;
                        let z = x*y;
                        #out_ident(z)
                    }
                }
            }
        }).collect::<TokenStream>();

        let signed_sub_ops = (1..max).filter(|rhs| core::cmp::max(rhs,&size) < &max).map(|s| {
            let signed_rhs = format_ident!("i{s}");
            let output_size = core::cmp::max(size,s) + 1;
            let signed_ident_plus_one = format_ident!("i{output_size}");

            quote! {
                impl core::ops::Sub<&#signed_rhs> for &#signed_ident {
                    type Output = #signed_ident_plus_one;
                    fn sub(self, rhs: &#signed_rhs) -> Self::Output {
                        let mut x = <#signed_ident_plus_one>::from(*self).0;
                        let y = <#signed_ident_plus_one>::from(*rhs).0;
                        x -= y;
                        #signed_ident_plus_one(x)
                    }
                }
                impl core::ops::Sub<&#signed_rhs> for #signed_ident {
                    type Output = #signed_ident_plus_one;
                    fn sub(self, rhs: &#signed_rhs) -> Self::Output {
                        let mut x = <#signed_ident_plus_one>::from(self).0;
                        let y = <#signed_ident_plus_one>::from(*rhs).0;
                        x -= y;
                        #signed_ident_plus_one(x)
                    }
                }
                impl<'a> core::ops::Sub<#signed_rhs> for &'a #signed_ident {
                    type Output = #signed_ident_plus_one;
                    fn sub(self, rhs: #signed_rhs) -> Self::Output {
                        let mut x = <#signed_ident_plus_one>::from(*self).0;
                        let y = <#signed_ident_plus_one>::from(rhs).0;
                        x -= y;
                        #signed_ident_plus_one(x)
                    }
                }
                impl core::ops::Sub<#signed_rhs> for #signed_ident {
                    type Output = #signed_ident_plus_one;
                    fn sub(self, rhs: #signed_rhs) -> Self::Output {
                        let mut x = <#signed_ident_plus_one>::from(self).0;
                        let y = <#signed_ident_plus_one>::from(rhs).0;
                        x -= y;
                        #signed_ident_plus_one(x)
                    }
                }
            }
        }).collect::<TokenStream>();

        let signed_rem_ops = (1..max).filter(|rhs|rhs < &max).map(|s|{
            let rhs_ident = format_ident!("i{s}");
            let out_size = core::cmp::min(size,s);
            let out_ident = format_ident!("i{out_size}");
            let max_ident = core::cmp::max(size,s);
            let max_ident = format_ident!("i{max_ident}");
            quote! {
                impl core::ops::Rem<&#rhs_ident> for &#signed_ident {
                    type Output = #out_ident;
                    fn rem(self, rhs: &#rhs_ident) -> Self::Output {
                        let x = <#max_ident>::from(*self).0;
                        let y = <#max_ident>::from(*rhs).0;
                        #out_ident::try_from(x % y).unwrap()
                    }
                }
                impl core::ops::Rem<&#rhs_ident> for #signed_ident {
                    type Output = #out_ident;
                    fn rem(self, rhs: &#rhs_ident) -> Self::Output {
                        let x = <#max_ident>::from(self).0;
                        let y = <#max_ident>::from(*rhs).0;
                        #out_ident::try_from(x % y).unwrap()
                    }
                }
                impl<'a> core::ops::Rem<#rhs_ident> for &'a #signed_ident {
                    type Output = #out_ident;
                    fn rem(self, rhs: #rhs_ident) -> Self::Output {
                        let x = <#max_ident>::from(*self).0;
                        let y = <#max_ident>::from(rhs).0;
                        #out_ident::try_from(x % y).unwrap()
                    }
                }
                impl core::ops::Rem<#rhs_ident> for #signed_ident {
                    type Output = #out_ident;
                    fn rem(self, rhs: #rhs_ident) -> Self::Output {
                        let x = <#max_ident>::from(self).0;
                        let y = <#max_ident>::from(rhs).0;
                        #out_ident::try_from(x % y).unwrap()
                    }
                }
            }
        }).collect::<TokenStream>();

        let unsigned_from_implementations = {
            let pointer_width_from = core::iter::once(match size.cmp(&TARGET_POINTER_WIDTH) {
                core::cmp::Ordering::Greater => quote! {
                    impl From<usize> for #unsigned_ident {
                        fn from(x: usize) -> Self {
                            Self(x as #unsigned_inner_ident)
                        }
                    }
                    impl TryFrom<#unsigned_ident> for usize {
                        type Error = TryFromIntError;
                        fn try_from(x: #unsigned_ident) -> Result<Self, Self::Error> {
                            usize::try_from(x.0).map_err(|_|TryFromIntError)
                        }
                    }
                },
                core::cmp::Ordering::Equal => quote! {
                    impl From<usize> for #unsigned_ident {
                        fn from(x: usize) -> Self {
                            Self(x as #unsigned_inner_ident)
                        }
                    }
                    impl From<#unsigned_ident> for usize {
                        fn from(x: #unsigned_ident) -> Self {
                            x.0 as usize
                        }
                    }
                },
                core::cmp::Ordering::Less => quote! {
                    impl TryFrom<usize> for #unsigned_ident {
                        type Error = TryFromIntError;
                        fn try_from(x: usize) -> Result<Self, Self::Error> {
                            let y = <#unsigned_inner_ident>::try_from(x).map_err(|_|TryFromIntError)?;
                            if (Self::MIN.0..=Self::MAX.0).contains(&y) {
                                Ok(Self(y))
                            }
                            else {
                                Err(TryFromIntError)
                            }
                        }
                    }
                    impl From<#unsigned_ident> for usize {
                        fn from(x: #unsigned_ident) -> Self {
                            x.0 as usize
                        }
                    }
                }
            });

            let primitive_from = [8,16,32,64,128].into_iter().map(|s| {
                let from_ident = format_ident!("u{s}");
                let from_ident = quote! { core::primitive::#from_ident };
                match size.cmp(&s) {
                    core::cmp::Ordering::Equal => quote!{
                        impl From<#from_ident> for #unsigned_ident {
                            fn from(x: #from_ident) -> Self {
                                Self(<#unsigned_inner_ident>::from(x))
                            }
                        }
                        impl From<#unsigned_ident> for #from_ident {
                            fn from(x: #unsigned_ident) -> Self {
                                <#from_ident>::from(x.0)
                            }
                        }
                    },
                    core::cmp::Ordering::Greater => quote! {
                        impl From<#from_ident> for #unsigned_ident {
                            fn from(x: #from_ident) -> Self {
                                Self(<#unsigned_inner_ident>::from(x))
                            }
                        }
                        impl TryFrom<#unsigned_ident> for #from_ident {
                            type Error = TryFromIntError;
                            fn try_from(x: #unsigned_ident) -> Result<Self, Self::Error> {
                                <#from_ident>::try_from(x.0).map_err(|_|TryFromIntError)
                            }
                        }
                    },
                    core::cmp::Ordering::Less => quote! {
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
                        impl From<#unsigned_ident> for #from_ident {
                            fn from(x: #unsigned_ident) -> Self {
                                <#from_ident>::from(x.0)
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
            let same = core::iter::once({
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
            pointer_width_from.chain(primitive_from).chain(smaller).chain(same).chain(bigger).collect::<TokenStream>()
        };
        let signed_from_implementations = {
            let pointer_width_from = core::iter::once(match size.cmp(&TARGET_POINTER_WIDTH) {
                core::cmp::Ordering::Greater => quote! {
                    impl From<isize> for #signed_ident {
                        fn from(x: isize) -> Self {
                            Self(x as #signed_inner_ident)
                        }
                    }
                    impl TryFrom<#signed_ident> for isize {
                        type Error = TryFromIntError;
                        fn try_from(x: #signed_ident) -> Result<Self, Self::Error> {
                            isize::try_from(x.0).map_err(|_|TryFromIntError)
                        }
                    }
                },
                core::cmp::Ordering::Equal => quote! {
                    impl From<isize> for #signed_ident {
                        fn from(x: isize) -> Self {
                            Self(x as #signed_inner_ident)
                        }
                    }
                    impl From<#signed_ident> for isize {
                        fn from(x: #signed_ident) -> Self {
                            x.0 as isize
                        }
                    }
                },
                core::cmp::Ordering::Less => quote! {
                    impl TryFrom<isize> for #signed_ident {
                        type Error = TryFromIntError;
                        fn try_from(x: isize) -> Result<Self,Self::Error> {
                            let y = <#signed_inner_ident>::try_from(x).map_err(|_|TryFromIntError)?;
                            if (Self::MIN.0..=Self::MAX.0).contains(&y) {
                                Ok(Self(y))
                            }
                            else {
                                Err(TryFromIntError)
                            }
                        }
                    }
                    impl From<#signed_ident> for isize {
                        fn from(x: #signed_ident) -> Self {
                            x.0 as isize
                        }
                    }
                }
            });

            let primitive_from = [8,16,32,64,128].into_iter().map(|s| {
                let from_ident = format_ident!("i{s}");
                let from_ident = quote! { core::primitive::#from_ident };
                match size.cmp(&s) {
                    core::cmp::Ordering::Equal => quote!{
                        impl From<#from_ident> for #signed_ident {
                            fn from(x: #from_ident) -> Self {
                                Self(<#signed_inner_ident>::from(x))
                            }
                        }
                        impl From<#signed_ident> for #from_ident {
                            fn from(x: #signed_ident) -> Self {
                                <#from_ident>::from(x.0)
                            }
                        }
                    },
                    core::cmp::Ordering::Greater => quote! {
                        impl From<#from_ident> for #signed_ident {
                            fn from(x: #from_ident) -> Self {
                                Self(<#signed_inner_ident>::from(x))
                            }
                        }
                        impl TryFrom<#signed_ident> for #from_ident {
                            type Error = TryFromIntError;
                            fn try_from(x: #signed_ident) -> Result<Self,Self::Error> {
                                <#from_ident>::try_from(x.0).map_err(|_|TryFromIntError)
                            }
                        }
                    },
                    core::cmp::Ordering::Less => quote! {
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
                        impl From<#signed_ident> for #from_ident {
                            fn from(x: #signed_ident) -> Self {
                                <#from_ident>::from(x.0)
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

            let same = core::iter::once({
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
            pointer_width_from.chain(primitive_from).chain(smaller).chain(same).chain(bigger).collect::<TokenStream>()
        };
        let unsigned_mask = match size {
            8 | 16 | 32 | 64 | 128 => quote! { #unsigned_min_ident },
            _ => quote! { (#unsigned_one << #size) }
        };
        let signed_mask = match size {
            8 | 16 | 32 | 64 | 128 => quote! { #signed_zero },
            _ => quote! { (#signed_one << #size) }
        };

        let signed_abs = if let Some(minus @ 1..) = size.checked_sub(1) {
            let ident = format_ident!("u{minus}");
            let inner_size = core::cmp::max(minus.next_power_of_two(),8);
            let inner_ident = format_ident!("u{inner_size}");
            let inner_ident = quote! { core::primitive::#inner_ident };
            quote! {
                /// Computes the absolute value of self.
                pub const fn abs(self) -> #ident {
                    #ident(self.0.abs() as #inner_ident)
                }
            }
        } else {
            TokenStream::new()
        };

        quote! {
            #[doc=#unsigned_doc]
            #[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
            #[repr(transparent)]
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

                /// Equivalent of an `as` cast.
                ///
                /// # Panics
                ///
                /// When `x` is out of bounds.
                pub const fn new(x: #unsigned_inner_ident) -> Self {
                    assert!(x >= Self::MIN.0 && x <= Self::MAX.0);
                    Self(x)
                }

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

            #unsigned_add_ops
            #unsigned_mul_ops
            #unsigned_sub_ops
            #unsigned_rem_ops

            impl core::ops::Div<&#unsigned_ident> for &#unsigned_ident {
                type Output = #unsigned_ident;
                fn div(self, rhs: &#unsigned_ident) -> Self::Output {
                    let x = self.0 / rhs.0;
                    debug_assert!((#unsigned_ident::MIN.0..#unsigned_ident::MAX.0).contains(&x));
                    #unsigned_ident(x)
                }
            }
            impl core::ops::Div<&#unsigned_ident> for #unsigned_ident {
                type Output = #unsigned_ident;
                fn div(self, rhs: &#unsigned_ident) -> Self::Output {
                    let x = self.0 / rhs.0;
                    debug_assert!((#unsigned_ident::MIN.0..#unsigned_ident::MAX.0).contains(&x));
                    #unsigned_ident(x)
                }
            }
            impl<'a> core::ops::Div<#unsigned_ident> for &'a #unsigned_ident {
                type Output = #unsigned_ident;
                fn div(self, rhs: #unsigned_ident) -> Self::Output {
                    let x = self.0 / rhs.0;
                    debug_assert!((#unsigned_ident::MIN.0..#unsigned_ident::MAX.0).contains(&x));
                    #unsigned_ident(x)
                }
            }
            impl core::ops::Div<#unsigned_ident> for #unsigned_ident {
                type Output = #unsigned_ident;
                fn div(self, rhs: #unsigned_ident) -> Self::Output {
                    let x = self.0 / rhs.0;
                    debug_assert!((#unsigned_ident::MIN.0..#unsigned_ident::MAX.0).contains(&x));
                    #unsigned_ident(x)
                }
            }

            impl core::fmt::Display for #unsigned_ident {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    write!(f,"{}",self.0)
                }
            }
            impl core::fmt::Binary for #unsigned_ident {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    core::fmt::Binary::fmt(&self.0, f)
                }
            }
            impl core::fmt::LowerHex for #unsigned_ident {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    core::fmt::LowerHex::fmt(&self.0, f)
                }
            }
            impl core::fmt::UpperHex for #unsigned_ident {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    core::fmt::UpperHex::fmt(&self.0, f)
                }
            }
            impl core::fmt::Octal for #unsigned_ident {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    core::fmt::Octal::fmt(&self.0, f)
                }
            }

            impl core::ops::BitAnd<&#unsigned_ident> for &#unsigned_ident {
                type Output = <#unsigned_ident as core::ops::BitAnd<#unsigned_ident>>::Output;

                fn bitand(self, rhs: &#unsigned_ident) -> Self::Output {
                    #unsigned_ident(self.0 & rhs.0)
                }
            }
            impl core::ops::BitAnd<&#unsigned_ident> for #unsigned_ident {
                type Output = <#unsigned_ident as core::ops::BitAnd<#unsigned_ident>>::Output;

                fn bitand(self, rhs: &#unsigned_ident) -> Self::Output {
                    #unsigned_ident(self.0 & rhs.0)
                }
            }
            impl<'a> core::ops::BitAnd<#unsigned_ident> for &'a #unsigned_ident {
                type Output = <#unsigned_ident as core::ops::BitAnd<#unsigned_ident>>::Output;

                fn bitand(self, rhs: #unsigned_ident) -> Self::Output {
                    #unsigned_ident(self.0 & rhs.0)
                }
            }
            impl core::ops::BitAnd<#unsigned_ident> for #unsigned_ident {
                type Output = Self;

                fn bitand(self, rhs: Self) -> Self::Output {
                    #unsigned_ident(self.0 & rhs.0)
                }
            }

            impl core::ops::BitOr<&#unsigned_ident> for &#unsigned_ident {
                type Output = <#unsigned_ident as core::ops::BitOr<#unsigned_ident>>::Output;

                fn bitor(self, rhs: &#unsigned_ident) -> Self::Output {
                    #unsigned_ident(self.0 | rhs.0)
                }
            }
            impl core::ops::BitOr<&#unsigned_ident> for #unsigned_ident {
                type Output = <#unsigned_ident as core::ops::BitOr<#unsigned_ident>>::Output;

                fn bitor(self, rhs: &#unsigned_ident) -> Self::Output {
                    #unsigned_ident(self.0 | rhs.0)
                }
            }
            impl<'a> core::ops::BitOr<#unsigned_ident> for &'a #unsigned_ident {
                type Output = <#unsigned_ident as core::ops::BitOr<#unsigned_ident>>::Output;

                fn bitor(self, rhs: #unsigned_ident) -> Self::Output {
                    #unsigned_ident(self.0 | rhs.0)
                }
            }
            impl core::ops::BitOr<#unsigned_ident> for #unsigned_ident {
                type Output = Self;

                fn bitor(self, rhs: Self) -> Self::Output {
                    #unsigned_ident(self.0 | rhs.0)
                }
            }

            impl core::ops::BitXor<&#unsigned_ident> for &#unsigned_ident {
                type Output = <#unsigned_ident as core::ops::BitXor<#unsigned_ident>>::Output;

                fn bitxor(self, rhs: &#unsigned_ident) -> Self::Output {
                    #unsigned_ident(self.0 ^ rhs.0)
                }
            }
            impl core::ops::BitXor<&#unsigned_ident> for #unsigned_ident {
                type Output = <#unsigned_ident as core::ops::BitXor<#unsigned_ident>>::Output;

                fn bitxor(self, rhs: &#unsigned_ident) -> Self::Output {
                    #unsigned_ident(self.0 ^ rhs.0)
                }
            }
            impl<'a> core::ops::BitXor<#unsigned_ident> for &'a #unsigned_ident {
                type Output = <#unsigned_ident as core::ops::BitXor<#unsigned_ident>>::Output;

                fn bitxor(self, rhs: #unsigned_ident) -> Self::Output {
                    #unsigned_ident(self.0 ^ rhs.0)
                }
            }
            impl core::ops::BitXor<#unsigned_ident> for #unsigned_ident {
                type Output = Self;

                fn bitxor(self, rhs: Self) -> Self::Output {
                    #unsigned_ident(self.0 ^ rhs.0)
                }
            }

            impl core::ops::Not for #unsigned_ident {
                type Output = Self;

                fn not(self) -> Self::Output {
                    #unsigned_ident(self.0)
                }
            }
            impl core::ops::Not for &#unsigned_ident {
                type Output = <#unsigned_ident as core::ops::Not>::Output;

                fn not(self) -> Self::Output {
                    #unsigned_ident(self.0)
                }
            }

            #unsigned_from_implementations

            #[doc=#signed_doc]
            #[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
            #[repr(transparent)]
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

                /// Equivalent of an `as` cast.
                ///
                /// # Panics
                ///
                /// When `x` is out of bounds.
                pub const fn new(x: #signed_inner_ident) -> Self {
                    assert!(x >= Self::MIN.0 && x <= Self::MAX.0);
                    Self(x)
                }

                #signed_abs

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
            }

            #signed_add_ops
            #signed_mul_ops
            #signed_sub_ops
            #signed_rem_ops

            impl core::ops::Div<&#signed_ident> for &#signed_ident {
                type Output = #signed_ident;
                fn div(self, rhs: &#signed_ident) -> Self::Output {
                    let x = self.0 / rhs.0;
                    debug_assert!((#signed_ident::MIN.0..#signed_ident::MAX.0).contains(&x));
                    #signed_ident(x)
                }
            }
            impl core::ops::Div<&#signed_ident> for #signed_ident {
                type Output = #signed_ident;
                fn div(self, rhs: &#signed_ident) -> Self::Output {
                    let x = self.0 / rhs.0;
                    debug_assert!((#signed_ident::MIN.0..#signed_ident::MAX.0).contains(&x));
                    #signed_ident(x)
                }
            }
            impl<'a> core::ops::Div<#signed_ident> for &'a #signed_ident {
                type Output = #signed_ident;
                fn div(self, rhs: #signed_ident) -> Self::Output {
                    let x = self.0 / rhs.0;
                    debug_assert!((#signed_ident::MIN.0..#signed_ident::MAX.0).contains(&x));
                    #signed_ident(x)
                }
            }
            impl core::ops::Div<#signed_ident> for #signed_ident {
                type Output = #signed_ident;
                fn div(self, rhs: #signed_ident) -> Self::Output {
                    let x = self.0 / rhs.0;
                    debug_assert!((#signed_ident::MIN.0..#signed_ident::MAX.0).contains(&x));
                    #signed_ident(x)
                }
            }

            impl core::fmt::Display for #signed_ident {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    write!(f,"{}",self.0)
                }
            }
            impl core::fmt::Binary for #signed_ident {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    core::fmt::Binary::fmt(&self.0, f)
                }
            }
            impl core::fmt::LowerHex for #signed_ident {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    core::fmt::LowerHex::fmt(&self.0, f)
                }
            }
            impl core::fmt::UpperHex for #signed_ident {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    core::fmt::UpperHex::fmt(&self.0, f)
                }
            }
            impl core::fmt::Octal for #signed_ident {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    core::fmt::Octal::fmt(&self.0, f)
                }
            }

            impl core::ops::BitAnd<&#signed_ident> for &#signed_ident {
                type Output = <#signed_ident as core::ops::BitAnd<#signed_ident>>::Output;

                fn bitand(self, rhs: &#signed_ident) -> Self::Output {
                    #signed_ident(self.0 & rhs.0)
                }
            }
            impl core::ops::BitAnd<&#signed_ident> for #signed_ident {
                type Output = <#signed_ident as core::ops::BitAnd<#signed_ident>>::Output;

                fn bitand(self, rhs: &#signed_ident) -> Self::Output {
                    #signed_ident(self.0 & rhs.0)
                }
            }
            impl<'a> core::ops::BitAnd<#signed_ident> for &'a #signed_ident {
                type Output = <#signed_ident as core::ops::BitAnd<#signed_ident>>::Output;

                fn bitand(self, rhs: #signed_ident) -> Self::Output {
                    #signed_ident(self.0 & rhs.0)
                }
            }
            impl core::ops::BitAnd<#signed_ident> for #signed_ident {
                type Output = Self;

                fn bitand(self, rhs: Self) -> Self::Output {
                    #signed_ident(self.0 & rhs.0)
                }
            }

            impl core::ops::BitOr<&#signed_ident> for &#signed_ident {
                type Output = <#signed_ident as core::ops::BitOr<#signed_ident>>::Output;

                fn bitor(self, rhs: &#signed_ident) -> Self::Output {
                    #signed_ident(self.0 | rhs.0)
                }
            }
            impl core::ops::BitOr<&#signed_ident> for #signed_ident {
                type Output = <#signed_ident as core::ops::BitOr<#signed_ident>>::Output;

                fn bitor(self, rhs: &#signed_ident) -> Self::Output {
                    #signed_ident(self.0 | rhs.0)
                }
            }
            impl<'a> core::ops::BitOr<#signed_ident> for &'a #signed_ident {
                type Output = <#signed_ident as core::ops::BitOr<#signed_ident>>::Output;

                fn bitor(self, rhs: #signed_ident) -> Self::Output {
                    #signed_ident(self.0 | rhs.0)
                }
            }
            impl core::ops::BitOr<#signed_ident> for #signed_ident {
                type Output = Self;

                fn bitor(self, rhs: Self) -> Self::Output {
                    #signed_ident(self.0 | rhs.0)
                }
            }

            impl core::ops::BitXor<&#signed_ident> for &#signed_ident {
                type Output = <#signed_ident as core::ops::BitXor<#signed_ident>>::Output;

                fn bitxor(self, rhs: &#signed_ident) -> Self::Output {
                    #signed_ident(self.0 ^ rhs.0)
                }
            }
            impl core::ops::BitXor<&#signed_ident> for #signed_ident {
                type Output = <#signed_ident as core::ops::BitXor<#signed_ident>>::Output;

                fn bitxor(self, rhs: &#signed_ident) -> Self::Output {
                    #signed_ident(self.0 ^ rhs.0)
                }
            }
            impl<'a> core::ops::BitXor<#signed_ident> for &'a #signed_ident {
                type Output = <#signed_ident as core::ops::BitXor<#signed_ident>>::Output;

                fn bitxor(self, rhs: #signed_ident) -> Self::Output {
                    #signed_ident(self.0 ^ rhs.0)
                }
            }
            impl core::ops::BitXor<#signed_ident> for #signed_ident {
                type Output = Self;

                fn bitxor(self, rhs: Self) -> Self::Output {
                    #signed_ident(self.0 ^ rhs.0)
                }
            }

            impl core::ops::Neg for #signed_ident {
                type Output = Self;
                fn neg(self) -> Self::Output {
                    assert_ne!(self,Self::MIN);
                    #signed_ident(-self.0)
                }
            }

            impl core::ops::Not for #signed_ident {
                type Output = Self;

                fn not(self) -> Self::Output {
                    #signed_ident(self.0)
                }
            }
            impl core::ops::Not for &#signed_ident {
                type Output = <#signed_ident as core::ops::Not>::Output;

                fn not(self) -> Self::Output {
                    #signed_ident(self.0)
                }
            }

            #signed_from_implementations

        }
    }).collect::<proc_macro2::TokenStream>();

    output.into()
}
