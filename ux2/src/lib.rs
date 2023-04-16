#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// https://doc.rust-lang.org/rustdoc/unstable-features.html#doc_auto_cfg-automatically-generate-doccfg

ux2_macros::generate_types!(128);

/// A mimic of [`std::num::TryFromIntError`] that can constructed on stable.
#[derive(Debug, Eq, PartialEq)]
pub struct TryFromIntError;
impl std::fmt::Display for TryFromIntError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Failed `TryFrom`.")
    }
}
impl std::error::Error for TryFromIntError {}

/// A mimic of [`std::num::ParseIntError`] that can constructed on stable.
#[derive(Debug, Eq, PartialEq)]
pub struct ParseIntError;
impl std::fmt::Display for ParseIntError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Failed `TryFrom`.")
    }
}
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
