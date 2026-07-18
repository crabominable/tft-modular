//! Deterministic combat simulation (fixed 50 ms ticks, 4×4 board).

pub mod effects;
pub mod targeting;

use std::collections::HashMap;

use crate::ids::{DefId, UnitInstanceId};
use crate::plugin::{
    AbilityDef, AbilityTrigger, TargetingType, UnitDef, UnitStats,
};
use crate::rng::Rng;

use effects::{
    apply_damage, apply_effect, basic_attack_damage, tick_status_expiry,
};
use targeting::{in_range, nearest_enemy, random_enemy_at, step_toward};

/// Simulation timestep in milliseconds.
pub const TICK_MS: i32 = 50;
/// Hard cap on combat length (30 s at 50 ms).
pub const MAX_TICKS: u32 = 600;
/// Board width / height (cells).
pub const BOARD_SIZE: i32 = 4;
/// Mana gained on each basic attack.
pub const MANA_PER_ATTACK: i32 = 10;

/// Player side occupies rows `y = 2..3`; enemy side occupies `y = 0..1`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    /// Player / Side A — rows y=2..=3.
    A,
    /// Enemy / Side B — rows y=0..=1.
    B,
}

impl Side {
    pub fn opposite(self) -> Side {
        match self {
            Side::A => Side::B,
            Side::B => Side::A,
        }
    }
}

/// Outcome of a finished combat.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CombatResult {
    Winner(Side),
    Draw,
}

/// Grid cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

/// Temporary stat modifier with expiry tick.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatModActive {
    pub stat: crate::plugin::StatKind,
    pub amount: i32,
    pub until_tick: u32,
}

/// One living (or dead) combatant on the board.
#[derive(Clone, Debug)]
pub struct CombatUnit {
    pub id: UnitInstanceId,
    pub side: Side,
    pub def_id: DefId,
    pub x: i32,
    pub y: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub mana: i32,
    pub shield: i32,
    pub shield_until_tick: u32,
    pub stun_until_tick: u32,
    pub atk: i32,
    pub base_atk: i32,
    pub armor: i32,
    pub base_armor: i32,
    pub range: i32,
    pub attack_speed_milli: i32,
    /// Next tick at which this unit may basic-attack.
    pub next_attack_tick: u32,
    pub ability_id: Option<DefId>,
    pub stat_mods: Vec<StatModActive>,
    /// Cached target index into `CombatState::units` (invalidated when dead/missing).
    pub target_idx: Option<usize>,
}

impl CombatUnit {
    pub fn pos(&self) -> Pos {
        Pos {
            x: self.x,
            y: self.y,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }

    pub fn is_stunned(&self, tick: u32) -> bool {
        tick < self.stun_until_tick
    }

    /// Build a combat unit from a unit definition and placement.
    pub fn from_def(
        id: UnitInstanceId,
        side: Side,
        def: &UnitDef,
        x: i32,
        y: i32,
    ) -> Self {
        Self::from_stats(
            id,
            side,
            def.id.clone(),
            &def.stats,
            def.ability_id.clone(),
            x,
            y,
        )
    }

    pub fn from_stats(
        id: UnitInstanceId,
        side: Side,
        def_id: DefId,
        stats: &UnitStats,
        ability_id: Option<DefId>,
        x: i32,
        y: i32,
    ) -> Self {
        Self {
            id,
            side,
            def_id,
            x,
            y,
            hp: stats.hp,
            max_hp: stats.hp,
            mana: 0,
            shield: 0,
            shield_until_tick: 0,
            stun_until_tick: 0,
            atk: stats.atk,
            base_atk: stats.atk,
            armor: stats.armor,
            base_armor: stats.armor,
            range: stats.range,
            attack_speed_milli: stats.attack_speed_milli,
            next_attack_tick: 0,
            ability_id,
            stat_mods: Vec::new(),
            target_idx: None,
        }
    }
}

/// Full combat arena state.
#[derive(Clone, Debug)]
pub struct CombatState {
    pub tick: u32,
    pub units: Vec<CombatUnit>,
    abilities: HashMap<String, AbilityDef>,
    rng: Rng,
    finished: Option<CombatResult>,
}

impl CombatState {
    /// Create an empty combat with ability catalog and seed for `RANDOM_ENEMY`.
    pub fn new(abilities: impl IntoIterator<Item = AbilityDef>, seed: u64) -> Self {
        let mut map = HashMap::new();
        for ab in abilities {
            map.insert(ab.id.as_str().to_string(), ab);
        }
        Self {
            tick: 0,
            units: Vec::new(),
            abilities: map,
            rng: Rng::new(seed),
            finished: None,
        }
    }

