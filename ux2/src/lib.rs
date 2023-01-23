ux2_macros::generate_types!(8);

/// A mimic of [`std::num::TryFromIntError`] that can constructed on stable.
#[derive(Debug, Eq, PartialEq)]
pub struct TryFromIntError;
impl std::fmt::Display for TryFromIntError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Failed `TryFrom`.")
    }
}
impl std::error::Error for TryFromIntError {}

// #![feature(fmt_helpers_for_derive)]
// #![feature(structural_match)]
// #![feature(no_coverage)]
// #![feature(derive_eq)]
