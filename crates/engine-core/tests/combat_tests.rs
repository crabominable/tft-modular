//! Integration tests for minimal deterministic combat.

use engine_core::combat::{
    CombatResult, CombatState, CombatUnit, Side, MAX_TICKS,
};
use engine_core::ids::{DefId, UnitInstanceId};
use engine_core::plugin::{
    AbilityDef, AbilityEffect, AbilityTrigger, EffectType, Scaling, Targeting, TargetingType,
    UnitStats,
};

fn stats(hp: i32, atk: i32, armor: i32, range: i32, as_milli: i32) -> UnitStats {
    UnitStats {
        hp,
        atk,
        armor,
        mr: 0,
        range,
        attack_speed_milli: as_milli,
    }
}

#[test]
fn melee_kills_stationary_target() {
    // Two units facing: Side A at (1,2), Side B at (1,1) — adjacent, range 1.
    // No movement needed. After enough ticks, defender dies and Side A wins.
    let mut combat = CombatState::empty(42);

    combat.add_unit(CombatUnit::from_stats(
        UnitInstanceId(1),
        Side::A,
        DefId::new("attacker"),
        &stats(500, 80, 0, 1, 1000),
        None,
        1,
        2,
    ));
    combat.add_unit(CombatUnit::from_stats(
        UnitInstanceId(2),
        Side::B,
        DefId::new("defender"),
        &stats(120, 5, 0, 1, 1000),
        None,
        1,
        1,
    ));

    let result = combat.run_to_completion();
    assert_eq!(result, CombatResult::Winner(Side::A));

    let defender = combat
        .units
        .iter()
        .find(|u| u.id == UnitInstanceId(2))
        .expect("defender");
    assert!(defender.hp <= 0, "defender should be dead, hp={}", defender.hp);

    let attacker = combat
        .units
        .iter()
        .find(|u| u.id == UnitInstanceId(1))
        .expect("attacker");
    assert!(attacker.hp > 0, "attacker should live");
}

#[test]
fn combat_respects_max_ticks_draw() {
    // Extremely tanky mutual stalemate with zero damage potential via huge armor
    // is hard with min-1 damage; use max ticks by placing out of range forever
    // with no path? On open board they walk. Instead: both unkillable via shield
    // churn is complex — use two units that never meet if we... they always path.
    // Simplest: very high HP and check tick advances; use short custom run count.
    let mut combat = CombatState::empty(1);
    combat.add_unit(CombatUnit::from_stats(
        UnitInstanceId(1),
        Side::A,
        DefId::new("a"),
        &stats(1_000_000, 1, 0, 1, 100), // slow AS, tiny damage
        None,
        0,
        3,
    ));
    combat.add_unit(CombatUnit::from_stats(
        UnitInstanceId(2),
        Side::B,
        DefId::new("b"),
        &stats(1_000_000, 1, 0, 1, 100),
        None,
        3,
        0,
    ));

    let result = combat.run_to_completion();
    // With 1 dmg and 1e6 hp they will not finish within 600 ticks → Draw.
    assert_eq!(result, CombatResult::Draw);
    assert!(combat.tick >= MAX_TICKS);
}

#[test]
fn on_cast_ability_deals_bonus_damage() {
    let spark = AbilityDef {
        id: DefId::new("spark_cut"),
        name: "Spark Cut".into(),
        mana_cost: 20, // 2 attacks at +10 mana
        trigger: AbilityTrigger::OnCast,
        targeting: Targeting {
            kind: TargetingType::NearestEnemy,
            count: Some(1),
        },
        effects: vec![AbilityEffect {
            kind: EffectType::Damage,
            amount: Some(100),
            duration_ms: None,
            stat: None,
            scaling: Some(Scaling::Atk),
        }],
    };

    let mut combat = CombatState::new([spark], 7);
    // atk 40 → ability damage 100 + 20 = 120; basic hits also apply.
    combat.add_unit(CombatUnit::from_stats(
        UnitInstanceId(1),
        Side::A,
        DefId::new("caster"),
        &stats(500, 40, 0, 1, 1000),
        Some(DefId::new("spark_cut")),
        1,
        2,
    ));
    combat.add_unit(CombatUnit::from_stats(
        UnitInstanceId(2),
        Side::B,
        DefId::new("dummy"),
        &stats(200, 0, 0, 1, 1000),
        None,
        1,
        1,
    ));

    let result = combat.run_to_completion();
    assert_eq!(result, CombatResult::Winner(Side::A));
    let dummy = combat.units.iter().find(|u| u.id == UnitInstanceId(2)).unwrap();
    assert!(dummy.hp <= 0);
}

#[test]
fn heal_ability_restores_hp() {
    let balm = AbilityDef {
        id: DefId::new("mist_balm"),
        name: "Mist Balm".into(),
        mana_cost: 10,
        trigger: AbilityTrigger::OnCast,
        targeting: Targeting {
            kind: TargetingType::SelfTarget,
            count: None,
        },
        effects: vec![AbilityEffect {
            kind: EffectType::Heal,
            amount: Some(50),
            duration_ms: None,
            stat: None,
            scaling: Some(Scaling::None),
        }],
    };

    let mut combat = CombatState::new([balm], 3);
    let mut healer = CombatUnit::from_stats(
        UnitInstanceId(1),
        Side::A,
        DefId::new("healer"),
        &stats(100, 30, 0, 1, 1000),
        Some(DefId::new("mist_balm")),
        1,
        2,
    );
    healer.hp = 40;
    combat.add_unit(healer);
    combat.add_unit(CombatUnit::from_stats(
        UnitInstanceId(2),
        Side::B,
        DefId::new("dummy"),
        &stats(500, 0, 0, 1, 1000),
        None,
        1,
        1,
    ));

    // One attack → mana 10 → cast heal → hp min(100, 40+50)=90.
    // Enemy still swings for min 1 damage after, so expect > starting 40.
    combat.tick_once();
    let healer = combat.units.iter().find(|u| u.id == UnitInstanceId(1)).unwrap();
    assert!(
        healer.hp > 40 && healer.hp <= 100,
        "expected heal after cast, hp={}",
        healer.hp
    );
    assert_eq!(healer.mana, 0, "mana spent on cast");
}