    /// Empty combat with no ability defs.
    pub fn empty(seed: u64) -> Self {
        Self::new(std::iter::empty(), seed)
    }

    pub fn add_unit(&mut self, unit: CombatUnit) {
        self.units.push(unit);
    }

    pub fn result(&self) -> Option<CombatResult> {
        self.finished
    }

    /// Living units on a side.
    pub fn living_on(&self, side: Side) -> impl Iterator<Item = &CombatUnit> {
        self.units
            .iter()
            .filter(move |u| u.is_alive() && u.side == side)
    }

    /// Run until a side is wiped or `MAX_TICKS` is reached.
    pub fn run_to_completion(&mut self) -> CombatResult {
        while self.finished.is_none() {
            self.tick_once();
        }
        self.finished.unwrap_or(CombatResult::Draw)
    }

    /// Advance one simulation tick. Returns `Some(result)` when combat ends.
    pub fn tick_once(&mut self) -> Option<CombatResult> {
        if let Some(r) = self.finished {
            return Some(r);
        }

        // 1. Decrement / expire timers for all units.
        let tick = self.tick;
        for u in &mut self.units {
            tick_status_expiry(u, tick);
        }

        // Stable processing order: sort indices by instance id.
        let mut order: Vec<usize> = (0..self.units.len()).collect();
        order.sort_by_key(|&i| self.units[i].id.0);

        // 2. Act with each living unit.
        for idx in order {
            if !self.units[idx].is_alive() {
                continue;
            }
            if self.units[idx].is_stunned(self.tick) {
                continue;
            }
            self.unit_act(idx);
        }

        self.tick = self.tick.saturating_add(1);

        if let Some(r) = self.check_end() {
            self.finished = Some(r);
            return Some(r);
        }
        None
    }

    fn check_end(&self) -> Option<CombatResult> {
        let a_alive = self.units.iter().any(|u| u.is_alive() && u.side == Side::A);
        let b_alive = self.units.iter().any(|u| u.is_alive() && u.side == Side::B);
        match (a_alive, b_alive) {
            (true, false) => Some(CombatResult::Winner(Side::A)),
            (false, true) => Some(CombatResult::Winner(Side::B)),
            (false, false) => Some(CombatResult::Draw),
            (true, true) if self.tick >= MAX_TICKS => Some(CombatResult::Draw),
            _ => None,
        }
    }

    fn unit_act(&mut self, idx: usize) {
        // Acquire / refresh target.
        let target = self.resolve_target(idx);
        let Some(tidx) = target else {
            self.units[idx].target_idx = None;
            return;
        };
        self.units[idx].target_idx = Some(tidx);

        let pos = self.units[idx].pos();
        let tpos = self.units[tidx].pos();
        let range = self.units[idx].range;

        if in_range(pos, tpos, range) {
            self.try_attack_and_cast(idx, tidx);
        } else {
            self.try_move_toward(idx, tpos);
        }
    }

    fn resolve_target(&self, idx: usize) -> Option<usize> {
        if let Some(t) = self.units[idx].target_idx {
            if t < self.units.len()
                && self.units[t].is_alive()
                && self.units[t].side != self.units[idx].side
            {
                return Some(t);
            }
        }
        nearest_enemy(&self.units, idx)
    }

    fn try_attack_and_cast(&mut self, idx: usize, tidx: usize) {
        let tick = self.tick;
        if tick < self.units[idx].next_attack_tick {
            return;
        }

        // Basic attack.
        let atk = self.units[idx].atk;
        let armor = self.units[tidx].armor;
        let dmg = basic_attack_damage(atk, armor);
        apply_damage(&mut self.units[tidx], dmg);

        // Attack cooldown from attack_speed_milli (APS milli / 1000).
        let interval = attack_interval_ticks(self.units[idx].attack_speed_milli);
        self.units[idx].next_attack_tick = tick.saturating_add(interval);

        // Mana on attack.
        self.units[idx].mana = self.units[idx].mana.saturating_add(MANA_PER_ATTACK);

        // Cast if ready (ON_CAST only for MVP auto-fire).
        self.try_cast(idx);
    }

