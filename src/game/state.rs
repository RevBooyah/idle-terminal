use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

use super::buildings::{all_building_defs, BuildingInstance, BuildingKind, ResourceType};
use super::events::{apply_event, maybe_generate_event, GameEvent, GameEventKind};
use super::resources::Resources;
use super::progression;
use super::upgrades::{all_upgrades, Upgrade, UpgradeEffect, UpgradeId};

const MAX_EVENT_LOG: usize = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub resources: Resources,
    pub buildings: HashMap<BuildingKind, BuildingInstance>,
    pub upgrades: Vec<Upgrade>,
    pub total_ticks: u64,
    pub global_multiplier: f64,
    pub production_per_tick: Resources,
    pub task_reward_multiplier: f64,
    pub offline_efficiency: f64,
    pub event_log: VecDeque<GameEvent>,
    pub traffic_spike_remaining: u32,
    pub traffic_spike_multiplier: f64,
    #[serde(skip, default = "default_rng")]
    pub rng: rand::rngs::StdRng,
    #[serde(default)]
    pub prestige_count: u32,
    #[serde(default)]
    pub lifetime_compute: f64,
    #[serde(default)]
    pub tasks_completed: u32,
    #[serde(default)]
    pub achievements: Vec<String>,
    #[serde(skip, default)]
    pub compute_history: VecDeque<u64>,
}

fn default_rng() -> rand::rngs::StdRng {
    rand::rngs::StdRng::from_entropy()
}

impl GameState {
    pub fn new() -> Self {
        let mut buildings = HashMap::new();
        for def in all_building_defs() {
            buildings.insert(def.kind, BuildingInstance::new(def.kind));
        }

        let mut state = Self {
            resources: Resources {
                compute: 50.0,
                ..Default::default()
            },
            buildings,
            upgrades: all_upgrades(),
            total_ticks: 0,
            global_multiplier: 1.0,
            production_per_tick: Resources::default(),
            task_reward_multiplier: 1.0,
            offline_efficiency: 0.25,
            event_log: VecDeque::new(),
            traffic_spike_remaining: 0,
            traffic_spike_multiplier: 1.0,
            rng: rand::rngs::StdRng::from_entropy(),
            prestige_count: 0,
            lifetime_compute: 0.0,
            tasks_completed: 0,
            achievements: Vec::new(),
            compute_history: VecDeque::new(),
        };
        state.recalculate_production();
        state
    }

    pub fn tick(&mut self) {
        self.total_ticks += 1;

        // Apply production (with traffic spike multiplier)
        let mut production = self.production_per_tick.clone();
        if self.traffic_spike_remaining > 0 {
            production.compute *= self.traffic_spike_multiplier;
            production.bandwidth *= self.traffic_spike_multiplier;
            production.storage *= self.traffic_spike_multiplier;
            self.traffic_spike_remaining -= 1;
        }
        self.resources.add(&production);

        // Track lifetime stats
        self.lifetime_compute += production.compute;

        // Update sparkline history every 4 ticks (1 second)
        if self.total_ticks % 4 == 0 {
            self.compute_history
                .push_back((self.resources.compute * 100.0) as u64);
            if self.compute_history.len() > 60 {
                self.compute_history.pop_front();
            }
        }

        // Try to generate a random event
        let monitoring_count = self
            .buildings
            .get(&BuildingKind::MonitoringStack)
            .map(|b| b.count)
            .unwrap_or(0);

        if let Some(event) = maybe_generate_event(
            &mut self.rng,
            self.total_ticks,
            monitoring_count,
            self.resources.compute,
        ) {
            // Apply immediate effects
            apply_event(&event.kind, &mut self.resources);

            // Handle traffic spike duration
            if let GameEventKind::TrafficSpike {
                multiplier,
                duration_ticks,
            } = &event.kind
            {
                self.traffic_spike_remaining = *duration_ticks;
                self.traffic_spike_multiplier = *multiplier;
            }

            // Log the event
            self.event_log.push_back(event);
            if self.event_log.len() > MAX_EVENT_LOG {
                self.event_log.pop_front();
            }
        }
    }

