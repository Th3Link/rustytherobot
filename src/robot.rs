use crate::direction::Direction;
use crate::position::Position;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

pub trait Movable {
    fn move_robot(&mut self, direction: Direction) -> Result<(), MovementError>;
}

#[derive(Debug)]
pub enum MovementError {
    TooFar,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Robot {
    name: String,
    position: Position,
    battery_level: i32,
}

impl Robot {
    pub fn new(name: String) -> Self {
        Self {
            name,
            position: Position::new(0, 0),
            battery_level: 100,
        }
    }

    pub fn rename(&mut self, new_name: &str) {
        info!(
            "robot name changeg from {from} to {to}",
            from = self.name,
            to = new_name
        );
        self.remove_file();
        self.name = String::from(new_name);
        self.store();
    }
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn remove_file(&self) {
        ProjectDirs::from("de", "marc", "rustytherobot")
            .map(|project_dirs| {
                project_dirs
                    .config_dir()
                    .join(format!("{robot_name}.json", robot_name = self.name))
            })
            .map(|path| {
                std::fs::remove_file(&path)
                    .inspect_err(|err| error!("cannot remove file {path:?}: {err}"))
                    .ok()
            });
    }

    pub fn would_collide(&self, direction: Direction, other_robot: &Robot) -> bool {
        let mut new_position = self.position.clone();
        if let Direction::Up(distance) = direction {
            new_position.y += distance;
        } else if direction == Direction::Down {
            new_position.y -= 1;
        } else if direction == Direction::Left {
            new_position.x -= 1;
        } else if direction == Direction::Right {
            new_position.x += 1;
        }

        new_position == other_robot.position
    }

    pub fn charge(&mut self, power: i32) {
        self.battery_level += power;
    }

    pub fn load(robot_name: String) -> Option<Self> {
        ProjectDirs::from("de", "marc", "rustytherobot")
            .map(|project_dirs| project_dirs.config_dir().join(format!("{robot_name}.json")))
            .and_then(|file| {
                std::fs::read_to_string(&file)
                    .inspect_err(|err| warn!("could not read file {file:?}: {err}"))
                    .ok()
            })
            .and_then(|data| {
                serde_json::from_str::<Robot>(&data)
                    .inspect_err(|err| warn!("could not deserialize: {err}"))
                    .ok()
            })
    }

    pub fn store(&self) {
        let config_dir = ProjectDirs::from("de", "marc", "rustytherobot")
            .map(|project_dirs| project_dirs.config_dir().to_path_buf());

        if let Some(config_dir) = config_dir.as_ref() {
            let data = serde_json::to_string_pretty(self).unwrap();
            std::fs::create_dir_all(config_dir)
                .inspect_err(|err| error!("could not create dir {config_dir:?}: {err}"))
                .ok();
            let path = config_dir.join(format!("{robot_name}.json", robot_name = self.name));
            std::fs::write(&path, data)
                .inspect_err(|err| error!("cannot write file {path:?}: {err}"))
                .ok();
        }
    }
}
impl Movable for Robot {
    fn move_robot(&mut self, direction: Direction) -> Result<(), MovementError> {
        match direction {
            Direction::Up(distance) => {
                if distance >= 3 {
                    return Err(MovementError::TooFar);
                }
                self.position.y += distance;
                self.battery_level -= distance;
            }
            Direction::Down => {
                self.position.y -= 1;
                self.battery_level -= 1;
            }
            Direction::Left => {
                self.position.x -= 1;
                self.battery_level -= 1;
            }
            Direction::Right => {
                self.position.x += 1;
                self.battery_level -= 1;
            }
        }
        Ok(())
    }
}

impl std::fmt::Display for Robot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "My id is {}. I am on position {}. My battery level is at {}",
            self.name, self.position, self.battery_level
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_move_robot() {
        let mut robot = Robot::new(String::from("one"));
        assert!(robot.move_robot(Direction::Down).is_ok());
        assert_eq!(robot.position.y, -1);

        for _i in 0..200 {
            assert!(robot.move_robot(Direction::Up(1)).is_ok());
        }
        assert_eq!(robot.position.y, 199);
    }
}
