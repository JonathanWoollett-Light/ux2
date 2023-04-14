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
