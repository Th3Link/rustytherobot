use crate::config::Config;
use crate::position::Position;
use crate::robot::Robot;
use anyhow::{Context, Result, anyhow};
use directories::ProjectDirs;
use rand::rng;
use rand::seq::SliceRandom;
use ron::ser::to_string_pretty;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tile::Tile;
use tracing::info;

pub mod tile;

const WORLD_DATA_FILE: &str = "world_state.ron";

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct World {
    robots: Vec<Robot>,
    tiles: HashMap<Position, Tile>,
}

impl World {
    pub fn new(config: Config) -> Self {
        let mut world = Self::default();
        let min = Position::new(-config.world_size.x / 2, -config.world_size.y / 2);
        let max = Position::new(
            config.world_size.x + min.x - 1,
            config.world_size.y + min.y - 1,
        ); // correct by 1 because we have a position 0/0... for 5x5 we have # -2 -1 0 1 2 #

        let left = min.x - 1;
        let right = max.x + 1;
        let bottom = min.y - 1;
        let top = max.y + 1;

        for x in left..=right {
            world.tiles.insert(Position::new(x, bottom), Tile::Wall);
            world.tiles.insert(Position::new(x, top), Tile::Wall);
        }

        for y in bottom..=top {
            world.tiles.insert(Position::new(left, y), Tile::Wall);
            world.tiles.insert(Position::new(right, y), Tile::Wall);
        }

        /* alternative (but is ir worth it?):
        let positions = (left..=right)
            .flat_map(|x| [Position::new(x, bottom), Position::new(x, top)])
            .chain((bottom..=top).flat_map(|y| [Position::new(left, y), Position::new(right, y)]));

        for position in positions {
            world.tiles.insert(position, Tile::Wall);
        }
        */

        // Charging Pads

        let field_count = config.world_size.x * config.world_size.y;
        let charging_pad_count = (field_count as f64 * config.chargingpad_ratio).round() as usize;
        world.insert_charging_pads(min, max, charging_pad_count);
        world
    }

    fn insert_charging_pads(&mut self, min: Position, max: Position, count: usize) {
        let mut positions = Vec::new();

        for x in min.x..=max.x {
            for y in min.y..=max.y {
                positions.push(Position::new(x, y));
            }
        }
        /* alternative:
        positions.extend(
            (min.x..=max.x)
                .flat_map(|x| (min.y..=max.y).map(move |y| Position::new(x, y))),
        );
        */
        let mut rng = rng();
        positions.shuffle(&mut rng);
        positions.into_iter().take(count).for_each(|cp| {
            self.tiles.insert(cp, Tile::ChargingPad);
        });
        /* alternative:
        for cp in positions.into_iter().take(count) {
            self.tiles.insert(cp, Tile::ChargingPad);
        }
        */
    }

    pub fn robot(&self, robot_name: &str) -> Option<&Robot> {
        self.robots
            .iter()
            .find(|&robot| robot.name() == robot_name)
            .map(|v| v as _)
    }

    pub fn robot_mut(&mut self, robot_name: &str) -> Option<&mut Robot> {
        self.robots
            .iter_mut()
            .find(|robot| robot.name() == robot_name)
            .map(|v| v as _)
    }

    pub fn remove_robot(&mut self, robot_name: &str) -> Result<()> {
        let pos = self
            .robots
            .iter()
            .position(|robot| robot.name() == robot_name)
            .ok_or_else(|| anyhow!("robot with name {robot_name} not found"))?;
        self.robots.remove(pos);
        Ok(())
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
            .with_context(|| format!("could not read world file {file:?}"))?;

        let world = ron::from_str::<World>(&data)
            .with_context(|| format!("could not deserialize world {file:?}"))?;
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

        let data = to_string_pretty(self, ron::ser::PrettyConfig::default())
            .context("could not serialize world")?;

        let path = data_dir.join(WORLD_DATA_FILE);

        std::fs::write(&path, data)
            .with_context(|| format!("could not write world file {path:?}"))?;

        Ok(())
    }
    fn draw_tiles(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let min_x = self.tiles.keys().map(|p| p.x).min().unwrap_or(0);
        let max_x = self.tiles.keys().map(|p| p.x).max().unwrap_or(0);
        let min_y = self.tiles.keys().map(|p| p.y).min().unwrap_or(0);
        let max_y = self.tiles.keys().map(|p| p.y).max().unwrap_or(0);

        for y in (min_y..=max_y).rev() {
            for x in min_x..=max_x {
                write!(f, "{} ", self.symbol_at(Position::new(x, y)))?;
            }

            writeln!(f)?;
        }

        Ok(())
    }
    fn symbol_at(&self, position: Position) -> char {
        if self
            .robots
            .iter()
            .any(|robot| robot.position() == &position)
        {
            return 'R';
        }

        match self.tiles.get(&position) {
            Some(Tile::Wall) => '#',
            Some(Tile::ChargingPad) => 'C',
            None => '.',
        }
    }
}

impl std::fmt::Display for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        self.draw_tiles(f)?;
        writeln!(f)?;
        writeln!(f, "World contains {} robots:", self.robots.len())?;

        for robot in &self.robots {
            writeln!(f, "  {robot}")?;
        }

        Ok(())
    }
}
