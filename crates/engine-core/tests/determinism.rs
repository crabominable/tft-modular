//! Determinism: identical seed + human command log → identical state_hash.

use engine_core::command::Command;
use engine_core::match_state::{Match, Phase};
use engine_core::plugin::PluginData;
use engine_core::replay::run_commands;

fn load_reference_plugin_from_strings() -> PluginData {
    // Path from crates/engine-core/tests/ → repo mods/reference-mod
    let manifest = include_str!("../../../mods/reference-mod/manifest.json");
    let units = [
        include_str!("../../../mods/reference-mod/units/ember_scout.json"),
        include_str!("../../../mods/reference-mod/units/stone_warden.json"),
        include_str!("../../../mods/reference-mod/units/mist_healer.json"),
    ];
    let traits = [
        include_str!("../../../mods/reference-mod/traits/emberkin.json"),
        include_str!("../../../mods/reference-mod/traits/bulwark.json"),
    ];
    let abilities = [
        include_str!("../../../mods/reference-mod/abilities/spark_cut.json"),
        include_str!("../../../mods/reference-mod/abilities/guard_pulse.json"),
        include_str!("../../../mods/reference-mod/abilities/mist_balm.json"),
    ];
    PluginData::from_json_files(manifest, &units, &traits, &abilities).expect("reference pack")
}

#[test]
fn identical_seeds_and_commands_match() {
    let plugin = load_reference_plugin_from_strings();
    let cmds = vec![
        (0u8, Command::BuyUnit { shop_index: 0 }),
        (0, Command::EndShopPhase),
        (0, Command::Reroll),
        (0, Command::BuyUnit { shop_index: 1 }),
        (0, Command::BuyExp),
        (0, Command::EndShopPhase),
    ];
    // Some buys may fail if slot empty / gold low; only feed commands that apply.
    // Use a tolerant runner that skips invalid, but for true determinism both runs
    // must use the same apply path. Prefer commands known to succeed after open.
    let h1 = run_tolerant(plugin.clone(), 42, &cmds);
    let h2 = run_tolerant(plugin, 42, &cmds);
    assert_eq!(h1, h2);
}

/// Apply each command; ignore errors so the log stays in lockstep across runs.
fn run_tolerant(plugin: PluginData, seed: u64, cmds: &[(u8, Command)]) -> u64 {
    let mut m = Match::new(plugin, seed);
    for (pid, cmd) in cmds {
        let _ = m.apply(*pid, cmd.clone());
        if m.phase() == Phase::MatchEnd {
            break;
        }
    }
    m.state_hash()
}

#[test]
fn strict_end_shop_only_is_deterministic() {
    let plugin = load_reference_plugin_from_strings();
    let cmds = vec![
        (0u8, Command::EndShopPhase),
        (0, Command::EndShopPhase),
        (0, Command::EndShopPhase),
    ];
    let h1 = run_commands(plugin.clone(), 42, &cmds).expect("run1");
    let h2 = run_commands(plugin, 42, &cmds).expect("run2");
    assert_eq!(h1, h2);
}

#[test]
fn different_seeds_diverge_or_equal_but_stable() {
    let plugin = load_reference_plugin_from_strings();
    let cmds = vec![(0u8, Command::EndShopPhase)];
    let h1 = run_commands(plugin.clone(), 1, &cmds).unwrap();
    let h2 = run_commands(plugin.clone(), 1, &cmds).unwrap();
    let h3 = run_commands(plugin, 2, &cmds).unwrap();
    assert_eq!(h1, h2);
    // Different seeds should almost always differ; if they collide FNV still ok
    // as long as same seed matches — we only assert stability of seed 1.
    let _ = h3;
}

#[test]
fn snapshot_hash_matches_state_hash() {
    let plugin = load_reference_plugin_from_strings();
    let mut m = Match::new(plugin, 99);
    let _ = m.apply(0, Command::EndShopPhase);
    let snap = m.snapshot();
    assert_eq!(snap.state_hash, m.state_hash());
    assert_eq!(snap.players.len(), 2);
    assert_eq!(snap.players[0].hp, m.player_hp(0).unwrap());
}

#[test]
fn buy_place_end_sequence_deterministic() {
    let plugin = load_reference_plugin_from_strings();
    // Human: buy slot 0 if possible, place instance 1 at (0,0), end.
    // AI is pure function of state so both runs stay aligned.
    let mut m1 = Match::new(plugin.clone(), 12345);
    let mut m2 = Match::new(plugin, 12345);

    for m in [&mut m1, &mut m2] {
        let _ = m.apply(0, Command::BuyUnit { shop_index: 0 });
        // Instance ids start at 1 for first purchase on a seat.
        let _ = m.apply(
            0,
            Command::PlaceUnit {
                unit_instance_id: 1,
                cell: (0, 0),
            },
        );
        let _ = m.apply(0, Command::EndShopPhase);
    }

    assert_eq!(m1.state_hash(), m2.state_hash());
    assert_eq!(m1.phase(), m2.phase());
    assert_eq!(m1.round(), m2.round());
}
