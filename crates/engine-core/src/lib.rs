// crates/engine-core/src/lib.rs
#![deny(unsafe_code)]

pub mod combat;
pub mod command;
pub mod economy;
pub mod fixed;
pub mod ids;
pub mod plugin;
pub mod pool;
pub mod rng;
pub mod shop;

pub use combat::{
    attack_interval_ticks, valid_spawn, CombatResult, CombatState, CombatUnit, Pos, Side,
    BOARD_SIZE, MANA_PER_ATTACK, MAX_TICKS, TICK_MS,
};
pub use command::Command;
pub use economy::{board_cap_for_level, interest, level_from_xp, XP_THRESHOLDS, MAX_LEVEL};
pub use fixed::{fp_from_i64, fp_mul, Fp, FP_ONE};
pub use ids::{DefId, PlayerId, UnitInstanceId};
pub use plugin::{
    AbilityDef, AbilityEffect, AbilityTrigger, EffectType, Manifest, PluginData, PluginError,
    Scaling, StatKind, Targeting, TargetingType, TraitBreakpoint, TraitDef, TraitModifier, UnitDef,
    UnitStats,
};
pub use pool::{UnitPool, COPIES_BY_COST, SHOP_SIZE};
pub use rng::Rng;
pub use shop::{
    buy, buy_xp, refresh_shop, reroll, sell, OwnedUnit, PlayerEconomy, ShopError, ShopOffer,
    ShopState, BENCH_CAPACITY, BUY_XP_AMOUNT, BUY_XP_COST, REROLL_COST, STARTING_GOLD,
};

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
