// crates/engine-core/tests/economy_tests.rs
use engine_core::economy::{board_cap_for_level, interest, level_from_xp, XP_THRESHOLDS};
use engine_core::ids::DefId;
use engine_core::plugin::{UnitDef, UnitStats};
use engine_core::pool::{UnitPool, COPIES_BY_COST, SHOP_SIZE};
use engine_core::rng::Rng;
use engine_core::shop::{
    buy, buy_xp, refresh_shop, reroll, sell, PlayerEconomy, ShopState, BENCH_CAPACITY, REROLL_COST,
    STARTING_GOLD,
};

#[test]
fn interest_caps_at_five() {
    assert_eq!(interest(0), 0);
    assert_eq!(interest(9), 0);
    assert_eq!(interest(10), 1);
    assert_eq!(interest(49), 4);
    assert_eq!(interest(50), 5);
    assert_eq!(interest(999), 5);
}

#[test]
fn level_thresholds_progress() {
    assert_eq!(level_from_xp(0), 1);
    assert_eq!(level_from_xp(2), 2);
    assert_eq!(level_from_xp(6), 3);
}

#[test]
fn xp_thresholds_match_parameter_table() {
    // cumulative XP to reach levels 1..=9
    assert_eq!(XP_THRESHOLDS, [0, 2, 6, 10, 20, 36, 56, 80, 100]);
    assert_eq!(level_from_xp(10), 4);
    assert_eq!(level_from_xp(20), 5);
    assert_eq!(level_from_xp(100), 9);
    assert_eq!(board_cap_for_level(4), 4);
}

fn sample_units() -> Vec<UnitDef> {
    vec![
        unit("ember_scout", 1),
        unit("mist_healer", 2),
        unit("stone_warden", 2),
        unit("rare_blade", 3),
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
fn pool_copies_follow_cost_table() {
    let pool = UnitPool::from_units(&sample_units());
    assert_eq!(pool.remaining("ember_scout"), COPIES_BY_COST[1]);
    assert_eq!(pool.remaining("mist_healer"), COPIES_BY_COST[2]);
    assert_eq!(pool.remaining("rare_blade"), COPIES_BY_COST[3]);
}

#[test]
fn shop_reroll_buy_sell_flow() {
    let mut pool = UnitPool::from_units(&sample_units());
    let mut rng = Rng::new(42);
    let mut player = PlayerEconomy::new();
    assert_eq!(player.gold, STARTING_GOLD);
    let mut shop = ShopState::empty();

    // Free initial fill (match start / shop phase entry).
    refresh_shop(&mut shop, &mut pool, &mut rng, player.level);
    assert_eq!(shop.slots.len(), SHOP_SIZE);
    assert!(shop.slots.iter().any(|s| s.is_some()));

    // Not enough gold for reroll at starting gold 3 with cost 2 is ok once; try with more.
    player.gold = 20;
    let gold_before = player.gold;
    reroll(&mut player, &mut shop, &mut pool, &mut rng).unwrap();
    assert_eq!(player.gold, gold_before - REROLL_COST);

    // Buy first filled slot.
    let idx = shop.slots.iter().position(|s| s.is_some()).expect("offer");
    let offer_cost = shop.slots[idx].as_ref().unwrap().cost;
    let offer_id = shop.slots[idx]
        .as_ref()
        .unwrap()
        .def_id
        .as_str()
        .to_string();
    let pool_before_buy = pool.remaining(&offer_id); // already drawn out
    let gold_before_buy = player.gold;
    let instance = buy(&mut player, &mut shop, idx).unwrap();
    assert_eq!(player.gold, gold_before_buy - offer_cost as i32);
    assert!(shop.slots[idx].is_none());
    assert_eq!(player.bench.len(), 1);
    // Buy does not change pool further (copy already removed on draw).
    assert_eq!(pool.remaining(&offer_id), pool_before_buy);

    // Sell refunds cost and returns one copy.
    sell(&mut player, &mut pool, instance).unwrap();
    assert_eq!(player.gold, gold_before_buy);
    assert!(player.bench.is_empty());
    assert_eq!(pool.remaining(&offer_id), pool_before_buy + 1);
}

#[test]
fn buy_xp_and_bench_capacity_constant() {
    assert_eq!(BENCH_CAPACITY, 9);
    let mut player = PlayerEconomy::with_gold(40);
    buy_xp(&mut player).unwrap();
    assert_eq!(player.xp, 4);
    assert_eq!(player.level, 2);
}
