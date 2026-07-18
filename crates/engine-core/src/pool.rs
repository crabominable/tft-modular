//! Shared unit pool (bag of copies) and shop-slot draws.

use std::collections::BTreeMap;

use crate::ids::DefId;
use crate::plugin::{PluginData, UnitDef};
use crate::rng::Rng;

/// Copies placed in the shared pool per unit, indexed by cost (1..=5).
pub const COPIES_BY_COST: [u16; 6] = [0, 30, 25, 18, 10, 9];

/// Default shop width.
pub const SHOP_SIZE: usize = 5;

/// Cost-tier odds (c1..c5 percentages) for levels 1..=9.
/// Rows sum to 100.
const SHOP_ODDS: [[u8; 5]; 9] = [
    [100, 0, 0, 0, 0],  // 1
    [100, 0, 0, 0, 0],  // 2
    [75, 25, 0, 0, 0],  // 3
    [55, 30, 15, 0, 0], // 4
    [45, 33, 20, 2, 0], // 5
    [30, 40, 25, 5, 0], // 6
    [19, 30, 35, 15, 1],// 7
    [18, 25, 32, 22, 3],// 8
    [10, 20, 25, 35, 10],// 9
];

/// Shared bag of unit copies keyed by definition id.
#[derive(Clone, Debug, Default)]
pub struct UnitPool {
    /// Remaining copies per unit def id.
    counts: BTreeMap<String, u16>,
    /// Cost per unit def id.
    costs: BTreeMap<String, u8>,
    /// Def ids grouped by cost (stable order from plugin load).
    by_cost: [Vec<String>; 6],
}

impl UnitPool {
    /// Initialize bag counts from plugin unit defs × cost copy tables.
    pub fn new(plugin: &PluginData) -> Self {
        Self::from_units(&plugin.units)
    }

    /// Build from an arbitrary unit list (tests / custom packs).
    pub fn from_units(units: &[UnitDef]) -> Self {
        let mut pool = Self::default();
        for unit in units {
            let cost = unit.cost.clamp(1, 5);
            let copies = COPIES_BY_COST[cost as usize];
            let id = unit.id.as_str().to_string();
            pool.counts.insert(id.clone(), copies);
            pool.costs.insert(id.clone(), cost);
            pool.by_cost[cost as usize].push(id);
        }
        pool
    }

    pub fn remaining(&self, unit_id: &str) -> u16 {
        self.counts.get(unit_id).copied().unwrap_or(0)
    }

    pub fn cost_of(&self, unit_id: &str) -> Option<u8> {
        self.costs.get(unit_id).copied()
    }

    /// Remove one copy if available.
    pub fn take(&mut self, unit_id: &str) -> bool {
        let Some(count) = self.counts.get_mut(unit_id) else {
            return false;
        };
        if *count == 0 {
            return false;
        }
        *count -= 1;
        true
    }

    /// Return one copy (e.g. sell / shop refresh of unsold offers).
    pub fn return_unit(&mut self, unit_id: &str) {
        if let Some(count) = self.counts.get_mut(unit_id) {
            *count = count.saturating_add(1);
        }
    }

    /// Odds row for a player level (clamped 1..=9).
    pub fn odds_for_level(level: u8) -> [u8; 5] {
        let idx = level.clamp(1, 9) as usize - 1;
        SHOP_ODDS[idx]
    }

    /// Draw `shop_size` offers. Each successful draw removes one pool copy.
    /// Empty slots are returned when the pool cannot fill a roll.
    pub fn draw_shop(&mut self, rng: &mut Rng, level: u8, shop_size: usize) -> Vec<Option<DefId>> {
        let mut out = Vec::with_capacity(shop_size);
        for _ in 0..shop_size {
            out.push(self.draw_one(rng, level));
        }
        out
    }

