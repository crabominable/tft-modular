//! Replay log: seed + ordered player commands → reproducible final state hash.

use serde::{Deserialize, Serialize};

use crate::command::Command;
use crate::match_state::{CoreError, Match};
use crate::plugin::PluginData;

/// One logged player input (human seat only in MVP; AI is pure from state).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayInput {
    pub player_id: u8,
    pub command: Command,
}

/// Compact replay package for tests / export.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Replay {
    pub seed: u64,
    /// Pack identity string (manifest id or content hash from host).
    pub mod_id: String,
    pub commands: Vec<ReplayInput>,
}

impl Replay {
    pub fn new(seed: u64, mod_id: impl Into<String>) -> Self {
        Self {
            seed,
            mod_id: mod_id.into(),
            commands: Vec::new(),
        }
    }

    pub fn push(&mut self, player_id: u8, command: Command) {
        self.commands.push(ReplayInput {
            player_id,
            command,
        });
    }
}

/// Re-run a command log and return the final `state_hash`.
pub fn run_replay(plugin: PluginData, replay: &Replay) -> Result<u64, CoreError> {
    let mut m = Match::new(plugin, replay.seed);
    for input in &replay.commands {
        m.apply(input.player_id, input.command.clone())?;
    }
    Ok(m.state_hash())
}

/// Apply a command sequence and return the final hash (test helper shape).
pub fn run_commands(
    plugin: PluginData,
    seed: u64,
    cmds: &[(u8, Command)],
) -> Result<u64, CoreError> {
    let mut m = Match::new(plugin, seed);
    for (player_id, cmd) in cmds {
        m.apply(*player_id, cmd.clone())?;
    }
    Ok(m.state_hash())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::{Manifest, PluginData, UnitDef, UnitStats};
    use crate::ids::DefId;

    fn tiny_plugin() -> PluginData {
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
                    hp: 100,
                    atk: 20,
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
    fn replay_run_is_deterministic() {
        let plugin = tiny_plugin();
        let cmds = vec![
            (0u8, Command::EndShopPhase),
            (0, Command::EndShopPhase),
        ];
        let h1 = run_commands(plugin.clone(), 99, &cmds).unwrap();
        let h2 = run_commands(plugin, 99, &cmds).unwrap();
        assert_eq!(h1, h2);
    }
}
