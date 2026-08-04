use crate::dimension::Dimension;
use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use tracing::warn;

const CONFIG_FILENAME: &str = "config.ron";
const DEFAULT_WORLD_SIZE: Dimension = Dimension { x: 100, y: 100 };
const DEFAULT_WALL_RATIO: f64 = 0.1;
const DEFAULT_CARGINGPAD_RATIO: f64 = 0.05;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    world_size: Dimension,
    wall_ratio: f64,
    chargingpad_ratio: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            world_size: DEFAULT_WORLD_SIZE,
            wall_ratio: DEFAULT_WALL_RATIO,
            chargingpad_ratio: DEFAULT_CARGINGPAD_RATIO,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let file = ProjectDirs::from("de", "marc", "rustytherobot")
            .context("Could not determine configuration directory")?
            .config_dir()
            .join(CONFIG_FILENAME);

        let data = std::fs::read_to_string(&file)
            .with_context(|| format!("Could not read configuration file {file:?}"))?;

        let config =
            ron::from_str::<Config>(&data).context("Failed to deserialize configuration")?;

        Ok(config)
    }
}
