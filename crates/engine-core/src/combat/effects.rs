//! Data-only ability effect application (closed allowlist).

use crate::plugin::{AbilityEffect, EffectType, Scaling, StatKind};

use super::{CombatUnit, StatModActive, TICK_MS};

/// Compute effect amount with optional ATK scaling: `amount + atk/2` when `Scaling::Atk`.
pub fn scaled_amount(effect: &AbilityEffect, caster_atk: i32) -> i32 {
    let base = effect.amount.unwrap_or(0);
    match effect.scaling.unwrap_or(Scaling::None) {
        Scaling::None | Scaling::Ap => base,
        Scaling::Atk => base.saturating_add(caster_atk / 2),
    }
}

/// Duration in ticks from `duration_ms` (50 ms timestep).
#[inline]
pub fn duration_ticks(duration_ms: Option<i32>) -> u32 {
    let ms = duration_ms.unwrap_or(0).max(0) as u32;
    ms / (TICK_MS as u32)
}

/// Apply physical/ability damage: shield first, then HP. Returns damage dealt to HP.
pub fn apply_damage(target: &mut CombatUnit, raw: i32) -> i32 {
    let mut remaining = raw.max(0);
    if remaining == 0 {
        return 0;
    }
    if target.shield > 0 {
        let absorbed = remaining.min(target.shield);
        target.shield -= absorbed;
        remaining -= absorbed;
    }
    if remaining > 0 {
        target.hp = (target.hp - remaining).max(0);
    }
    remaining
}

/// Armor-mitigated basic attack damage: `max(1, atk * 100 / (100 + armor))`.
pub fn basic_attack_damage(atk: i32, armor: i32) -> i32 {
    let armor = armor.max(0);
    let atk = atk.max(0);
    (atk * 100 / (100 + armor)).max(1)
}

/// Apply a single allowlisted effect from `caster` onto `target` at `tick`.
pub fn apply_effect(effect: &AbilityEffect, caster: &CombatUnit, target: &mut CombatUnit, tick: u32) {
    match effect.kind {
        EffectType::Damage => {
            let amount = scaled_amount(effect, caster.atk);
            apply_damage(target, amount);
        }
        EffectType::Heal => {
            let amount = scaled_amount(effect, caster.atk);
            target.hp = (target.hp + amount).min(target.max_hp);
        }
        EffectType::Shield => {
            let amount = effect.amount.unwrap_or(0).max(0);
            target.shield = target.shield.saturating_add(amount);
            let dur = duration_ticks(effect.duration_ms);
            if dur > 0 {
                target.shield_until_tick = tick.saturating_add(dur);
            }
        }
        EffectType::Stun => {
            let dur = duration_ticks(effect.duration_ms);
            if dur > 0 {
                let until = tick.saturating_add(dur);
                if until > target.stun_until_tick {
                    target.stun_until_tick = until;
                }
            }
        }
        EffectType::StatMod => {
            let amount = effect.amount.unwrap_or(0);
            let stat = effect.stat.unwrap_or(StatKind::Atk);
            let dur = duration_ticks(effect.duration_ms);
            let until = if dur > 0 {
                tick.saturating_add(dur)
            } else {
                // Permanent for the rest of combat when duration omitted.
                u32::MAX
            };
            // Apply immediately to current stats.
            match stat {
                StatKind::Atk => target.atk = target.atk.saturating_add(amount),
                StatKind::Armor => target.armor = target.armor.saturating_add(amount),
                StatKind::Hp => {
                    target.max_hp = target.max_hp.saturating_add(amount);
                    target.hp = target.hp.saturating_add(amount);
                }
                StatKind::Mr => {
                    // MR unused in damage formula for MVP; still track as stat mod.
                }
            }
            target.stat_mods.push(StatModActive {
                stat,
                amount,
                until_tick: until,
            });
        }
    }
}

/// Expire timed shields and stat mods; recompute atk/armor from base + active mods.
pub fn tick_status_expiry(unit: &mut CombatUnit, tick: u32) {
    if unit.shield > 0 && unit.shield_until_tick > 0 && tick >= unit.shield_until_tick {
        unit.shield = 0;
        unit.shield_until_tick = 0;
    }

    let before = unit.stat_mods.len();
    unit.stat_mods.retain(|m| tick < m.until_tick);
    if unit.stat_mods.len() != before {
        recompute_stats(unit);
    }
}

fn recompute_stats(unit: &mut CombatUnit) {
    let mut atk = unit.base_atk;
    let mut armor = unit.base_armor;
    for m in &unit.stat_mods {
        match m.stat {
            StatKind::Atk => atk = atk.saturating_add(m.amount),
            StatKind::Armor => armor = armor.saturating_add(m.amount),
            StatKind::Hp | StatKind::Mr => {}
        }
    }
    unit.atk = atk;
    unit.armor = armor;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{DefId, UnitInstanceId};
    use crate::plugin::AbilityEffect;
    use super::super::Side;

    fn unit(hp: i32, atk: i32, armor: i32) -> CombatUnit {
        CombatUnit {
            id: UnitInstanceId(1),
            side: Side::A,
            def_id: DefId::new("u"),
            x: 0,
            y: 2,
            hp,
            max_hp: hp,
            mana: 0,
            shield: 0,
            shield_until_tick: 0,
            stun_until_tick: 0,
            atk,
            base_atk: atk,
            armor,
            base_armor: armor,
            range: 1,
            attack_speed_milli: 1000,
            next_attack_tick: 0,
            ability_id: None,
            stat_mods: Vec::new(),
            target_idx: None,
        }
    }

    #[test]
    fn basic_attack_formula() {
        assert_eq!(basic_attack_damage(100, 0), 100);
        assert_eq!(basic_attack_damage(100, 100), 50);
        assert_eq!(basic_attack_damage(1, 1000), 1);
    }

    #[test]
    fn atk_scaling_adds_half_atk() {
        let e = AbilityEffect {
            kind: EffectType::Damage,
            amount: Some(120),
            duration_ms: None,
            stat: None,
            scaling: Some(Scaling::Atk),
        };
        assert_eq!(scaled_amount(&e, 40), 140);
    }

    #[test]
    fn damage_hits_shield_then_hp() {
        let mut t = unit(100, 10, 0);
        t.shield = 30;
        apply_damage(&mut t, 50);
        assert_eq!(t.shield, 0);
        assert_eq!(t.hp, 80);
    }
}
