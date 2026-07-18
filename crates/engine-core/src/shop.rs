//! Player economy surface and shop actions (buy / sell / reroll / buy XP).

use thiserror::Error;

use crate::economy::{level_from_xp, MAX_LEVEL};
use crate::ids::{DefId, UnitInstanceId};
use crate::pool::{UnitPool, SHOP_SIZE};
use crate::rng::Rng;

/// Bench slots per player (MVP parameter).
pub const BENCH_CAPACITY: usize = 9;

/// Gold cost of a shop reroll.
pub const REROLL_COST: i32 = 2;

/// Gold cost and XP gained when buying experience.
pub const BUY_XP_COST: i32 = 4;
pub const BUY_XP_AMOUNT: u32 = 4;

/// Starting gold at match open (MVP).
pub const STARTING_GOLD: i32 = 3;

/// One unit owned by a player (bench or later board).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwnedUnit {
    pub instance_id: UnitInstanceId,
    pub def_id: DefId,
    /// Star-1 purchase cost; sell refund equals this in MVP.
    pub cost: u8,
}

/// A single shop offer (already removed from the pool when drawn).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShopOffer {
    pub def_id: DefId,
    pub cost: u8,
}

/// Fixed-width shop row.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ShopState {
    pub slots: [Option<ShopOffer>; SHOP_SIZE],
}

impl ShopState {
    pub fn empty() -> Self {
        Self {
            slots: std::array::from_fn(|_| None),
        }
    }

    pub fn from_draws(pool: &UnitPool, draws: Vec<Option<DefId>>) -> Self {
        let mut slots = std::array::from_fn(|_| None);
        for (i, draw) in draws.into_iter().take(SHOP_SIZE).enumerate() {
            slots[i] = draw.map(|def_id| {
                let cost = pool.cost_of(def_id.as_str()).unwrap_or(1);
                ShopOffer { def_id, cost }
            });
        }
        Self { slots }
    }
}

/// Per-player gold / XP / level / bench inventory.
#[derive(Clone, Debug)]
pub struct PlayerEconomy {
    pub gold: i32,
    pub xp: u32,
    pub level: u8,
    pub bench: Vec<OwnedUnit>,
    next_instance_id: u32,
}

impl Default for PlayerEconomy {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerEconomy {
    pub fn new() -> Self {
        Self {
            gold: STARTING_GOLD,
            xp: 0,
            level: 1,
            bench: Vec::new(),
            next_instance_id: 1,
        }
    }

    pub fn with_gold(gold: i32) -> Self {
        Self {
            gold,
            ..Self::new()
        }
    }

    fn alloc_instance_id(&mut self) -> UnitInstanceId {
        let id = UnitInstanceId(self.next_instance_id);
        self.next_instance_id = self.next_instance_id.saturating_add(1);
        id
    }

