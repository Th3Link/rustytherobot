use crate::direction::Direction;
use crate::position::Position;

pub trait Movable {
    fn move_robot(&mut self, direction: Direction) -> Result<(), MovementError>;
}

#[derive(Debug)]
pub enum MovementError {
    TooFar,
}

pub struct Robot {
    robot_id: u32,
    position: Position,
    battery_level: i32,
}

impl Robot {
    pub fn new(robot_id: u32) -> Self {
        Self {
            robot_id,
            position: Position::new(0, 0),
            battery_level: 100,
        }
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
            self.robot_id, self.position, self.battery_level
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_move_robot() {
        let mut robot = Robot::new(1);
        assert!(robot.move_robot(Direction::Down).is_ok());
        assert_eq!(robot.position.y, -1);

        for _i in 0..200 {
            assert!(robot.move_robot(Direction::Up(1)).is_ok());
        }
        assert_eq!(robot.position.y, 199);
    }
}
