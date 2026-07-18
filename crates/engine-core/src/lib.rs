// crates/engine-core/src/lib.rs
#![deny(unsafe_code)]

pub fn engine_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_semver_like() {
        assert!(!engine_version().is_empty());
    }
}