    pub fn recalculate_production(&mut self) {
        let defs = all_building_defs();
        let mut production = Resources::default();

        // Calculate CI/CD pipeline global bonus
        let cicd_count = self
            .buildings
            .get(&BuildingKind::CICDPipeline)
            .map(|b| b.count)
            .unwrap_or(0);
        let cicd_multiplier = 1.0 + (cicd_count as f64 * 0.10);

        // Calculate per-building upgrade multipliers
        let mut building_multipliers: HashMap<BuildingKind, f64> = HashMap::new();
        for upgrade in &self.upgrades {
            if !upgrade.purchased {
                continue;
            }
            if let UpgradeEffect::MultiplyProduction(kind, mult) = &upgrade.effect {
                let entry = building_multipliers.entry(*kind).or_insert(1.0);
                *entry *= mult;
            }
        }

        let total_multiplier = self.global_multiplier * cicd_multiplier;

        for def in &defs {
            if def.kind == BuildingKind::CICDPipeline {
                continue;
            }
            if let Some(instance) = self.buildings.get(&def.kind) {
                if instance.count == 0 {
                    continue;
                }
                let building_mult = building_multipliers.get(&def.kind).copied().unwrap_or(1.0);
                let prod = def.production_per_tick(
                    instance.count,
                    instance.level,
                    total_multiplier * building_mult,
                );
                match def.resource_type {
                    ResourceType::Compute => production.compute += prod,
                    ResourceType::Bandwidth => production.bandwidth += prod,
                    ResourceType::Storage => production.storage += prod,
                    ResourceType::Crypto => production.crypto += prod,
                }
            }
        }

        self.production_per_tick = production;
    }

    pub fn purchase_building(&mut self, kind: BuildingKind) -> bool {
        let defs = all_building_defs();
        let def = match defs.iter().find(|d| d.kind == kind) {
            Some(d) => d,
            None => return false,
        };

        let instance = match self.buildings.get(&kind) {
            Some(i) => i,
            None => return false,
        };

        let cost = def.cost_as_resources(instance.count);
        if !self.resources.can_afford(&cost) {
            return false;
        }

        self.resources.subtract(&cost);
        self.buildings.get_mut(&kind).unwrap().count += 1;
        self.recalculate_production();
        true
    }

    pub fn upgrade_building(&mut self, kind: BuildingKind) -> bool {
        let instance = match self.buildings.get(&kind) {
            Some(i) if i.count > 0 => i,
            _ => return false,
        };

        let defs = all_building_defs();
        let def = match defs.iter().find(|d| d.kind == kind) {
            Some(d) => d,
            None => return false,
        };

        let upgrade_cost = def.base_cost * 10.0 * 2.0_f64.powi(instance.level as i32);
        let cost = match def.resource_type {
            ResourceType::Compute => Resources {
                compute: upgrade_cost,
                ..Default::default()
            },
            ResourceType::Bandwidth => Resources {
                bandwidth: upgrade_cost,
                ..Default::default()
            },
            ResourceType::Storage => Resources {
                storage: upgrade_cost,
                ..Default::default()
            },
            ResourceType::Crypto => Resources {
                crypto: upgrade_cost,
                ..Default::default()
            },
        };

        if !self.resources.can_afford(&cost) {
            return false;
        }

        self.resources.subtract(&cost);
        self.buildings.get_mut(&kind).unwrap().level += 1;
        self.recalculate_production();
        true
    }

    pub fn purchase_upgrade(&mut self, id: UpgradeId) -> bool {
        // Check if already purchased
        let upgrade = match self.upgrades.iter().find(|u| u.id == id) {
            Some(u) => u,
            None => return false,
        };
        if upgrade.purchased {
            return false;
        }

        // Check prerequisites
        for prereq_id in &upgrade.prerequisites {
            if !self.upgrades.iter().any(|u| u.id == *prereq_id && u.purchased) {
                return false;
            }
        }

        // Check cost
        let cost = upgrade.cost.clone();
        if !self.resources.can_afford(&cost) {
            return false;
        }

        // Purchase
        self.resources.subtract(&cost);
        let upgrade = self.upgrades.iter_mut().find(|u| u.id == id).unwrap();
        upgrade.purchased = true;

        // Apply effect
        let effect = upgrade.effect.clone();
        match effect {
            UpgradeEffect::MultiplyAllProduction(mult) => {
                self.global_multiplier *= mult;
            }
            UpgradeEffect::IncreaseTaskReward(mult) => {
                self.task_reward_multiplier *= mult;
            }
            UpgradeEffect::IncreaseOfflineEfficiency(val) => {
                self.offline_efficiency = val;
            }
            // MultiplyProduction and ReduceCost are applied in recalculate_production
            _ => {}
        }

        self.recalculate_production();
        true
    }

