use serde::{Deserialize, Serialize};

use super::buildings::BuildingKind;
use super::resources::Resources;

pub type UpgradeId = usize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Upgrade {
    pub id: UpgradeId,
    pub name: String,
    pub description: String,
    pub cost: Resources,
    pub prerequisites: Vec<UpgradeId>,
    pub effect: UpgradeEffect,
    pub purchased: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpgradeEffect {
    MultiplyProduction(BuildingKind, f64),
    MultiplyAllProduction(f64),
    ReduceCost(BuildingKind, f64),
    UnlockBuilding(BuildingKind),
    IncreaseOfflineEfficiency(f64),
    IncreaseTaskReward(f64),
}

pub fn all_upgrades() -> Vec<Upgrade> {
    vec![
        // Tier 1 upgrades
        Upgrade {
            id: 0,
            name: "Overclocking".into(),
            description: "x2 Raspberry Pi production".into(),
            cost: Resources { compute: 500.0, ..Default::default() },
            prerequisites: vec![],
            effect: UpgradeEffect::MultiplyProduction(BuildingKind::RaspberryPi, 2.0),
            purchased: false,
        },
        Upgrade {
            id: 1,
            name: "QoS Rules".into(),
            description: "x2 Home Router production".into(),
            cost: Resources { bandwidth: 300.0, ..Default::default() },
            prerequisites: vec![],
            effect: UpgradeEffect::MultiplyProduction(BuildingKind::HomeRouter, 2.0),
            purchased: false,
        },
        Upgrade {
            id: 2,
            name: "USB 3.0".into(),
            description: "x2 USB Drive production".into(),
            cost: Resources { storage: 400.0, ..Default::default() },
            prerequisites: vec![],
            effect: UpgradeEffect::MultiplyProduction(BuildingKind::USBDrive, 2.0),
            purchased: false,
        },
        // Tier 2 upgrades
        Upgrade {
            id: 3,
            name: "Containerization".into(),
            description: "x2 VPS production".into(),
            cost: Resources { compute: 5_000.0, ..Default::default() },
            prerequisites: vec![0],
            effect: UpgradeEffect::MultiplyProduction(BuildingKind::VPS, 2.0),
            purchased: false,
        },
        Upgrade {
            id: 4,
            name: "Fiber Optic Upgrade".into(),
            description: "x2 Fiber Connection production".into(),
            cost: Resources { bandwidth: 3_000.0, ..Default::default() },
            prerequisites: vec![1],
            effect: UpgradeEffect::MultiplyProduction(BuildingKind::FiberConnection, 2.0),
            purchased: false,
        },
        Upgrade {
            id: 5,
            name: "RAID Configuration".into(),
            description: "x2 NAS Box production".into(),
            cost: Resources { storage: 4_000.0, ..Default::default() },
            prerequisites: vec![2],
            effect: UpgradeEffect::MultiplyProduction(BuildingKind::NASBox, 2.0),
            purchased: false,
        },
        // Global upgrades
        Upgrade {
            id: 6,
            name: "Automation Scripts".into(),
            description: "x1.25 all production".into(),
            cost: Resources { compute: 10_000.0, ..Default::default() },
            prerequisites: vec![3],
            effect: UpgradeEffect::MultiplyAllProduction(1.25),
            purchased: false,
        },
        Upgrade {
            id: 7,
            name: "Kubernetes".into(),
            description: "x1.5 all production".into(),
            cost: Resources { compute: 100_000.0, bandwidth: 50_000.0, ..Default::default() },
            prerequisites: vec![6],
            effect: UpgradeEffect::MultiplyAllProduction(1.5),
            purchased: false,
        },
        Upgrade {
            id: 8,
            name: "Terraform".into(),
            description: "x1.5 all production".into(),
            cost: Resources { compute: 1_000_000.0, ..Default::default() },
            prerequisites: vec![7],
            effect: UpgradeEffect::MultiplyAllProduction(1.5),
            purchased: false,
        },
        // Tier 3 upgrades
        Upgrade {
            id: 9,
            name: "Blade Servers".into(),
            description: "x3 Dedicated Server production".into(),
            cost: Resources { compute: 50_000.0, ..Default::default() },
            prerequisites: vec![3],
            effect: UpgradeEffect::MultiplyProduction(BuildingKind::DedicatedServer, 3.0),
            purchased: false,
        },
        Upgrade {
            id: 10,
            name: "Anycast Routing".into(),
            description: "x3 Load Balancer production".into(),
            cost: Resources { bandwidth: 30_000.0, ..Default::default() },
            prerequisites: vec![4],
            effect: UpgradeEffect::MultiplyProduction(BuildingKind::LoadBalancer, 3.0),
            purchased: false,
        },
        // Task reward upgrades
        Upgrade {
            id: 11,
            name: "Incident Playbooks".into(),
            description: "x2 task rewards".into(),
            cost: Resources { compute: 20_000.0, ..Default::default() },
            prerequisites: vec![],
            effect: UpgradeEffect::IncreaseTaskReward(2.0),
            purchased: false,
        },
        // Offline upgrades
        Upgrade {
            id: 12,
            name: "Cron Jobs".into(),
            description: "50% offline efficiency (up from 25%)".into(),
            cost: Resources { compute: 50_000.0, ..Default::default() },
            prerequisites: vec![6],
            effect: UpgradeEffect::IncreaseOfflineEfficiency(0.50),
            purchased: false,
        },
        Upgrade {
            id: 13,
            name: "Systemd Timers".into(),
            description: "75% offline efficiency".into(),
            cost: Resources { compute: 500_000.0, ..Default::default() },
            prerequisites: vec![12],
            effect: UpgradeEffect::IncreaseOfflineEfficiency(0.75),
            purchased: false,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upgrade_ids_unique() {
        let upgrades = all_upgrades();
        let mut ids: Vec<_> = upgrades.iter().map(|u| u.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), upgrades.len());
    }

    #[test]
    fn test_prerequisites_valid() {
        let upgrades = all_upgrades();
        let ids: Vec<_> = upgrades.iter().map(|u| u.id).collect();
        for upgrade in &upgrades {
            for prereq in &upgrade.prerequisites {
                assert!(ids.contains(prereq), "Upgrade {} has invalid prerequisite {}", upgrade.id, prereq);
            }
        }
    }
}
