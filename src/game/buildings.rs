use serde::{Deserialize, Serialize};

use super::formulas;
use super::resources::Resources;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BuildingKind {
    // Tier 1
    RaspberryPi,
    HomeRouter,
    USBDrive,
    // Tier 2
    VPS,
    FiberConnection,
    NASBox,
    // Tier 3
    DedicatedServer,
    LoadBalancer,
    SANArray,
    // Tier 4
    ServerCluster,
    CDN,
    DataWarehouse,
    // Tier 5
    Datacenter,
    BackboneLink,
    ObjectStorage,
    // Tier 6
    CloudRegion,
    SubmarineCable,
    DistributedFS,
    // Special
    CICDPipeline,
    MonitoringStack,
    CryptoMiner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingInstance {
    pub kind: BuildingKind,
    pub count: u32,
    pub level: u32,
}

impl BuildingInstance {
    pub fn new(kind: BuildingKind) -> Self {
        Self {
            kind,
            count: 0,
            level: 0,
        }
    }
}

/// Static definition of a building type.
pub struct BuildingDef {
    pub kind: BuildingKind,
    pub name: &'static str,
    pub description: &'static str,
    pub base_cost: f64,
    pub cost_multiplier: f64,
    pub base_production: f64,
    pub level_bonus: f64,
    pub resource_type: ResourceType,
    pub unlock_threshold: f64, // Compute threshold to unlock
    pub tier: u8,
}

#[derive(Debug, Clone, Copy)]
pub enum ResourceType {
    Compute,
    Bandwidth,
    Storage,
    Crypto,
}

impl BuildingDef {
    pub fn next_cost(&self, count: u32) -> f64 {
        formulas::building_cost(self.base_cost, self.cost_multiplier, count)
    }

    pub fn production_per_tick(&self, count: u32, level: u32, global_multiplier: f64) -> f64 {
        formulas::building_production(count, self.base_production, level, self.level_bonus, global_multiplier)
    }

    pub fn cost_as_resources(&self, count: u32) -> Resources {
        let cost = self.next_cost(count);
        match self.resource_type {
            ResourceType::Compute => Resources {
                compute: cost,
                ..Default::default()
            },
            ResourceType::Bandwidth => Resources {
                bandwidth: cost,
                ..Default::default()
            },
            ResourceType::Storage => Resources {
                storage: cost,
                ..Default::default()
            },
            ResourceType::Crypto => Resources {
                crypto: cost,
                ..Default::default()
            },
        }
    }
}

pub fn all_building_defs() -> Vec<BuildingDef> {
    vec![
        // Tier 1
        BuildingDef {
            kind: BuildingKind::RaspberryPi,
            name: "Raspberry Pi",
            description: "A tiny single-board computer",
            base_cost: 10.0,
            cost_multiplier: 1.15,
            base_production: 0.5,
            level_bonus: 0.5,
            resource_type: ResourceType::Compute,
            unlock_threshold: 0.0,
            tier: 1,
        },
        BuildingDef {
            kind: BuildingKind::HomeRouter,
            name: "Home Router",
            description: "Basic network connectivity",
            base_cost: 15.0,
            cost_multiplier: 1.15,
            base_production: 0.3,
            level_bonus: 0.5,
            resource_type: ResourceType::Bandwidth,
            unlock_threshold: 0.0,
            tier: 1,
        },
        BuildingDef {
            kind: BuildingKind::USBDrive,
            name: "USB Drive",
            description: "Portable storage",
            base_cost: 20.0,
            cost_multiplier: 1.15,
            base_production: 0.2,
            level_bonus: 0.5,
            resource_type: ResourceType::Storage,
            unlock_threshold: 0.0,
            tier: 1,
        },
        // Tier 2
        BuildingDef {
            kind: BuildingKind::VPS,
            name: "VPS",
            description: "Virtual private server",
            base_cost: 100.0,
            cost_multiplier: 1.15,
            base_production: 4.0,
            level_bonus: 0.5,
            resource_type: ResourceType::Compute,
            unlock_threshold: 1_000.0,
            tier: 2,
        },
        BuildingDef {
            kind: BuildingKind::FiberConnection,
            name: "Fiber Connection",
            description: "High-speed fiber optic link",
            base_cost: 150.0,
            cost_multiplier: 1.15,
            base_production: 2.5,
            level_bonus: 0.5,
            resource_type: ResourceType::Bandwidth,
            unlock_threshold: 1_000.0,
            tier: 2,
        },
        BuildingDef {
            kind: BuildingKind::NASBox,
            name: "NAS Box",
            description: "Network-attached storage",
            base_cost: 200.0,
            cost_multiplier: 1.15,
            base_production: 1.5,
            level_bonus: 0.5,
            resource_type: ResourceType::Storage,
            unlock_threshold: 1_000.0,
            tier: 2,
        },
        // Tier 3
        BuildingDef {
            kind: BuildingKind::DedicatedServer,
            name: "Dedicated Server",
            description: "Full rack-mounted server",
            base_cost: 1_000.0,
            cost_multiplier: 1.15,
            base_production: 30.0,
            level_bonus: 0.5,
            resource_type: ResourceType::Compute,
            unlock_threshold: 100_000.0,
            tier: 3,
        },
        BuildingDef {
            kind: BuildingKind::LoadBalancer,
            name: "Load Balancer",
            description: "Distributes network traffic",
            base_cost: 1_500.0,
            cost_multiplier: 1.15,
            base_production: 20.0,
            level_bonus: 0.5,
            resource_type: ResourceType::Bandwidth,
            unlock_threshold: 100_000.0,
            tier: 3,
        },
        BuildingDef {
            kind: BuildingKind::SANArray,
            name: "SAN Array",
            description: "Storage area network",
            base_cost: 2_000.0,
            cost_multiplier: 1.15,
            base_production: 12.0,
            level_bonus: 0.5,
            resource_type: ResourceType::Storage,
            unlock_threshold: 100_000.0,
            tier: 3,
        },
        // Tier 4
        BuildingDef {
            kind: BuildingKind::ServerCluster,
            name: "Server Cluster",
            description: "Clustered compute nodes",
            base_cost: 10_000.0,
            cost_multiplier: 1.15,
            base_production: 200.0,
            level_bonus: 0.5,
            resource_type: ResourceType::Compute,
            unlock_threshold: 10_000_000.0,
            tier: 4,
        },
        BuildingDef {
            kind: BuildingKind::CDN,
            name: "CDN",
            description: "Content delivery network",
            base_cost: 15_000.0,
            cost_multiplier: 1.15,
            base_production: 130.0,
            level_bonus: 0.5,
            resource_type: ResourceType::Bandwidth,
            unlock_threshold: 10_000_000.0,
            tier: 4,
        },
        BuildingDef {
            kind: BuildingKind::DataWarehouse,
            name: "Data Warehouse",
            description: "Enterprise data storage",
            base_cost: 20_000.0,
            cost_multiplier: 1.15,
            base_production: 80.0,
            level_bonus: 0.5,
            resource_type: ResourceType::Storage,
            unlock_threshold: 10_000_000.0,
            tier: 4,
        },
        // Tier 5
        BuildingDef {
            kind: BuildingKind::Datacenter,
            name: "Datacenter",
            description: "Full-scale data center",
            base_cost: 100_000.0,
            cost_multiplier: 1.15,
            base_production: 1_500.0,
            level_bonus: 0.5,
            resource_type: ResourceType::Compute,
            unlock_threshold: 1_000_000_000.0,
            tier: 5,
        },
        BuildingDef {
            kind: BuildingKind::BackboneLink,
            name: "Backbone Link",
            description: "Internet backbone connection",
            base_cost: 150_000.0,
            cost_multiplier: 1.15,
            base_production: 1_000.0,
            level_bonus: 0.5,
            resource_type: ResourceType::Bandwidth,
            unlock_threshold: 1_000_000_000.0,
            tier: 5,
        },
        BuildingDef {
            kind: BuildingKind::ObjectStorage,
            name: "Object Storage",
            description: "Cloud object store (S3-like)",
            base_cost: 200_000.0,
            cost_multiplier: 1.15,
            base_production: 600.0,
            level_bonus: 0.5,
            resource_type: ResourceType::Storage,
            unlock_threshold: 1_000_000_000.0,
            tier: 5,
        },
        // Tier 6
        BuildingDef {
            kind: BuildingKind::CloudRegion,
            name: "Cloud Region",
            description: "Entire cloud availability zone",
            base_cost: 1_000_000.0,
            cost_multiplier: 1.15,
            base_production: 10_000.0,
            level_bonus: 0.5,
            resource_type: ResourceType::Compute,
            unlock_threshold: 1_000_000_000_000.0,
            tier: 6,
        },
        BuildingDef {
            kind: BuildingKind::SubmarineCable,
            name: "Submarine Cable",
            description: "Undersea fiber optic cable",
            base_cost: 1_500_000.0,
            cost_multiplier: 1.15,
            base_production: 7_000.0,
            level_bonus: 0.5,
            resource_type: ResourceType::Bandwidth,
            unlock_threshold: 1_000_000_000_000.0,
            tier: 6,
        },
        BuildingDef {
            kind: BuildingKind::DistributedFS,
            name: "Distributed FS",
            description: "Planet-scale filesystem",
            base_cost: 2_000_000.0,
            cost_multiplier: 1.15,
            base_production: 4_500.0,
            level_bonus: 0.5,
            resource_type: ResourceType::Storage,
            unlock_threshold: 1_000_000_000_000.0,
            tier: 6,
        },
        // Special buildings
        BuildingDef {
            kind: BuildingKind::CICDPipeline,
            name: "CI/CD Pipeline",
            description: "Automates all production (+10% global)",
            base_cost: 5_000.0,
            cost_multiplier: 1.20,
            base_production: 0.0, // Effect is global multiplier
            level_bonus: 0.0,
            resource_type: ResourceType::Compute,
            unlock_threshold: 50_000.0,
            tier: 3,
        },
        BuildingDef {
            kind: BuildingKind::MonitoringStack,
            name: "Monitoring Stack",
            description: "Generates bonus events",
            base_cost: 3_000.0,
            cost_multiplier: 1.20,
            base_production: 5.0,
            level_bonus: 0.5,
            resource_type: ResourceType::Compute,
            unlock_threshold: 25_000.0,
            tier: 2,
        },
        BuildingDef {
            kind: BuildingKind::CryptoMiner,
            name: "Crypto Miner",
            description: "Mines cryptocurrency",
            base_cost: 50_000.0,
            cost_multiplier: 1.20,
            base_production: 0.1,
            level_bonus: 0.5,
            resource_type: ResourceType::Crypto,
            unlock_threshold: 1_000_000_000.0,
            tier: 5,
        },
    ]
}
