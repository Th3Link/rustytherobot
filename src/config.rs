use crate::dimension::Dimension;
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
    pub fn load() -> Result<Self, ()> {
        ProjectDirs::from("de", "marc", "rustytherobot")
            .map(|project_dirs| project_dirs.config_dir().join(CONFIG_FILENAME))
            .ok_or(())
            .and_then(|file| {
                std::fs::read_to_string(&file)
                    .inspect_err(|err| warn!("could not read file {file:?}: {err}"))
                    .map_err(|_err| ())
            })
            .and_then(|data| {
                ron::from_str::<Config>(&data)
                    .inspect_err(|err| warn!("could not deserialize: {err}"))
                    .map_err(|_err| ())
            })
    }
}