    fn draw_one(&mut self, rng: &mut Rng, level: u8) -> Option<DefId> {
        let odds = Self::odds_for_level(level);
        let cost = self.roll_available_cost(rng, &odds)?;
        let candidates: Vec<&String> = self.by_cost[cost as usize]
            .iter()
            .filter(|id| self.remaining(id) > 0)
            .collect();
        if candidates.is_empty() {
            return None;
        }
        let pick = candidates[rng.gen_range_usize(candidates.len())];
        let id = pick.clone();
        self.take(&id);
        Some(DefId::new(id))
    }

    /// Weighted cost roll among tiers that still have at least one copy.
    fn roll_available_cost(&self, rng: &mut Rng, odds: &[u8; 5]) -> Option<u8> {
        let mut weights = [0u32; 5];
        let mut total = 0u32;
        for (i, &pct) in odds.iter().enumerate() {
            let cost = (i as u8) + 1;
            if pct == 0 {
                continue;
            }
            let available = self
                .by_cost
                .get(cost as usize)
                .map(|ids| ids.iter().any(|id| self.remaining(id) > 0))
                .unwrap_or(false);
            if available {
                weights[i] = pct as u32;
                total += pct as u32;
            }
        }
        if total == 0 {
            // Fallback: any cost with stock, uniform by unit count later via first available tier.
            for cost in 1u8..=5 {
                let available = self.by_cost[cost as usize]
                    .iter()
                    .any(|id| self.remaining(id) > 0);
                if available {
                    return Some(cost);
                }
            }
            return None;
        }
        let roll = rng.gen_range_usize(total as usize) as u32;
        let mut acc = 0u32;
        for (i, w) in weights.iter().enumerate() {
            if *w == 0 {
                continue;
            }
            acc += *w;
            if roll < acc {
                return Some((i as u8) + 1);
            }
        }
        // Should be unreachable if total > 0
        Some(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::{UnitDef, UnitStats};

    fn sample_units() -> Vec<UnitDef> {
        vec![
            unit("c1a", 1),
            unit("c1b", 1),
            unit("c2a", 2),
            unit("c3a", 3),
        ]
    }

    fn unit(id: &str, cost: u8) -> UnitDef {
        UnitDef {
            id: DefId::new(id),
            name: id.to_string(),
            cost,
            traits: vec![],
            ability_id: None,
            stats: UnitStats {
                hp: 100,
                atk: 10,
                armor: 0,
                mr: 0,
                range: 1,
                attack_speed_milli: 500,
            },
        }
    }

    #[test]
    fn pool_init_uses_cost_tables() {
        let pool = UnitPool::from_units(&sample_units());
        assert_eq!(pool.remaining("c1a"), 30);
        assert_eq!(pool.remaining("c2a"), 25);
        assert_eq!(pool.remaining("c3a"), 18);
        assert_eq!(pool.remaining("missing"), 0);
    }

    #[test]
    fn take_and_return() {
        let mut pool = UnitPool::from_units(&sample_units());
        assert!(pool.take("c1a"));
        assert_eq!(pool.remaining("c1a"), 29);
        pool.return_unit("c1a");
        assert_eq!(pool.remaining("c1a"), 30);
    }

    #[test]
    fn draw_shop_level1_is_cost1_and_removes_copies() {
        let mut pool = UnitPool::from_units(&sample_units());
        let mut rng = Rng::new(1);
        let shop = pool.draw_shop(&mut rng, 1, SHOP_SIZE);
        assert_eq!(shop.len(), 5);
        let mut removed = 0u16;
        for slot in &shop {
            let id = slot.as_ref().expect("filled");
            assert_eq!(pool.cost_of(id.as_str()), Some(1));
            removed += 1;
        }
        let left = pool.remaining("c1a") + pool.remaining("c1b");
        assert_eq!(left, 60 - removed);
    }

    #[test]
    fn odds_level_tables() {
        assert_eq!(UnitPool::odds_for_level(1), [100, 0, 0, 0, 0]);
        assert_eq!(UnitPool::odds_for_level(3), [75, 25, 0, 0, 0]);
        assert_eq!(UnitPool::odds_for_level(9), [10, 20, 25, 35, 10]);
    }
}