    /// Get available (unpurchased, prerequisites met) upgrades.
    pub fn available_upgrades(&self) -> Vec<&Upgrade> {
        self.upgrades
            .iter()
            .filter(|u| {
                !u.purchased
                    && u.prerequisites
                        .iter()
                        .all(|p| self.upgrades.iter().any(|u2| u2.id == *p && u2.purchased))
            })
            .collect()
    }

    pub fn unlocked_buildings(&self) -> Vec<BuildingKind> {
        let peak_compute = self.resources.compute;
        let defs = all_building_defs();
        let mut unlocked: Vec<_> = defs
            .iter()
            .filter(|d| {
                peak_compute >= d.unlock_threshold
                    || self
                        .buildings
                        .get(&d.kind)
                        .map(|b| b.count)
                        .unwrap_or(0)
                        > 0
            })
            .map(|d| (d.tier, d.kind))
            .collect();
        unlocked.sort_by_key(|(tier, _)| *tier);
        unlocked.into_iter().map(|(_, kind)| kind).collect()
    }

    pub fn can_prestige(&self) -> bool {
        self.resources.compute >= 1_000_000.0
    }

    pub fn prestige(&mut self) -> f64 {
        let rep_earned = progression::prestige_reputation(self.resources.compute);
        self.resources.reputation += rep_earned;

        // Reset resources (keep reputation)
        self.resources.compute = 50.0;
        self.resources.bandwidth = 0.0;
        self.resources.storage = 0.0;
        self.resources.crypto = 0.0;

        // Reset buildings
        for instance in self.buildings.values_mut() {
            instance.count = 0;
            instance.level = 0;
        }

        // Reset upgrades
        for upgrade in &mut self.upgrades {
            upgrade.purchased = false;
        }

        // Apply reputation multiplier
        self.global_multiplier = progression::reputation_multiplier(self.resources.reputation);
        self.task_reward_multiplier = 1.0;
        self.offline_efficiency = 0.25;

        // Clear transient state
        self.event_log.clear();
        self.traffic_spike_remaining = 0;
        self.traffic_spike_multiplier = 1.0;
        self.compute_history.clear();

        self.prestige_count += 1;
        self.recalculate_production();

        rep_earned
    }

