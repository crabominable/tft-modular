//! Match state machine: shop → combat → resolve, human + AI seats.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::combat::{
    valid_spawn, CombatResult, CombatState, CombatUnit, Side, BOARD_SIZE,
};
use crate::command::Command;
use crate::economy::{board_cap_for_level, interest};
use crate::hash_state::StateHasher;
use crate::ids::UnitInstanceId;
use crate::plugin::{PluginData, UnitDef};
use crate::pool::UnitPool;
use crate::rng::Rng;
use crate::shop::{
    buy, buy_xp, refresh_shop, reroll, sell, OwnedUnit, PlayerEconomy, ShopError, ShopState,
    BUY_XP_COST, REROLL_COST,
};

/// Starting player HP (MVP).
pub const STARTING_HP: i32 = 20;
/// Base gold granted at the start of each shop phase after round 1.
pub const ROUND_INCOME_BASE: i32 = 5;
/// Flat component of player damage on combat loss.
pub const LOSS_DAMAGE_BASE: i32 = 2;
/// Local board width (matches combat columns).
pub const LOCAL_BOARD_W: u8 = 4;
/// Local board depth (2 rows per side).
pub const LOCAL_BOARD_H: u8 = 2;

/// Match phase.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Shop,
    Combat,
    MatchEnd,
}

/// Observable events emitted by `Match::apply` (and internal AI resolution).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    PhaseChanged {
        phase: Phase,
    },
    UnitBought {
        player_id: u8,
        instance_id: u32,
        def_id: String,
        cost: u8,
    },
    UnitSold {
        player_id: u8,
        instance_id: u32,
    },
    UnitPlaced {
        player_id: u8,
        instance_id: u32,
        cell: (u8, u8),
    },
    Rerolled {
        player_id: u8,
    },
    BoughtExp {
        player_id: u8,
        level: u8,
        xp: u32,
    },
    ShopEnded {
        player_id: u8,
    },
    CombatResolved {
        winner_player: Option<u8>,
        living_a: u8,
        living_b: u8,
    },
    PlayerDamaged {
        player_id: u8,
        amount: i32,
        hp_after: i32,
    },
    RoundAdvanced {
        round: u32,
    },
    MatchEnded {
        winner_player: Option<u8>,
    },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CoreError {
    #[error("wrong phase for command")]
    WrongPhase,
    #[error("invalid player id")]
    InvalidPlayer,
    #[error("match already over")]
    MatchOver,
    #[error("unit not found")]
    UnitNotFound,
    #[error("invalid board cell")]
    InvalidCell,
    #[error("cell occupied")]
    CellOccupied,
    #[error("board is full")]
    BoardFull,
    #[error("shop error: {0}")]
    Shop(#[from] ShopError),
}

/// Unit on a player's board (local half-grid cell).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoardUnit {
    pub instance_id: u32,
    pub def_id: String,
    pub cost: u8,
    /// Local cell `(x, y)` with `x < 4`, `y < 2`. `y=0` is the front row.
    pub cell: (u8, u8),
}

/// Serializable player view for wasm / UI.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerSnapshot {
    pub player_id: u8,
    pub hp: i32,
    pub gold: i32,
    pub xp: u32,
    pub level: u8,
    pub shop_ready: bool,
    pub shop: Vec<Option<ShopOfferSnap>>,
    pub bench: Vec<OwnedUnitSnap>,
    pub board: Vec<BoardUnit>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShopOfferSnap {
    pub def_id: String,
    pub cost: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnedUnitSnap {
    pub instance_id: u32,
    pub def_id: String,
    pub cost: u8,
}

/// Full match snapshot (Task 9 bridge).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchSnapshot {
    pub phase: Phase,
    pub round: u32,
    pub players: Vec<PlayerSnapshot>,
    pub state_hash: u64,
    pub winner_player: Option<u8>,
}

#[derive(Clone, Debug)]
struct PlayerSeat {
    economy: PlayerEconomy,
    shop: ShopState,
    board: Vec<BoardUnit>,
    hp: i32,
    shop_ready: bool,
}