    fn recompute_level(&mut self) {
        self.level = level_from_xp(self.xp);
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ShopError {
    #[error("not enough gold")]
    NotEnoughGold,
    #[error("bench is full")]
    BenchFull,
    #[error("shop slot is empty")]
    EmptySlot,
    #[error("invalid shop index")]
    InvalidIndex,
    #[error("unit not found on bench")]
    UnitNotFound,
    #[error("already at max level")]
    MaxLevel,
}

/// Fill shop from the pool (no gold cost). Returns previous offers to the pool first.
pub fn refresh_shop(
    shop: &mut ShopState,
    pool: &mut UnitPool,
    rng: &mut Rng,
    level: u8,
) {
    return_shop_to_pool(shop, pool);
    let draws = pool.draw_shop(rng, level, SHOP_SIZE);
    *shop = ShopState::from_draws(pool, draws);
}

fn return_shop_to_pool(shop: &mut ShopState, pool: &mut UnitPool) {
    for slot in shop.slots.iter_mut() {
        if let Some(offer) = slot.take() {
            pool.return_unit(offer.def_id.as_str());
        }
    }
}

/// Reroll shop for `REROLL_COST` gold.
pub fn reroll(
    player: &mut PlayerEconomy,
    shop: &mut ShopState,
    pool: &mut UnitPool,
    rng: &mut Rng,
) -> Result<(), ShopError> {
    if player.gold < REROLL_COST {
        return Err(ShopError::NotEnoughGold);
    }
    player.gold -= REROLL_COST;
    refresh_shop(shop, pool, rng, player.level);
    Ok(())
}

/// Buy unit at `shop_index` onto the bench if gold and space allow.
pub fn buy(
    player: &mut PlayerEconomy,
    shop: &mut ShopState,
    shop_index: usize,
) -> Result<UnitInstanceId, ShopError> {
    if shop_index >= SHOP_SIZE {
        return Err(ShopError::InvalidIndex);
    }
    let offer = shop.slots[shop_index]
        .as_ref()
        .ok_or(ShopError::EmptySlot)?
        .clone();
    if player.gold < offer.cost as i32 {
        return Err(ShopError::NotEnoughGold);
    }
    if player.bench.len() >= BENCH_CAPACITY {
        return Err(ShopError::BenchFull);
    }
    player.gold -= offer.cost as i32;
    shop.slots[shop_index] = None;
    let instance_id = player.alloc_instance_id();
    player.bench.push(OwnedUnit {
        instance_id,
        def_id: offer.def_id,
        cost: offer.cost,
    });
    Ok(instance_id)
}

/// Sell a star-1 unit from the bench; refund equals cost and returns one pool copy.
pub fn sell(
    player: &mut PlayerEconomy,
    pool: &mut UnitPool,
    unit_instance_id: UnitInstanceId,
) -> Result<(), ShopError> {
    let pos = player
        .bench
        .iter()
        .position(|u| u.instance_id == unit_instance_id)
        .ok_or(ShopError::UnitNotFound)?;
    let unit = player.bench.remove(pos);
    player.gold += unit.cost as i32;
    pool.return_unit(unit.def_id.as_str());
    Ok(())
}

/// Spend gold for XP and update level.
pub fn buy_xp(player: &mut PlayerEconomy) -> Result<(), ShopError> {
    if player.level >= MAX_LEVEL {
        return Err(ShopError::MaxLevel);
    }
    if player.gold < BUY_XP_COST {
        return Err(ShopError::NotEnoughGold);
    }
    player.gold -= BUY_XP_COST;
    player.xp = player.xp.saturating_add(BUY_XP_AMOUNT);
    player.recompute_level();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::DefId;
    use crate::plugin::{UnitDef, UnitStats};

    fn units() -> Vec<UnitDef> {
        vec![
            UnitDef {
                id: DefId::new("a"),
                name: "A".into(),
                cost: 1,
                traits: vec![],
                ability_id: None,
                stats: UnitStats {
                    hp: 1,
                    atk: 1,
                    armor: 0,
                    mr: 0,
                    range: 1,
                    attack_speed_milli: 500,
                },
            },
            UnitDef {
                id: DefId::new("b"),
                name: "B".into(),
                cost: 2,
                traits: vec![],
                ability_id: None,
                stats: UnitStats {
                    hp: 1,
                    atk: 1,
                    armor: 0,
                    mr: 0,
                    range: 1,
                    attack_speed_milli: 500,
                },
            },
        ]
    }

    #[test]
    fn buy_and_sell_roundtrip_gold_and_pool() {
        let mut pool = UnitPool::from_units(&units());
        let mut rng = Rng::new(7);
        let mut player = PlayerEconomy::with_gold(50);
        let mut shop = ShopState::empty();
        refresh_shop(&mut shop, &mut pool, &mut rng, 1);

        // Find a filled slot
        let idx = shop.slots.iter().position(|s| s.is_some()).unwrap();
        let cost = shop.slots[idx].as_ref().unwrap().cost;
        let def = shop.slots[idx].as_ref().unwrap().def_id.as_str().to_string();
        let pool_before_buy = pool.remaining(&def);
        let gold_before = player.gold;
        let id = buy(&mut player, &mut shop, idx).unwrap();
        assert_eq!(player.gold, gold_before - cost as i32);
        assert!(shop.slots[idx].is_none());
        assert_eq!(player.bench.len(), 1);
        // Buy does not touch pool further (already drawn).
        assert_eq!(pool.remaining(&def), pool_before_buy);

        sell(&mut player, &mut pool, id).unwrap();
        assert_eq!(player.gold, gold_before);
        assert!(player.bench.is_empty());
        // Sold unit returned to pool.
        assert_eq!(pool.remaining(&def), pool_before_buy + 1);
    }

    #[test]
    fn reroll_costs_two() {
        let mut pool = UnitPool::from_units(&units());
        let mut rng = Rng::new(3);
        let mut player = PlayerEconomy::with_gold(10);
        let mut shop = ShopState::empty();
        refresh_shop(&mut shop, &mut pool, &mut rng, 1);
        reroll(&mut player, &mut shop, &mut pool, &mut rng).unwrap();
        assert_eq!(player.gold, 8);
        assert!(reroll(&mut player, &mut shop, &mut pool, &mut rng).is_ok());
        assert_eq!(player.gold, 6);
    }

    #[test]
    fn buy_fails_when_broke_or_bench_full() {
        let mut pool = UnitPool::from_units(&units());
        let mut rng = Rng::new(9);
        let mut player = PlayerEconomy::with_gold(0);
        let mut shop = ShopState::empty();
        refresh_shop(&mut shop, &mut pool, &mut rng, 1);
        let idx = shop.slots.iter().position(|s| s.is_some()).unwrap();
        assert_eq!(buy(&mut player, &mut shop, idx), Err(ShopError::NotEnoughGold));

        player.gold = 100;
        for _ in 0..BENCH_CAPACITY {
            // force offers
            if shop.slots[idx].is_none() {
                refresh_shop(&mut shop, &mut pool, &mut rng, 1);
            }
            let i = shop.slots.iter().position(|s| s.is_some()).unwrap();
            buy(&mut player, &mut shop, i).unwrap();
            if shop.slots.iter().all(|s| s.is_none()) {
                refresh_shop(&mut shop, &mut pool, &mut rng, 1);
            }
        }
        if shop.slots[idx].is_none() {
            refresh_shop(&mut shop, &mut pool, &mut rng, 1);
        }
        let i = shop.slots.iter().position(|s| s.is_some()).unwrap();
        assert_eq!(buy(&mut player, &mut shop, i), Err(ShopError::BenchFull));
    }

    #[test]
    fn buy_xp_levels_up() {
        let mut player = PlayerEconomy::with_gold(20);
        assert_eq!(player.level, 1);
        buy_xp(&mut player).unwrap();
        assert_eq!(player.xp, 4);
        assert_eq!(player.level, 2);
        assert_eq!(player.gold, 16);
    }
}
