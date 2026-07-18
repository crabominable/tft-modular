//! Player commands (partial — shop/economy subset until match loop lands).

use serde::{Deserialize, Serialize};

/// Commands a human (or AI) may issue during a match.
///
/// Task 6 implements shop/economy actions only. Placement and phase end
/// arrive with the match state machine (Task 8).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Command {
    BuyUnit { shop_index: u8 },
    SellUnit { unit_instance_id: u32 },
    Reroll,
    BuyExp,
    PlaceUnit {
        unit_instance_id: u32,
        cell: (u8, u8),
    },
    EndShopPhase,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_json_roundtrip() {
        let cmds = [
            Command::BuyUnit { shop_index: 2 },
            Command::SellUnit {
                unit_instance_id: 7,
            },
            Command::Reroll,
            Command::BuyExp,
        ];
        for cmd in cmds {
            let json = serde_json::to_string(&cmd).unwrap();
            let again: Command = serde_json::from_str(&json).unwrap();
            assert_eq!(cmd, again);
        }
    }
}