    fn try_cast(&mut self, caster_idx: usize) {
        let ability_id = match self.units[caster_idx].ability_id.clone() {
            Some(id) => id,
            None => return,
        };
        let ab = match self.abilities.get(ability_id.as_str()) {
            Some(a) => a.clone(),
            None => return,
        };
        if ab.trigger != AbilityTrigger::OnCast {
            return;
        }
        if self.units[caster_idx].mana < ab.mana_cost {
            return;
        }

        // Spend mana.
        self.units[caster_idx].mana = 0;

        // Resolve targets for ability.
        let targets = self.ability_targets(caster_idx, &ab);
        let tick = self.tick;
        // Snapshot caster for scaling (stats at cast time).
        let caster_snap = self.units[caster_idx].clone();

        for tidx in targets {
            if tidx >= self.units.len() {
                continue;
            }
            // Self-target or still valid.
            for effect in &ab.effects {
                // Re-borrow target mutably per effect.
                // Caster may be the same as target (SELF).
                if tidx == caster_idx {
                    apply_effect(effect, &caster_snap, &mut self.units[caster_idx], tick);
                } else {
                    apply_effect(effect, &caster_snap, &mut self.units[tidx], tick);
                }
            }
        }
    }

    fn ability_targets(&mut self, caster_idx: usize, ab: &AbilityDef) -> Vec<usize> {
        let side = self.units[caster_idx].side;
        match ab.targeting.kind {
            TargetingType::SelfTarget => vec![caster_idx],
            TargetingType::NearestEnemy => nearest_enemy(&self.units, caster_idx)
                .into_iter()
                .collect(),
            TargetingType::RandomEnemy => {
                let pick = self.rng.gen_range_usize(usize::MAX);
                random_enemy_at(&self.units, side, pick)
                    .into_iter()
                    .collect()
            }
            TargetingType::AllEnemies => {
                let mut idxs: Vec<usize> = self
                    .units
                    .iter()
                    .enumerate()
                    .filter(|(_, u)| u.is_alive() && u.side != side)
                    .map(|(i, _)| i)
                    .collect();
                idxs.sort_by_key(|&i| self.units[i].id.0);
                if let Some(count) = ab.targeting.count {
                    idxs.truncate(count as usize);
                }
                idxs
            }
        }
    }

    fn try_move_toward(&mut self, idx: usize, target_pos: Pos) {
        let from = self.units[idx].pos();
        // Occupancy: any living unit's cell (except self).
        let units = &self.units;
        let occupied = |p: Pos| {
            units.iter().enumerate().any(|(i, u)| {
                i != idx && u.is_alive() && u.x == p.x && u.y == p.y
            })
        };
        let in_bounds = |p: Pos| p.x >= 0 && p.x < BOARD_SIZE && p.y >= 0 && p.y < BOARD_SIZE;

        if let Some(next) = step_toward(from, target_pos, &occupied, &in_bounds) {
            self.units[idx].x = next.x;
            self.units[idx].y = next.y;
        }
    }
}

/// Convert attack speed (milli-APS, 1000 = 1.0/s) to ticks between attacks.
pub fn attack_interval_ticks(attack_speed_milli: i32) -> u32 {
    let as_m = attack_speed_milli.max(100) as i64;
    // period_ms = 1000 / (as_m/1000) = 1_000_000 / as_m
    let period_ms = 1_000_000 / as_m;
    let ticks = period_ms / (TICK_MS as i64);
    ticks.max(1) as u32
}

/// Validate placement: player y in 2..3, enemy y in 0..1, x in 0..3.
pub fn valid_spawn(side: Side, x: i32, y: i32) -> bool {
    if x < 0 || x >= BOARD_SIZE || y < 0 || y >= BOARD_SIZE {
        return false;
    }
    match side {
        Side::A => y >= 2,
        Side::B => y <= 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attack_interval_at_1aps_is_20_ticks() {
        // 1000 milli APS → 1000 ms period → 20 ticks of 50 ms
        assert_eq!(attack_interval_ticks(1000), 20);
    }

    #[test]
    fn spawn_rows_match_sides() {
        assert!(valid_spawn(Side::A, 0, 2));
        assert!(valid_spawn(Side::A, 3, 3));
        assert!(!valid_spawn(Side::A, 0, 1));
        assert!(valid_spawn(Side::B, 1, 0));
        assert!(!valid_spawn(Side::B, 1, 2));
    }
}
