use crate::cli::{Cli, Command, MoveCommand, MoveDirection, RenameCommand, Up};
use crate::direction::Direction;
use crate::robot::Movable;
use crate::robot::Robot;
use tracing::{info, warn};

pub fn run(cli: Cli) {
    info!("run with cli {cli:?}", cli = cli);
    let mut robot = Robot::new(cli.robot_name);
    info!("{robot}");
    match cli.command {
        Command::Info(_) => {
            println!("Robot position: {robot}")
        }
        Command::Rename(RenameCommand { new_name }) => {
            robot.rename(&new_name);
        }
        Command::Move(MoveCommand { direction }) => match direction {
            MoveDirection::Up(Up { steps }) => {
                robot
                    .move_robot(Direction::Up(steps))
                    .inspect_err(|err| warn!("Could not move robot: {err:?}"))
                    .ok();
            }
            MoveDirection::Down(_) => {
                robot
                    .move_robot(Direction::Down)
                    .inspect_err(|err| warn!("Could not move robot: {err:?}"))
                    .ok();
            }
            MoveDirection::Left(_) => {
                robot
                    .move_robot(Direction::Left)
                    .inspect_err(|err| warn!("Could not move robot: {err:?}"))
                    .ok();
            }
            MoveDirection::Right(_) => {
                robot
                    .move_robot(Direction::Right)
                    .inspect_err(|err| warn!("Could not move robot: {err:?}"))
                    .ok();
            }
        },
    }
    println!("New robot position: {robot}");
}
