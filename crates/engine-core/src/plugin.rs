//! Plugin pack data types (serde), matching plugin-schema JSON shapes.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ids::DefId;

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("json parse error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Pack manifest (`manifest.json`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub api_version: String,
    #[serde(default)]
    pub description: Option<String>,
}

/// Unit combat stats (`stats` object on unit defs).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitStats {
    pub hp: i32,
    pub atk: i32,
    #[serde(default)]
    pub armor: i32,
    #[serde(default)]
    pub mr: i32,
    pub range: i32,
    pub attack_speed_milli: i32,
}

/// Unit definition (`units/*.json`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitDef {
    pub id: DefId,
    pub name: String,
    pub cost: u8,
    pub traits: Vec<DefId>,
    #[serde(default)]
    pub ability_id: Option<DefId>,
    pub stats: UnitStats,
}

/// Stat key used by trait modifiers and ability STAT_MOD effects.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatKind {
    Hp,
    Atk,
    Armor,
    Mr,
}

/// Single flat modifier on a trait breakpoint.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraitModifier {
    pub stat: StatKind,
    pub amount: i32,
}

/// Trait breakpoint (stack threshold → modifiers).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraitBreakpoint {
    pub min_units: u8,
    #[serde(default)]
    pub modifiers: Vec<TraitModifier>,
}

/// Trait definition (`traits/*.json`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraitDef {
    pub id: DefId,
    pub name: String,
    pub breakpoints: Vec<TraitBreakpoint>,
}

/// Ability trigger kinds (closed allowlist).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AbilityTrigger {
    OnCast,
    OnHit,
    OnDeath,
    RoundStart,
}

/// Targeting mode kinds (closed allowlist).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TargetingType {
    #[serde(rename = "SELF")]
    SelfTarget,
    NearestEnemy,
    RandomEnemy,
    AllEnemies,
}

/// Ability targeting block.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Targeting {
    #[serde(rename = "type")]
    pub kind: TargetingType,
    #[serde(default)]
    pub count: Option<u8>,
}

/// Effect kinds (closed allowlist).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EffectType {
    Damage,
    Heal,
    Shield,
    Stun,
    StatMod,
}

/// Damage/heal scaling mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Scaling {
    None,
    Ap,
    Atk,
}

/// Single ability effect entry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbilityEffect {
    #[serde(rename = "type")]
    pub kind: EffectType,
    #[serde(default)]
    pub amount: Option<i32>,
    #[serde(default)]
    pub duration_ms: Option<i32>,
    #[serde(default)]
    pub stat: Option<StatKind>,
    #[serde(default)]
    pub scaling: Option<Scaling>,
}

/// Ability definition (`abilities/*.json`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbilityDef {
    pub id: DefId,
    pub name: String,
    #[serde(default = "default_mana_cost")]
    pub mana_cost: i32,
    pub trigger: AbilityTrigger,
    pub targeting: Targeting,
    pub effects: Vec<AbilityEffect>,
}

fn default_mana_cost() -> i32 {
    100
}

/// In-memory validated pack bundle consumed by the core (no FS I/O).
///
/// WASM path JSON shape:
/// ```json
/// { "manifest": {...}, "units": [...], "traits": [...], "abilities": [...] }
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginData {
    pub manifest: Manifest,
    pub units: Vec<UnitDef>,
    pub traits: Vec<TraitDef>,
    pub abilities: Vec<AbilityDef>,
}

impl PluginData {
    /// Build from JSON text fragments (tests / host loaders). Does not touch the filesystem.
    pub fn from_json_files(
        manifest_json: &str,
        unit_jsons: &[&str],
        trait_jsons: &[&str],
        ability_jsons: &[&str],
    ) -> Result<Self, PluginError> {
        let manifest: Manifest = serde_json::from_str(manifest_json)?;
        let mut units = Vec::with_capacity(unit_jsons.len());
        for s in unit_jsons {
            units.push(serde_json::from_str(s)?);
        }
        let mut traits = Vec::with_capacity(trait_jsons.len());
        for s in trait_jsons {
            traits.push(serde_json::from_str(s)?);
        }
        let mut abilities = Vec::with_capacity(ability_jsons.len());
        for s in ability_jsons {
            abilities.push(serde_json::from_str(s)?);
        }
        Ok(Self {
            manifest,
            units,
            traits,
            abilities,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MANIFEST: &str = r#"{
        "id": "reference_embers",
        "name": "Ember Frontier",
        "version": "0.1.0",
        "api_version": "1.0.0",
        "description": "sample"
    }"#;

    const UNIT: &str = r#"{
        "id": "ember_scout",
        "name": "Ember Scout",
        "cost": 1,
        "traits": ["emberkin"],
        "ability_id": "spark_cut",
        "stats": {
            "hp": 500,
            "atk": 45,
            "armor": 10,
            "mr": 10,
            "range": 1,
            "attack_speed_milli": 700
        }
    }"#;

    const TRAIT: &str = r#"{
        "id": "emberkin",
        "name": "Emberkin",
        "breakpoints": [
            {
                "min_units": 2,
                "modifiers": [{ "stat": "atk", "amount": 15 }]
            }
        ]
    }"#;

    const ABILITY: &str = r#"{
        "id": "spark_cut",
        "name": "Spark Cut",
        "mana_cost": 60,
        "trigger": "ON_CAST",
        "targeting": { "type": "NEAREST_ENEMY", "count": 1 },
        "effects": [
            { "type": "DAMAGE", "amount": 120, "scaling": "ATK" }
        ]
    }"#;

    #[test]
    fn from_json_files_parses_reference_shapes() {
        let pack = PluginData::from_json_files(MANIFEST, &[UNIT], &[TRAIT], &[ABILITY])
            .expect("parse pack");
        assert_eq!(pack.manifest.id, "reference_embers");
        assert_eq!(pack.units.len(), 1);
        assert_eq!(pack.units[0].id.as_str(), "ember_scout");
        assert_eq!(pack.units[0].stats.hp, 500);
        assert_eq!(pack.traits[0].breakpoints[0].min_units, 2);
        assert_eq!(pack.abilities[0].trigger, AbilityTrigger::OnCast);
        assert_eq!(pack.abilities[0].effects[0].kind, EffectType::Damage);
        assert_eq!(pack.abilities[0].effects[0].scaling, Some(Scaling::Atk));
    }

    #[test]
    fn bundle_roundtrip_matches_wasm_shape() {
        let pack = PluginData::from_json_files(MANIFEST, &[UNIT], &[TRAIT], &[ABILITY]).unwrap();
        let json = serde_json::to_string(&pack).unwrap();
        let again: PluginData = serde_json::from_str(&json).unwrap();
        assert_eq!(pack, again);
    }

    #[test]
    fn ability_defaults_mana_and_self_target() {
        let json = r#"{
            "id": "mist_balm",
            "name": "Mist Balm",
            "trigger": "ON_CAST",
            "targeting": { "type": "SELF" },
            "effects": [{ "type": "HEAL", "amount": 180, "scaling": "NONE" }]
        }"#;
        let ab: AbilityDef = serde_json::from_str(json).unwrap();
        assert_eq!(ab.mana_cost, 100);
        assert_eq!(ab.targeting.kind, TargetingType::SelfTarget);
    }
}
