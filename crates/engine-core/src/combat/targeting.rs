//! Deterministic targeting helpers for combat.

use super::{CombatUnit, Pos, Side};

/// Chebyshev distance (king-move). Range checks use this so range-1 is any adjacent cell including diagonal.
#[inline]
pub fn chebyshev(a: Pos, b: Pos) -> i32 {
    (a.x - b.x).abs().max((a.y - b.y).abs())
}

/// Manhattan distance (4-directional path length).
#[inline]
pub fn manhattan(a: Pos, b: Pos) -> i32 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}

/// True when `target` is within attack `range` of `attacker` (Chebyshev).
#[inline]
pub fn in_range(attacker: Pos, target: Pos, range: i32) -> bool {
    chebyshev(attacker, target) <= range
}

/// Living units on the opposite side of `side`.
pub fn enemies<'a>(units: &'a [CombatUnit], side: Side) -> impl Iterator<Item = &'a CombatUnit> {
    units
        .iter()
        .filter(move |u| u.is_alive() && u.side != side)
}

/// Nearest living enemy by Chebyshev distance; ties broken by smaller instance id.
pub fn nearest_enemy(units: &[CombatUnit], self_idx: usize) -> Option<usize> {
    let me = units.get(self_idx)?;
    if !me.is_alive() {
        return None;
    }
    let origin = me.pos();
    let side = me.side;
    let mut best: Option<(i32, u32, usize)> = None;
    for (i, u) in units.iter().enumerate() {
        if i == self_idx || !u.is_alive() || u.side == side {
            continue;
        }
        let d = chebyshev(origin, u.pos());
        let key = (d, u.id.0, i);
        match best {
            None => best = Some(key),
            Some(cur) if key < cur => best = Some(key),
            _ => {}
        }
    }
    best.map(|(_, _, i)| i)
}

/// All living enemy indices, stable id order.
pub fn all_enemies(units: &[CombatUnit], side: Side) -> Vec<usize> {
    let mut idxs: Vec<usize> = units
        .iter()
        .enumerate()
        .filter(|(_, u)| u.is_alive() && u.side != side)
        .map(|(i, _)| i)
        .collect();
    idxs.sort_by_key(|&i| units[i].id.0);
    idxs
}

/// Pick a random living enemy using the provided index in `0..count` (caller supplies RNG).
/// Returns the unit index among enemies sorted by instance id.
pub fn random_enemy_at(units: &[CombatUnit], side: Side, pick: usize) -> Option<usize> {
    let enemies = all_enemies(units, side);
    if enemies.is_empty() {
        return None;
    }
    Some(enemies[pick % enemies.len()])
}

/// One 4-directional step from `from` toward `to`.
///
/// Tie-break: when both axes need movement, prefer the axis with smaller absolute
/// delta; if equal, prefer x before y. Occupied cells (and out of bounds) are
/// skipped; if the preferred axis is blocked, try the other.
pub fn step_toward(
    from: Pos,
    to: Pos,
    occupied: &dyn Fn(Pos) -> bool,
    in_bounds: &dyn Fn(Pos) -> bool,
) -> Option<Pos> {
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    if dx == 0 && dy == 0 {
        return None;
    }

    let mut candidates: Vec<(i32, i32)> = Vec::with_capacity(2);
    // Prefer smaller |dx| axis first; x before y when equal.
    if dx != 0 && dy != 0 {
        if dx.abs() < dy.abs() {
            candidates.push((dx.signum(), 0));
            candidates.push((0, dy.signum()));
        } else if dy.abs() < dx.abs() {
            candidates.push((0, dy.signum()));
            candidates.push((dx.signum(), 0));
        } else {
            // equal |dx| == |dy|: x before y
            candidates.push((dx.signum(), 0));
            candidates.push((0, dy.signum()));
        }
    } else if dx != 0 {
        candidates.push((dx.signum(), 0));
    } else {
        candidates.push((0, dy.signum()));
    }

    for (sx, sy) in candidates {
        let next = Pos {
            x: from.x + sx,
            y: from.y + sy,
        };
        if in_bounds(next) && !occupied(next) {
            return Some(next);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{DefId, UnitInstanceId};

    fn dummy(id: u32, side: Side, x: i32, y: i32) -> CombatUnit {
        CombatUnit {
            id: UnitInstanceId(id),
            side,
            def_id: DefId::new("u"),
            x,
            y,
            hp: 100,
            max_hp: 100,
            mana: 0,
            shield: 0,
            shield_until_tick: 0,
            stun_until_tick: 0,
            atk: 10,
            base_atk: 10,
            armor: 0,
            base_armor: 0,
            range: 1,
            attack_speed_milli: 1000,
            next_attack_tick: 0,
            ability_id: None,
            stat_mods: Vec::new(),
            target_idx: None,
        }
    }

    #[test]
    fn nearest_picks_closer_then_lower_id() {
        let units = vec![
            dummy(1, Side::A, 0, 2),
            dummy(2, Side::B, 3, 0),
            dummy(3, Side::B, 0, 1),
            dummy(4, Side::B, 0, 1), // same cell distance as 3; higher id
        ];
        // From unit 0 at (0,2): unit 3 and 4 at dist 1, unit 2 at dist 3.
        assert_eq!(nearest_enemy(&units, 0), Some(2)); // id 3
    }

    #[test]
    fn step_prefers_x_when_equal_deltas() {
        let from = Pos { x: 0, y: 0 };
        let to = Pos { x: 2, y: 2 };
        let occ = |_p: Pos| false;
        let bounds = |p: Pos| p.x >= 0 && p.x < 4 && p.y >= 0 && p.y < 4;
        let next = step_toward(from, to, &occ, &bounds).unwrap();
        assert_eq!(next, Pos { x: 1, y: 0 });
    }
}
