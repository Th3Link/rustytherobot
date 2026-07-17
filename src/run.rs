use crate::cli::{Cli, Command, MoveCommand, MoveDirection, RenameCommand};
use crate::direction::Direction;
use crate::robot::Movable;
use crate::robot::Robot;
use tracing::{info, warn};

pub fn run(cli: Cli) {
    info!("run with cli {cli:?}");

    let mut robot = Robot::new(cli.robot_name);
    info!("{robot}");

    match cli.command {
        Command::Info => {
            println!("Robot position: {robot}");
        }

        Command::Rename(RenameCommand { new_name }) => {
            robot.rename(&new_name);
        }

        Command::Move(MoveCommand { direction }) => {
            let result = match direction {
                MoveDirection::Up { steps } => robot.move_robot(Direction::Up(steps)),
                MoveDirection::Down => robot.move_robot(Direction::Down),
                MoveDirection::Left => robot.move_robot(Direction::Left),
                MoveDirection::Right => robot.move_robot(Direction::Right),
            };

            result
                .inspect_err(|err| warn!("Could not move robot: {err:?}"))
                .ok();
        }
    }

    println!("New robot position: {robot}");
}