    pub fn check_achievements(&mut self) -> Vec<String> {
        let total_buildings: u32 = self.buildings.values().map(|b| b.count).sum();
        let upgrades_purchased = self.upgrades.iter().filter(|u| u.purchased).count();

        let checks: &[(&str, &str, bool)] = &[
            ("first_build", "Hello World", total_buildings >= 1),
            ("ten_builds", "Sys Admin", total_buildings >= 10),
            (
                "first_upgrade",
                "Patch Tuesday",
                upgrades_purchased >= 1,
            ),
            ("first_prestige", "Reboot", self.prestige_count >= 1),
            (
                "compute_1m",
                "Megahertz",
                self.lifetime_compute >= 1_000_000.0,
            ),
            (
                "compute_1b",
                "Gigaflops",
                self.lifetime_compute >= 1_000_000_000.0,
            ),
            (
                "compute_1t",
                "Teraflops",
                self.lifetime_compute >= 1_000_000_000_000.0,
            ),
            ("task_10", "On Call", self.tasks_completed >= 10),
            (
                "task_50",
                "Incident Commander",
                self.tasks_completed >= 50,
            ),
            ("prestige_5", "Veteran", self.prestige_count >= 5),
        ];

        let mut newly_unlocked = Vec::new();
        for &(id, name, condition) in checks {
            if condition && !self.achievements.contains(&id.to_string()) {
                self.achievements.push(id.to_string());
                newly_unlocked.push(name.to_string());
            }
        }
        newly_unlocked
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game_state() {
        let state = GameState::new();
        assert_eq!(state.resources.compute, 50.0);
        assert_eq!(state.total_ticks, 0);
    }

    #[test]
    fn test_purchase_building() {
        let mut state = GameState::new();
        state.resources.compute = 100.0;

        let success = state.purchase_building(BuildingKind::RaspberryPi);
        assert!(success);
        assert_eq!(state.buildings[&BuildingKind::RaspberryPi].count, 1);
        assert!(state.resources.compute < 100.0);
        assert!(state.production_per_tick.compute > 0.0);
    }

    #[test]
    fn test_cannot_afford() {
        let mut state = GameState::new();
        state.resources.compute = 0.0;

        let success = state.purchase_building(BuildingKind::RaspberryPi);
        assert!(!success);
        assert_eq!(state.buildings[&BuildingKind::RaspberryPi].count, 0);
    }

    #[test]
    fn test_tick_produces_resources() {
        let mut state = GameState::new();
        state.resources.compute = 100.0;
        state.purchase_building(BuildingKind::RaspberryPi);

        let compute_before = state.resources.compute;
        state.tick();
        assert!(state.resources.compute > compute_before);
    }

    #[test]
    fn test_purchase_upgrade() {
        let mut state = GameState::new();
        state.resources.compute = 1000.0;
        state.purchase_building(BuildingKind::RaspberryPi);

        let prod_before = state.production_per_tick.compute;
        let success = state.purchase_upgrade(0); // Overclocking: x2 RaspberryPi
        assert!(success);
        assert!(state.production_per_tick.compute > prod_before);
        // Should be approximately 2x
        assert!((state.production_per_tick.compute / prod_before - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_upgrade_prerequisites() {
        let mut state = GameState::new();
        state.resources.compute = 100_000.0;

        // Containerization (id=3) requires Overclocking (id=0)
        let success = state.purchase_upgrade(3);
        assert!(!success); // Should fail: missing prerequisite

        state.purchase_upgrade(0); // Buy Overclocking first
        let success = state.purchase_upgrade(3);
        assert!(success); // Now should succeed
    }

    #[test]
    fn test_prestige() {
        let mut state = GameState::new();
        state.resources.compute = 4_000_000.0;

        assert!(state.can_prestige());
        let rep = state.prestige();
        assert_eq!(rep, 2.0); // sqrt(4M / 1M) = 2
        assert_eq!(state.resources.reputation, 2.0);
        assert_eq!(state.resources.compute, 50.0);
        assert_eq!(state.prestige_count, 1);
        assert!(state.global_multiplier > 1.0);
    }

    #[test]
    fn test_cannot_prestige_under_threshold() {
        let state = GameState::new();
        assert!(!state.can_prestige());
    }

    #[test]
    fn test_check_achievements() {
        let mut state = GameState::new();
        state.resources.compute = 100.0;
        state.purchase_building(BuildingKind::RaspberryPi);

        let new = state.check_achievements();
        assert!(new.contains(&"Hello World".to_string()));

        // Checking again shouldn't return duplicates
        let new2 = state.check_achievements();
        assert!(!new2.contains(&"Hello World".to_string()));
    }

    #[test]
    fn test_global_multiplier_upgrade() {
        let mut state = GameState::new();
        state.resources.compute = 200_000.0;
        state.purchase_building(BuildingKind::RaspberryPi);

        let prod_before = state.production_per_tick.compute;
        state.purchase_upgrade(0); // Overclocking (prereq for Automation Scripts)
        state.purchase_upgrade(3); // Containerization (prereq for Automation Scripts)
        state.purchase_upgrade(6); // Automation Scripts: x1.25 all
        let prod_after = state.production_per_tick.compute;

        // Should be 2x (overclocking) * 1.25 (automation) = 2.5x
        assert!((prod_after / prod_before - 2.5).abs() < 0.1);
    }
}
