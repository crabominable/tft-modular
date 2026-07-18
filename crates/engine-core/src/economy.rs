//! Gold interest, XP/level thresholds, and board capacity.

/// Cumulative XP required to reach levels 1..=9 (index = level - 1).
///
/// From plan: `2→2, 3→6, 4→10, 5→20, 6→36, 7→56, 8→80, 9→100`.
pub const XP_THRESHOLDS: [u32; 9] = [0, 2, 6, 10, 20, 36, 56, 80, 100];

/// Maximum player level in MVP.
pub const MAX_LEVEL: u8 = 9;

/// Interest: `min(5, gold / 10)` (integer division). Negative gold yields 0.
pub fn interest(gold: i32) -> i32 {
    if gold <= 0 {
        return 0;
    }
    (gold / 10).min(5)
}

/// Level derived from total cumulative XP (clamped to 1..=9).
pub fn level_from_xp(xp: u32) -> u8 {
    let mut level = 1u8;
    for (idx, &need) in XP_THRESHOLDS.iter().enumerate().skip(1) {
        if xp >= need {
            level = (idx as u8) + 1;
        } else {
            break;
        }
    }
    level.min(MAX_LEVEL)
}

/// Board unit capacity equals player level in MVP.
pub fn board_cap_for_level(level: u8) -> u8 {
    level.clamp(1, MAX_LEVEL)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interest_table() {
        assert_eq!(interest(-5), 0);
        assert_eq!(interest(0), 0);
        assert_eq!(interest(9), 0);
        assert_eq!(interest(10), 1);
        assert_eq!(interest(49), 4);
        assert_eq!(interest(50), 5);
        assert_eq!(interest(999), 5);
    }

    #[test]
    fn level_from_xp_edges() {
        assert_eq!(level_from_xp(0), 1);
        assert_eq!(level_from_xp(1), 1);
        assert_eq!(level_from_xp(2), 2);
        assert_eq!(level_from_xp(5), 2);
        assert_eq!(level_from_xp(6), 3);
        assert_eq!(level_from_xp(100), 9);
        assert_eq!(level_from_xp(10_000), 9);
    }

    #[test]
    fn board_cap_equals_level() {
        assert_eq!(board_cap_for_level(1), 1);
        assert_eq!(board_cap_for_level(5), 5);
        assert_eq!(board_cap_for_level(9), 9);
    }
}
