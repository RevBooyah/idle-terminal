/// Calculate reputation earned from a prestige.
/// Formula: floor(sqrt(total_compute / 1_000_000))
pub fn prestige_reputation(compute: f64) -> f64 {
    (compute / 1_000_000.0).sqrt().floor().max(0.0)
}

/// Calculate global multiplier from total reputation.
/// Each point of reputation gives +10% production.
pub fn reputation_multiplier(reputation: f64) -> f64 {
    1.0 + 0.10 * reputation
}

/// Static achievement definition.
pub struct AchievementDef {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
}

pub fn all_achievement_defs() -> Vec<AchievementDef> {
    vec![
        AchievementDef {
            id: "first_build",
            name: "Hello World",
            description: "Purchase your first building",
        },
        AchievementDef {
            id: "ten_builds",
            name: "Sys Admin",
            description: "Own 10 buildings total",
        },
        AchievementDef {
            id: "first_upgrade",
            name: "Patch Tuesday",
            description: "Purchase your first upgrade",
        },
        AchievementDef {
            id: "first_prestige",
            name: "Reboot",
            description: "Prestige for the first time",
        },
        AchievementDef {
            id: "compute_1m",
            name: "Megahertz",
            description: "Accumulate 1M compute",
        },
        AchievementDef {
            id: "compute_1b",
            name: "Gigaflops",
            description: "Accumulate 1B compute",
        },
        AchievementDef {
            id: "compute_1t",
            name: "Teraflops",
            description: "Accumulate 1T compute",
        },
        AchievementDef {
            id: "task_10",
            name: "On Call",
            description: "Complete 10 tasks",
        },
        AchievementDef {
            id: "task_50",
            name: "Incident Commander",
            description: "Complete 50 tasks",
        },
        AchievementDef {
            id: "prestige_5",
            name: "Veteran",
            description: "Prestige 5 times",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prestige_reputation() {
        assert_eq!(prestige_reputation(0.0), 0.0);
        assert_eq!(prestige_reputation(999_999.0), 0.0);
        assert_eq!(prestige_reputation(1_000_000.0), 1.0);
        assert_eq!(prestige_reputation(4_000_000.0), 2.0);
        assert_eq!(prestige_reputation(9_000_000.0), 3.0);
        assert_eq!(prestige_reputation(100_000_000.0), 10.0);
    }

    #[test]
    fn test_reputation_multiplier() {
        assert_eq!(reputation_multiplier(0.0), 1.0);
        assert!((reputation_multiplier(1.0) - 1.1).abs() < 0.001);
        assert!((reputation_multiplier(10.0) - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_achievement_ids_unique() {
        let defs = all_achievement_defs();
        let mut ids: Vec<_> = defs.iter().map(|d| d.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), defs.len());
    }
}
