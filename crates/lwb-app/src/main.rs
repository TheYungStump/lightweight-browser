//! The browser binary. Phase 0a stub: prints a version line and exits.

fn version_line() -> String {
    format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
}

fn main() {
    println!("{}", version_line());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_line_has_name_and_version() {
        assert_eq!(version_line(), "lwb-app 0.0.0");
    }
}
