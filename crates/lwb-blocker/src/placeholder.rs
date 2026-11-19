//! Placeholder module so the crate compiles and has a test.

/// The crate's package name, as Cargo knows it.
pub const CRATE_NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_name_is_set() {
        assert_eq!(CRATE_NAME, "lwb-blocker");
    }
}
