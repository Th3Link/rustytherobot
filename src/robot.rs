use crate::direction::Direction;
use crate::position::Position;
use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use tracing::info;

pub const MAX_BATTERY_LEVEL: i32 = 10;
const MAX_HEALTH: i32 = 30;
const REPAIR_COST: i32 = 3;

pub trait Movable {
    fn move_robot(&mut self, direction: Direction) -> Result<(), MovementError>;
}

#[derive(Debug, thiserror::Error)]
pub enum MovementError {
    #[error("movement distance is too far")]
    TooFar,
    #[error("battery is empty cannot move")]
    BatteryEmpty,
}

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub name: String,
    pub position: Position,
    pub battery_level: i32,
    pub health: i32,
    pub team_id: i32,
}

impl From<&Robot> for Snapshot {
    fn from(robot: &Robot) -> Self {
        Self {
            name: robot.name.clone(),
            position: robot.position.clone(),
            battery_level: robot.battery_level,
            health: robot.health,
            team_id: robot.team_id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Robot {
    name: String,
    position: Position,
    battery_level: i32,
    health: i32,
    team_id: i32,
}

impl Robot {
    pub fn new(name: String, team_id: i32) -> Self {
        Self {
            name,
            position: Position::new(0, 0),
            battery_level: MAX_BATTERY_LEVEL,
            health: MAX_HEALTH,
            team_id,
        }
    }

    pub fn rename(&mut self, new_name: &str) {
        info!(
            "robot name changeg from {from} to {to}",
            from = self.name,
            to = new_name
        );
        self.name = String::from(new_name);
    }

    pub fn position(&self) -> &Position {
        &self.position
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn team_id(&self) -> i32 {
        self.team_id
    }

    pub fn recharge(&mut self, battery_level: i32, team_id: i32) {
        self.battery_level = std::cmp::max(MAX_BATTERY_LEVEL, self.battery_level + battery_level);
        if battery_level < MAX_BATTERY_LEVEL / 2 {
            self.team_id = team_id;
        }
    }

    pub fn charge_transfer(&mut self, robot_count: i32) -> i32 {
        if robot_count == 0 {
            return 0;
        }
        let charge = (self.battery_level as f64 / robot_count as f64).ceil() as i32;
        self.battery_level = std::cmp::min(charge, MAX_BATTERY_LEVEL);
        charge
    }

    pub fn perform_repair(&mut self) -> Result<()> {
        if self.battery_level < REPAIR_COST {
            bail!("not enough energy to perform repair");
        }
        self.battery_level -= REPAIR_COST;
        Ok(())
    }

    pub fn repair(&mut self, team_id: i32) -> bool {
        if team_id == self.team_id {
            self.health = MAX_HEALTH;
            true
        } else if self.health == 0 {
            self.health = MAX_HEALTH;
            self.team_id = team_id;
            true
        } else {
            false
        }
    }

    pub fn take_damage(&mut self, damage: i32) {
        self.health = std::cmp::min(damage, self.health);
    }

    pub fn attack(&mut self, power: i32) -> i32 {
        let power = std::cmp::min(power, self.battery_level);
        self.battery_level -= power;
        power
    }

    pub fn path(&self, direction: Direction) -> Vec<Position> {
        match direction {
            Direction::Up(distance) => (1..=distance)
                .map(|step| Position::new(self.position.x, self.position.y + step))
                .collect(),

            Direction::Down => {
                vec![Position::new(self.position.x, self.position.y - 1)]
            }

            Direction::Left => {
                vec![Position::new(self.position.x - 1, self.position.y)]
            }

            Direction::Right => {
                vec![Position::new(self.position.x + 1, self.position.y)]
            }
        }
    }

    //    pub fn charge(&mut self, power: i32) {
    //        self.battery_level += power;
    //    }
}
impl Movable for Robot {
    fn move_robot(&mut self, direction: Direction) -> Result<(), MovementError> {
        if self.battery_level <= 0 {
            return Err(MovementError::BatteryEmpty);
        }

        match direction {
            Direction::Up(distance) => {
                if distance > 3 {
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
                self.position.x -= 1;
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
        let mut robot = Robot::new(String::from("one"), 1);
        assert!(robot.move_robot(Direction::Down).is_ok());
        assert_eq!(robot.position.y, -1);

        for _i in 0..200 {
            assert!(robot.move_robot(Direction::Up(1)).is_ok());
        }
        assert_eq!(robot.position.y, 199);
    }
}
