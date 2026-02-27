use rand::Rng;
use serde::{Deserialize, Serialize};

use super::buildings::BuildingKind;
use super::resources::Resources;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEvent {
    pub kind: GameEventKind,
    pub tick: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameEventKind {
    ServerOverloaded(BuildingKind),
    DDoSAttack { severity: u8 },
    ViralRepo { bonus_reputation: f64 },
    SecurityBreach { lost_compute: f64 },
    TrafficSpike { multiplier: f64, duration_ticks: u32 },
    HardwareFailure(BuildingKind),
    BonusDrop { resource: BonusResource, amount: f64 },
    OpenSourceContribution { bonus_reputation: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BonusResource {
    Compute,
    Bandwidth,
    Storage,
}

impl GameEventKind {
    pub fn description(&self) -> String {
        match self {
            GameEventKind::ServerOverloaded(kind) => {
                format!("Server overloaded: {:?} throttled", kind)
            }
            GameEventKind::DDoSAttack { severity } => {
                format!("DDoS attack! Severity {}/10 - bandwidth drain", severity)
            }
            GameEventKind::ViralRepo { bonus_reputation } => {
                format!("Repo went viral! +{:.0} reputation", bonus_reputation)
            }
            GameEventKind::SecurityBreach { lost_compute } => {
                format!("Security breach! Lost {:.0} compute", lost_compute)
            }
            GameEventKind::TrafficSpike { multiplier, duration_ticks } => {
                format!("Traffic spike! x{:.1} production for {}s", multiplier, duration_ticks / 4)
            }
            GameEventKind::HardwareFailure(kind) => {
                format!("Hardware failure: {:?} offline temporarily", kind)
            }
            GameEventKind::BonusDrop { resource, amount } => {
                let res_name = match resource {
                    BonusResource::Compute => "compute",
                    BonusResource::Bandwidth => "bandwidth",
                    BonusResource::Storage => "storage",
                };
                format!("Bonus drop! +{:.0} {}", amount, res_name)
            }
            GameEventKind::OpenSourceContribution { bonus_reputation } => {
                format!("Open source PR merged! +{:.0} reputation", bonus_reputation)
            }
        }
    }

    pub fn severity_color(&self) -> EventSeverity {
        match self {
            GameEventKind::ServerOverloaded(_) => EventSeverity::Warning,
            GameEventKind::DDoSAttack { .. } => EventSeverity::Error,
            GameEventKind::ViralRepo { .. } => EventSeverity::Good,
            GameEventKind::SecurityBreach { .. } => EventSeverity::Error,
            GameEventKind::TrafficSpike { .. } => EventSeverity::Good,
            GameEventKind::HardwareFailure(_) => EventSeverity::Warning,
            GameEventKind::BonusDrop { .. } => EventSeverity::Good,
            GameEventKind::OpenSourceContribution { .. } => EventSeverity::Good,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EventSeverity {
    Good,
    Warning,
    Error,
}

/// Apply the immediate effect of a game event to resources.
pub fn apply_event(event: &GameEventKind, resources: &mut Resources) {
    match event {
        GameEventKind::DDoSAttack { severity } => {
            let drain = resources.bandwidth * 0.05 * (*severity as f64);
            resources.bandwidth = (resources.bandwidth - drain).max(0.0);
        }
        GameEventKind::SecurityBreach { lost_compute } => {
            resources.compute = (resources.compute - lost_compute).max(0.0);
        }
        GameEventKind::ViralRepo { bonus_reputation } => {
            resources.reputation += bonus_reputation;
        }
        GameEventKind::BonusDrop { resource, amount } => match resource {
            BonusResource::Compute => resources.compute += amount,
            BonusResource::Bandwidth => resources.bandwidth += amount,
            BonusResource::Storage => resources.storage += amount,
        },
        GameEventKind::OpenSourceContribution { bonus_reputation } => {
            resources.reputation += bonus_reputation;
        }
        // TrafficSpike, ServerOverloaded, HardwareFailure have duration-based
        // effects handled separately via active_effects in GameState
        _ => {}
    }
}

/// Probability of any event firing per tick. Scales with monitoring stacks.
const BASE_EVENT_CHANCE: f64 = 0.005; // ~2% per second at 4Hz
const MONITORING_BONUS: f64 = 0.002;

/// Try to generate a random event based on current game state.
pub fn maybe_generate_event(
    rng: &mut impl Rng,
    tick: u64,
    monitoring_count: u32,
    total_compute: f64,
) -> Option<GameEvent> {
    let chance = BASE_EVENT_CHANCE + (monitoring_count as f64 * MONITORING_BONUS);
    if rng.r#gen::<f64>() >= chance {
        return None;
    }

    // Weight good events higher than bad ones (60/40)
    let roll: f64 = rng.r#gen();
    let kind = if roll < 0.25 {
        // Bonus drop (25%)
        let amount = total_compute * 0.01 + 10.0; // 1% of current compute + base
        let resource = match rng.r#gen_range(0..3) {
            0 => BonusResource::Compute,
            1 => BonusResource::Bandwidth,
            _ => BonusResource::Storage,
        };
        GameEventKind::BonusDrop {
            resource,
            amount,
        }
    } else if roll < 0.40 {
        // Traffic spike (15%)
        GameEventKind::TrafficSpike {
            multiplier: 1.5 + rng.r#gen::<f64>() * 1.5, // 1.5x to 3.0x
            duration_ticks: rng.r#gen_range(20..60),     // 5-15 seconds
        }
    } else if roll < 0.50 {
        // Viral repo (10%)
        GameEventKind::ViralRepo {
            bonus_reputation: 1.0 + rng.r#gen::<f64>() * 5.0,
        }
    } else if roll < 0.60 {
        // Open source (10%)
        GameEventKind::OpenSourceContribution {
            bonus_reputation: 0.5 + rng.r#gen::<f64>() * 2.0,
        }
    } else if roll < 0.75 {
        // DDoS (15%)
        GameEventKind::DDoSAttack {
            severity: rng.r#gen_range(1..6),
        }
    } else if roll < 0.85 {
        // Security breach (10%)
        GameEventKind::SecurityBreach {
            lost_compute: total_compute * 0.02,
        }
    } else if roll < 0.95 {
        // Server overloaded (10%)
        GameEventKind::ServerOverloaded(BuildingKind::RaspberryPi) // Simplified
    } else {
        // Hardware failure (5%)
        GameEventKind::HardwareFailure(BuildingKind::VPS)
    };

    Some(GameEvent { kind, tick })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_bonus_drop() {
        let mut resources = Resources {
            compute: 100.0,
            ..Default::default()
        };
        apply_event(
            &GameEventKind::BonusDrop {
                resource: BonusResource::Compute,
                amount: 50.0,
            },
            &mut resources,
        );
        assert_eq!(resources.compute, 150.0);
    }

    #[test]
    fn test_apply_ddos() {
        let mut resources = Resources {
            bandwidth: 100.0,
            ..Default::default()
        };
        apply_event(
            &GameEventKind::DDoSAttack { severity: 5 },
            &mut resources,
        );
        assert!(resources.bandwidth < 100.0);
        assert!(resources.bandwidth >= 0.0);
    }

    #[test]
    fn test_event_descriptions() {
        let events = vec![
            GameEventKind::DDoSAttack { severity: 3 },
            GameEventKind::ViralRepo { bonus_reputation: 5.0 },
            GameEventKind::BonusDrop { resource: BonusResource::Compute, amount: 100.0 },
        ];
        for event in events {
            assert!(!event.description().is_empty());
        }
    }
}
