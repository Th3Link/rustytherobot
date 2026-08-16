use crate::robot::{self, Robot};
use anyhow::{Context, Result, anyhow};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use tracing::info;

pub mod wall;

const WORLD_DATA_FILE: &str = "world_state.json";

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct World {
    robots: Vec<Robot>,
}

impl World {
    pub fn robot(&self, robot_name: &str) -> Option<&Robot> {
        for robot in &self.robots {
            if robot.name() == robot_name {
                return Some(robot);
            }
        }
        None
    }

    pub fn robot_mut(&mut self, robot_name: &str) -> Option<&mut Robot> {
        for robot in &mut self.robots {
            if robot.name() == robot_name {
                return Some(robot);
            }
        }
        None
    }

    pub fn remove_robot(&mut self, robot_name: &str) -> Result<()> {
        for i in 0..self.robots.len() {
            if self.robots[i].name() == robot_name {
                self.robots.remove(i);
                return Ok(());
            }
        }
        Err(anyhow!("robot with name {robot_name} not found"))
    }

    pub fn add_robot(&mut self, robot: Robot) {
        self.robots.push(robot);
    }

    pub fn load() -> Result<Self> {
        let file = ProjectDirs::from("de", "marc", "rustytherobot")
            .context("could not determine configuration directory")?
            .data_dir()
            .join(WORLD_DATA_FILE);

        let data = std::fs::read_to_string(&file)
            .with_context(|| format!("could not read robot file {file:?}"))?;

        let world = serde_json::from_str::<World>(&data)
            .with_context(|| format!("could not deserialize robot {file:?}"))?;
        info!("loaded world from {file:?}");
        Ok(world)
    }

    pub fn store(&self) -> Result<()> {
        let data_dir = ProjectDirs::from("de", "marc", "rustytherobot")
            .context("could not determine configuration directory")?
            .data_dir()
            .to_path_buf();

        std::fs::create_dir_all(&data_dir)
            .with_context(|| format!("could not create directory {data_dir:?}"))?;

        let data = serde_json::to_string_pretty(self).context("could not serialize robot")?;

        let path = data_dir.join(WORLD_DATA_FILE);

        std::fs::write(&path, data)
            .with_context(|| format!("could not write robot file {path:?}"))?;

        Ok(())
    }
}
impl std::fmt::Display for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "World contains {} robots:", self.robots.len())?;

        for robot in &self.robots {
            writeln!(f, "  {robot}")?;
        }

        Ok(())
    }
}
