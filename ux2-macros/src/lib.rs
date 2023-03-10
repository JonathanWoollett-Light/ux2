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
    // eprintln!("max: {}",max);

    let output = (1..=max).filter_map(|size| {
        // eprintln!("size: {size}");

        if size == 8 || size == 16 || size == 32 || size == 64 || size == 128 {
            None
        }
        else {
            let inner_size = std::cmp::max(size.next_power_of_two(),8);
            let bits = size as u32;

            let unsigned_doc = format!(" The {size}-bit unsigned integer type.");
            let unsigned_inner_ident = format_ident!("u{inner_size}");
            let unsigned_ident = format_ident!("u{size}");
            let unsigned_max = 2u128.pow(u32::from(size)) - 1;
            let unsigned_max_ident = match size {
                0..=8 => Literal::u8_suffixed(unsigned_max as u8),
                9..=16 => Literal::u16_suffixed(unsigned_max as u16),
                17..=32 => Literal::u32_suffixed(unsigned_max as u32),
                33..=64 => Literal::u64_suffixed(unsigned_max as u64),
                65..=128 => Literal::u128_suffixed(unsigned_max as u128),
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
                format!(" // `i1` cannot contain `1`.")
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

            let check = quote! {
                #[doc=#unsigned_doc]
                #[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Copy)]
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

                #unsigned_from_implementations

                #[doc=#signed_doc]
                #[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Copy)]
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
                    pub fn abs(self) -> #signed_ident {
                        Self(self.0.abs())
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

                #signed_from_implementations
            };

            // eprintln!("check: {}",check);

            Some(check)
        }
    }).collect::<proc_macro2::TokenStream>();
    // eprintln!("output: {}",output);

    output.into()
}
