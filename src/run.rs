use crate::cli::{Cli, Command, MoveCommand, MoveDirection, RenameCommand, Up};
use crate::config::Config;
use crate::direction::Direction;
use crate::robot::Movable;
use crate::robot::Robot;
use tracing::{error, info, warn};
pub fn run(cli: Cli) {
    info!("run with cli {cli:?}", cli = cli);

    let _config = Config::load()
        .inspect_err(|err| warn!("{err}"))
        .unwrap_or_default();

    // load state
    let robot_name = cli.robot_name;
    let robot = Robot::load(robot_name.clone());

    match cli.command {
        Command::Info(_) => robot.map_or_else(
            |err| error!("robot {robot_name} does not exist: {err}"),
            |robot| println!("Robot position: {robot}"),
        ),
        Command::Rename(RenameCommand { new_name }) => {
            robot.map_or_else(
                |err| error!("robot {robot_name} does not exist: {err}"),
                |mut robot| {
                    robot
                        .rename(&new_name)
                        .inspect_err(|err| {
                            error!("could not rename robot {robot_name} to {new_name}: {err}")
                        })
                        .ok();
                },
            );
        }
        Command::Delete(_) => {
            robot.map_or_else(
                |err| error!("robot {robot_name} does not exist: {err}"),
                |robot| robot.remove_file(),
            );
        }
        Command::Create(_) => {
            if robot.is_ok() {
                warn!("Could not create robot: already exist");
            } else {
                Robot::new(robot_name.clone())
                    .store()
                    .inspect_err(|err| error!("could not store robot {robot_name}: {err}"))
                    .ok();
            }
        }
        Command::Move(MoveCommand { direction }) => {
            let mut robot = robot.unwrap_or(Robot::new(robot_name));
            match direction {
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
            }
            println!("New robot position: {robot}");
            robot
                .store()
                .inspect_err(|err| {
                    error!(
                        "could not store robot {robot_name}: {err}",
                        robot_name = robot.name()
                    )
                })
                .ok();
        }
    }
}
