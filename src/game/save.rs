use chrono::{DateTime, Utc};
use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::state::GameState;

const SAVE_FILE: &str = "idle_terminal_save.json";

#[derive(Serialize, Deserialize)]
struct SaveData {
    pub game_state: GameState,
    pub save_time: DateTime<Utc>,
    pub version: u32,
}

const SAVE_VERSION: u32 = 1;

pub fn save_path() -> PathBuf {
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("idle-terminal");
    std::fs::create_dir_all(&data_dir).ok();
    data_dir.join(SAVE_FILE)
}

pub fn save_game(state: &GameState) -> Result<()> {
    let save_data = SaveData {
        game_state: state.clone(),
        save_time: Utc::now(),
        version: SAVE_VERSION,
    };

    let json = serde_json::to_string_pretty(&save_data)?;
    let path = save_path();
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, &json)?;
    std::fs::rename(&tmp, &path)?;

    tracing::debug!("Game saved to {:?}", path);
    Ok(())
}

pub struct LoadResult {
    pub state: GameState,
    pub offline_ticks: u64,
    pub offline_earnings: super::resources::Resources,
}

pub fn load_game() -> Result<Option<LoadResult>> {
    let path = save_path();
    if !path.exists() {
        return Ok(None);
    }

    let json = std::fs::read_to_string(&path)?;
    let save_data: SaveData = serde_json::from_str(&json)?;

    let mut state = save_data.game_state;

    // Calculate offline progression
    let now = Utc::now();
    let elapsed = now - save_data.save_time;
    let elapsed_ms = elapsed.num_milliseconds().max(0) as u64;
    let missed_ticks = elapsed_ms / 250; // 4Hz game tick

    // Cap offline ticks at 8 hours = 115,200 ticks
    let offline_ticks = missed_ticks.min(115_200);

    // Record resources before offline progression
    let resources_before = state.resources.clone();

    // Apply offline production at reduced rate
    let efficiency = state.offline_efficiency;
    let mut offline_production = state.production_per_tick.clone();
    offline_production.compute *= efficiency;
    offline_production.bandwidth *= efficiency;
    offline_production.storage *= efficiency;
    offline_production.crypto *= efficiency;

    for _ in 0..offline_ticks {
        state.resources.add(&offline_production);
        state.total_ticks += 1;
    }

    // Calculate earnings
    let offline_earnings = super::resources::Resources {
        compute: state.resources.compute - resources_before.compute,
        bandwidth: state.resources.bandwidth - resources_before.bandwidth,
        storage: state.resources.storage - resources_before.storage,
        reputation: 0.0,
        crypto: state.resources.crypto - resources_before.crypto,
    };

    Ok(Some(LoadResult {
        state,
        offline_ticks,
        offline_earnings,
    }))
}

pub fn delete_save() -> Result<()> {
    let path = save_path();
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_path_is_valid() {
        let path = save_path();
        assert!(path.ends_with(SAVE_FILE));
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        // Use a temp dir to avoid polluting the real save location
        let dir = std::env::temp_dir().join("idle_terminal_test");
        std::fs::create_dir_all(&dir).ok();
        let path = dir.join("test_save.json");

        let state = GameState::new();
        let save_data = SaveData {
            game_state: state.clone(),
            save_time: Utc::now(),
            version: SAVE_VERSION,
        };

        let json = serde_json::to_string_pretty(&save_data).unwrap();
        std::fs::write(&path, &json).unwrap();

        let loaded_json = std::fs::read_to_string(&path).unwrap();
        let loaded: SaveData = serde_json::from_str(&loaded_json).unwrap();

        assert_eq!(loaded.game_state.resources.compute, state.resources.compute);
        assert_eq!(loaded.version, SAVE_VERSION);

        // Cleanup
        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(&dir).ok();
    }
}
