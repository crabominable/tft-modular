// crates/engine-core/src/lib.rs
#![deny(unsafe_code)]

pub mod fixed;
pub mod ids;
pub mod plugin;
pub mod rng;

pub use fixed::{fp_from_i64, fp_mul, Fp, FP_ONE};
pub use ids::{DefId, PlayerId, UnitInstanceId};
pub use plugin::{
    AbilityDef, AbilityEffect, AbilityTrigger, EffectType, Manifest, PluginData, PluginError,
    Scaling, StatKind, Targeting, TargetingType, TraitBreakpoint, TraitDef, TraitModifier, UnitDef,
    UnitStats,
};
pub use rng::Rng;

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