/// Full match simulation (1 human seat 0 + 1 AI seat 1).
pub struct Match {
    plugin: PluginData,
    unit_index: BTreeMap<String, UnitDef>,
    rng: Rng,
    seed: u64,
    phase: Phase,
    round: u32,
    pool: UnitPool,
    players: [PlayerSeat; 2],
    winner_player: Option<u8>,
}

impl Match {
    pub fn new(plugin: PluginData, seed: u64) -> Self {
        let mut unit_index = BTreeMap::new();
        for u in &plugin.units {
            unit_index.insert(u.id.as_str().to_string(), u.clone());
        }
        let pool = UnitPool::new(&plugin);
        let mut m = Self {
            plugin,
            unit_index,
            rng: Rng::new(seed),
            seed,
            phase: Phase::Shop,
            round: 1,
            pool,
            players: [
                PlayerSeat {
                    economy: PlayerEconomy::new(),
                    shop: ShopState::empty(),
                    board: Vec::new(),
                    hp: STARTING_HP,
                    shop_ready: false,
                },
                PlayerSeat {
                    economy: PlayerEconomy::new(),
                    shop: ShopState::empty(),
                    board: Vec::new(),
                    hp: STARTING_HP,
                    shop_ready: false,
                },
            ],
            winner_player: None,
        };
        m.open_shop_phase(false);
        m
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn round(&self) -> u32 {
        self.round
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn winner(&self) -> Option<u8> {
        self.winner_player
    }

    pub fn player_hp(&self, player_id: u8) -> Option<i32> {
        self.players.get(player_id as usize).map(|p| p.hp)
    }

    /// Apply a command from `player_id`. Only the human seat (0) may call this
    /// for MVP; the AI seat is driven internally.
    pub fn apply(&mut self, player_id: u8, cmd: Command) -> Result<Vec<Event>, CoreError> {
        if self.phase == Phase::MatchEnd {
            return Err(CoreError::MatchOver);
        }
        if player_id != 0 {
            return Err(CoreError::InvalidPlayer);
        }
        if self.phase != Phase::Shop {
            return Err(CoreError::WrongPhase);
        }
        if self.players[0].shop_ready {
            return Err(CoreError::WrongPhase);
        }

        let mut events = Vec::new();
        match cmd {
            Command::BuyUnit { shop_index } => {
                let id = buy(
                    &mut self.players[0].economy,
                    &mut self.players[0].shop,
                    shop_index as usize,
                )?;
                let unit = self.players[0]
                    .economy
                    .bench
                    .iter()
                    .find(|u| u.instance_id == id)
                    .expect("just bought");
                events.push(Event::UnitBought {
                    player_id: 0,
                    instance_id: id.0,
                    def_id: unit.def_id.as_str().to_string(),
                    cost: unit.cost,
                });
            }
            Command::SellUnit { unit_instance_id } => {
                self.sell_unit(0, UnitInstanceId(unit_instance_id))?;
                events.push(Event::UnitSold {
                    player_id: 0,
                    instance_id: unit_instance_id,
                });
            }
            Command::Reroll => {
                reroll(
                    &mut self.players[0].economy,
                    &mut self.players[0].shop,
                    &mut self.pool,
                    &mut self.rng,
                )?;
                events.push(Event::Rerolled { player_id: 0 });
            }
            Command::BuyExp => {
                buy_xp(&mut self.players[0].economy)?;
                events.push(Event::BoughtExp {
                    player_id: 0,
                    level: self.players[0].economy.level,
                    xp: self.players[0].economy.xp,
                });
            }
            Command::PlaceUnit {
                unit_instance_id,
                cell,
            } => {
                self.place_unit(0, UnitInstanceId(unit_instance_id), cell)?;
                events.push(Event::UnitPlaced {
                    player_id: 0,
                    instance_id: unit_instance_id,
                    cell,
                });
            }
            Command::EndShopPhase => {
                self.players[0].shop_ready = true;
                events.push(Event::ShopEnded { player_id: 0 });
                // AI already finished at shop open; if not, run now.
                if !self.players[1].shop_ready {
                    self.run_ai_shop(&mut events);
                }
                if self.players[0].shop_ready && self.players[1].shop_ready {
                    self.resolve_combat_and_advance(&mut events)?;
                }
            }
        }
        Ok(events)
    }

    pub fn state_hash(&self) -> u64 {
        let mut h = StateHasher::new();
        h.write_u8(match self.phase {
            Phase::Shop => 1,
            Phase::Combat => 2,
            Phase::MatchEnd => 3,
        });
        h.write_u32(self.round);
        h.write_u8(self.winner_player.unwrap_or(255));
        for (pi, p) in self.players.iter().enumerate() {
            h.write_u8(pi as u8);
            h.write_i32(p.hp);
            h.write_i32(p.economy.gold);
            h.write_u32(p.economy.xp);
            h.write_u8(p.economy.level);
            h.write_u8(if p.shop_ready { 1 } else { 0 });
            // Shop slots left-to-right.
            for slot in &p.shop.slots {
                match slot {
                    Some(o) => {
                        h.write_u8(1);
                        h.write_str(o.def_id.as_str());
                        h.write_u8(o.cost);
                    }
                    None => h.write_u8(0),
                }
            }
            // Bench ordered by instance id for stability.
            let mut bench: Vec<&OwnedUnit> = p.economy.bench.iter().collect();
            bench.sort_by_key(|u| u.instance_id.0);
            h.write_u32(bench.len() as u32);
            for u in bench {
                h.write_u32(u.instance_id.0);
                h.write_str(u.def_id.as_str());
                h.write_u8(u.cost);
            }
            // Board sorted by cell then id.
            let mut board = p.board.clone();
            board.sort_by_key(|b| (b.cell.1, b.cell.0, b.instance_id));
            h.write_u32(board.len() as u32);
            for b in board {
                h.write_u32(b.instance_id);
                h.write_str(&b.def_id);
                h.write_u8(b.cost);
                h.write_u8(b.cell.0);
                h.write_u8(b.cell.1);
            }
        }
        // Pool remaining counts in sorted def-id order (via unit_index keys).
        for (id, _) in &self.unit_index {
            h.write_str(id);
            h.write_u32(u32::from(self.pool.remaining(id)));
        }
        h.finish()
    }

    pub fn snapshot(&self) -> MatchSnapshot {
        MatchSnapshot {
            phase: self.phase,
            round: self.round,
            players: (0..2)
                .map(|i| self.player_snapshot(i as u8))
                .collect(),
            state_hash: self.state_hash(),
            winner_player: self.winner_player,
        }
    }

    fn player_snapshot(&self, player_id: u8) -> PlayerSnapshot {
        let p = &self.players[player_id as usize];
        PlayerSnapshot {
            player_id,
            hp: p.hp,
            gold: p.economy.gold,
            xp: p.economy.xp,
            level: p.economy.level,
            shop_ready: p.shop_ready,
            shop: p
                .shop
                .slots
                .iter()
                .map(|s| {
                    s.as_ref().map(|o| ShopOfferSnap {
                        def_id: o.def_id.as_str().to_string(),
                        cost: o.cost,
                    })
                })
                .collect(),
            bench: p
                .economy
                .bench
                .iter()
                .map(|u| OwnedUnitSnap {
                    instance_id: u.instance_id.0,
                    def_id: u.def_id.as_str().to_string(),
                    cost: u.cost,
                })
                .collect(),
            board: p.board.clone(),
        }
    }

    fn open_shop_phase(&mut self, grant_income: bool) {
        self.phase = Phase::Shop;
        for p in &mut self.players {
            p.shop_ready = false;
            if grant_income {
                let bonus = ROUND_INCOME_BASE + interest(p.economy.gold);
                p.economy.gold = p.economy.gold.saturating_add(bonus);
            }
        }
        // Deterministic shop order: player 0 then 1.
        for i in 0..2 {
            refresh_shop(
                &mut self.players[i].shop,
                &mut self.pool,
                &mut self.rng,
                self.players[i].economy.level,
            );
        }
        // AI plays immediately so it is ready when human ends.
        let mut discard = Vec::new();
        self.run_ai_shop(&mut discard);
    }

    fn run_ai_shop(&mut self, events: &mut Vec<Event>) {
        const AI: usize = 1;
        if self.players[AI].shop_ready {
            return;
        }

        // 1. Level: while gold ≥ 4 and level < 5, buy exp.
        //    Optional: if level < round+1, still buy once when affordable.
        while self.players[AI].economy.gold >= BUY_XP_COST && self.players[AI].economy.level < 5 {
            if buy_xp(&mut self.players[AI].economy).is_err() {
                break;
            }
            events.push(Event::BoughtExp {
                player_id: 1,
                level: self.players[AI].economy.level,
                xp: self.players[AI].economy.xp,
            });
        }
        if self.players[AI].economy.level < self.round.saturating_add(1).min(255) as u8
            && self.players[AI].economy.level < 5
            && self.players[AI].economy.gold >= BUY_XP_COST
        {
            if buy_xp(&mut self.players[AI].economy).is_ok() {
                events.push(Event::BoughtExp {
                    player_id: 1,
                    level: self.players[AI].economy.level,
                    xp: self.players[AI].economy.xp,
                });
            }
        }

        // 2. Reroll at most once if nothing affordable.
        if !self.ai_has_affordable_unit() && self.players[AI].economy.gold >= REROLL_COST {
            if reroll(
                &mut self.players[AI].economy,
                &mut self.players[AI].shop,
                &mut self.pool,
                &mut self.rng,
            )
            .is_ok()
            {
                events.push(Event::Rerolled { player_id: 1 });
            }
        }

        // 3. Buy cheapest affordable slot left-to-right, repeatedly.
        loop {
            let Some(idx) = self.ai_cheapest_affordable_index() else {
                break;
            };
            match buy(
                &mut self.players[AI].economy,
                &mut self.players[AI].shop,
                idx,
            ) {
                Ok(id) => {
                    let unit = self.players[AI]
                        .economy
                        .bench
                        .iter()
                        .find(|u| u.instance_id == id)
                        .expect("just bought");
                    events.push(Event::UnitBought {
                        player_id: 1,
                        instance_id: id.0,
                        def_id: unit.def_id.as_str().to_string(),
                        cost: unit.cost,
                    });
                }
                Err(_) => break,
            }
        }

        // 4. Place units on first empty cells row-major up to board cap.
        let cap = board_cap_for_level(self.players[AI].economy.level) as usize;
        while self.players[AI].board.len() < cap {
            let Some(unit) = self.players[AI].economy.bench.first().cloned() else {
                break;
            };
            let Some(cell) = self.first_empty_cell(AI) else {
                break;
            };
            // Remove from bench then place.
            self.players[AI]
                .economy
                .bench
                .retain(|u| u.instance_id != unit.instance_id);
            self.players[AI].board.push(BoardUnit {
                instance_id: unit.instance_id.0,
                def_id: unit.def_id.as_str().to_string(),
                cost: unit.cost,
                cell,
            });
            events.push(Event::UnitPlaced {
                player_id: 1,
                instance_id: unit.instance_id.0,
                cell,
            });
        }

        // 5. Auto end shop.
        self.players[AI].shop_ready = true;
        events.push(Event::ShopEnded { player_id: 1 });
    }

    fn ai_has_affordable_unit(&self) -> bool {
        let gold = self.players[1].economy.gold;
        self.players[1]
            .shop
            .slots
            .iter()
            .any(|s| s.as_ref().is_some_and(|o| gold >= o.cost as i32))
    }

    fn ai_cheapest_affordable_index(&self) -> Option<usize> {
        let gold = self.players[1].economy.gold;
        let mut best: Option<(u8, usize)> = None;
        for (i, slot) in self.players[1].shop.slots.iter().enumerate() {
            if let Some(o) = slot {
                if gold >= o.cost as i32 {
                    match best {
                        None => best = Some((o.cost, i)),
                        Some((c, _)) if o.cost < c => best = Some((o.cost, i)),
                        Some((c, bi)) if o.cost == c && i < bi => best = Some((o.cost, i)),
                        _ => {}
                    }
                }
            }
        }
        best.map(|(_, i)| i)
    }

    fn first_empty_cell(&self, player: usize) -> Option<(u8, u8)> {
        for y in 0..LOCAL_BOARD_H {
            for x in 0..LOCAL_BOARD_W {
                let occupied = self.players[player]
                    .board
                    .iter()
                    .any(|b| b.cell == (x, y));
                if !occupied {
                    return Some((x, y));
                }
            }
        }
        None
    }

    fn sell_unit(&mut self, player: u8, id: UnitInstanceId) -> Result<(), CoreError> {
        let p = player as usize;
        // Bench first.
        if self.players[p]
            .economy
            .bench
            .iter()
            .any(|u| u.instance_id == id)
        {
            sell(&mut self.players[p].economy, &mut self.pool, id)?;
            return Ok(());
        }
        // Board.
        let pos = self.players[p]
            .board
            .iter()
            .position(|b| b.instance_id == id.0)
            .ok_or(CoreError::UnitNotFound)?;
        let unit = self.players[p].board.remove(pos);
        self.players[p].economy.gold += unit.cost as i32;
        self.pool.return_unit(&unit.def_id);
        Ok(())
    }

    fn place_unit(
        &mut self,
        player: u8,
        id: UnitInstanceId,
        cell: (u8, u8),
    ) -> Result<(), CoreError> {
        let p = player as usize;
        if cell.0 >= LOCAL_BOARD_W || cell.1 >= LOCAL_BOARD_H {
            return Err(CoreError::InvalidCell);
        }

        // Already on board? Move.
        if let Some(idx) = self.players[p]
            .board
            .iter()
            .position(|b| b.instance_id == id.0)
        {
            if self.players[p]
                .board
                .iter()
                .any(|b| b.cell == cell && b.instance_id != id.0)
            {
                return Err(CoreError::CellOccupied);
            }
            self.players[p].board[idx].cell = cell;
            return Ok(());
        }

        // From bench.
        let bench_pos = self.players[p]
            .economy
            .bench
            .iter()
            .position(|u| u.instance_id == id)
            .ok_or(CoreError::UnitNotFound)?;

        if self.players[p].board.iter().any(|b| b.cell == cell) {
            return Err(CoreError::CellOccupied);
        }

        let cap = board_cap_for_level(self.players[p].economy.level) as usize;
        if self.players[p].board.len() >= cap {
            return Err(CoreError::BoardFull);
        }

        let unit = self.players[p].economy.bench.remove(bench_pos);
        self.players[p].board.push(BoardUnit {
            instance_id: unit.instance_id.0,
            def_id: unit.def_id.as_str().to_string(),
            cost: unit.cost,
            cell,
        });
        Ok(())
    }

    fn resolve_combat_and_advance(&mut self, events: &mut Vec<Event>) -> Result<(), CoreError> {
        self.phase = Phase::Combat;
        events.push(Event::PhaseChanged {
            phase: Phase::Combat,
        });

        let combat_seed = self
            .seed
            .wrapping_add(u64::from(self.round).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let mut combat = CombatState::new(self.plugin.abilities.clone(), combat_seed);

        // Side A = player 0, Side B = player 1.
        self.spawn_board_into_combat(0, Side::A, &mut combat);
        self.spawn_board_into_combat(1, Side::B, &mut combat);

        let result = combat.run_to_completion();
        let living_a = combat.living_on(Side::A).count() as u8;
        let living_b = combat.living_on(Side::B).count() as u8;

        let winner_player = match result {
            CombatResult::Winner(Side::A) => Some(0u8),
            CombatResult::Winner(Side::B) => Some(1u8),
            CombatResult::Draw => None,
        };

        events.push(Event::CombatResolved {
            winner_player,
            living_a,
            living_b,
        });

        // Apply damage to loser(s).
        match result {
            CombatResult::Winner(Side::A) => {
                let dmg = LOSS_DAMAGE_BASE + living_a as i32;
                self.apply_damage(1, dmg, events);
            }
            CombatResult::Winner(Side::B) => {
                let dmg = LOSS_DAMAGE_BASE + living_b as i32;
                self.apply_damage(0, dmg, events);
            }
            CombatResult::Draw => {}
        }

        // Match end if any HP ≤ 0.
        let p0_dead = self.players[0].hp <= 0;
        let p1_dead = self.players[1].hp <= 0;
        if p0_dead || p1_dead {
            self.phase = Phase::MatchEnd;
            self.winner_player = match (p0_dead, p1_dead) {
                (true, false) => Some(1),
                (false, true) => Some(0),
                (true, true) => None,
                (false, false) => None,
            };
            events.push(Event::PhaseChanged {
                phase: Phase::MatchEnd,
            });
            events.push(Event::MatchEnded {
                winner_player: self.winner_player,
            });
            return Ok(());
        }

        // Next shop.
        self.round = self.round.saturating_add(1);
        events.push(Event::RoundAdvanced { round: self.round });
        self.open_shop_phase(true);
        events.push(Event::PhaseChanged {
            phase: Phase::Shop,
        });
        Ok(())
    }

    fn apply_damage(&mut self, player: u8, amount: i32, events: &mut Vec<Event>) {
        let p = &mut self.players[player as usize];
        p.hp = p.hp.saturating_sub(amount);
        events.push(Event::PlayerDamaged {
            player_id: player,
            amount,
            hp_after: p.hp,
        });
    }

    fn spawn_board_into_combat(&self, player: u8, side: Side, combat: &mut CombatState) {
        for b in &self.players[player as usize].board {
            let Some(def) = self.unit_index.get(&b.def_id) else {
                continue;
            };
            let (x, y) = local_to_combat(side, b.cell);
            if !valid_spawn(side, x, y) {
                continue;
            }
            // Unique combat ids: high byte encodes player seat.
            let cid = UnitInstanceId(((player as u32) << 24) | b.instance_id);
            combat.add_unit(CombatUnit::from_def(cid, side, def, x, y));
        }
    }
}

/// Map local half-board cell to combat coordinates.
///
/// Local `(x, 0)` = front row facing the opponent.
/// - Side A (bottom): front y=2, back y=3
/// - Side B (top): front y=1, back y=0
pub fn local_to_combat(side: Side, cell: (u8, u8)) -> (i32, i32) {
    let x = cell.0 as i32;
    let y = match side {
        Side::A => 2 + cell.1 as i32,
        Side::B => 1 - cell.1 as i32,
    };
    debug_assert!(x >= 0 && x < BOARD_SIZE);
    (x, y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::DefId;
    use crate::plugin::{Manifest, UnitStats};

    fn plugin() -> PluginData {
        PluginData {
            manifest: Manifest {
                id: "t".into(),
                name: "t".into(),
                version: "0.1.0".into(),
                api_version: "1.0.0".into(),
                description: None,
            },
            units: vec![UnitDef {
                id: DefId::new("u1"),
                name: "U1".into(),
                cost: 1,
                traits: vec![],
                ability_id: None,
                stats: UnitStats {
                    hp: 200,
                    atk: 40,
                    armor: 0,
                    mr: 0,
                    range: 1,
                    attack_speed_milli: 1000,
                },
            }],
            traits: vec![],
            abilities: vec![],
        }
    }

    #[test]
    fn new_match_opens_shop_with_ai_ready() {
        let m = Match::new(plugin(), 1);
        assert_eq!(m.phase(), Phase::Shop);
        assert!(m.players[1].shop_ready);
        assert!(!m.players[0].shop_ready);
        assert_eq!(m.players[0].hp, STARTING_HP);
    }

    #[test]
    fn end_shop_runs_combat_and_returns_to_shop_or_end() {
        let mut m = Match::new(plugin(), 7);
        let events = m.apply(0, Command::EndShopPhase).unwrap();
        assert!(events.iter().any(|e| matches!(
            e,
            Event::CombatResolved { .. }
        )));
        assert!(m.phase() == Phase::Shop || m.phase() == Phase::MatchEnd);
    }

    #[test]
    fn local_to_combat_front_rows_face() {
        assert_eq!(local_to_combat(Side::A, (0, 0)), (0, 2));
        assert_eq!(local_to_combat(Side::A, (1, 1)), (1, 3));
        assert_eq!(local_to_combat(Side::B, (0, 0)), (0, 1));
        assert_eq!(local_to_combat(Side::B, (2, 1)), (2, 0));
    }

    #[test]
    fn state_hash_stable_across_clones_path() {
        let p = plugin();
        let mut a = Match::new(p.clone(), 42);
        let mut b = Match::new(p, 42);
        assert_eq!(a.state_hash(), b.state_hash());
        let _ = a.apply(0, Command::EndShopPhase);
        let _ = b.apply(0, Command::EndShopPhase);
        assert_eq!(a.state_hash(), b.state_hash());
    }
}
