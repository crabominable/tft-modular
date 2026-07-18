//! Stable identifiers used by the simulation.

use serde::{Deserialize, Serialize};

/// Pack definition id (unit / trait / ability slug from JSON `id`).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DefId(pub String);

impl DefId {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for DefId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<&str> for DefId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for DefId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Runtime unit instance on board, bench, or shop.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UnitInstanceId(pub u32);

/// Player seat index (0 = human in MVP, 1 = AI, …).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PlayerId(pub u8);
