use crate::cli::{Cli, Command, MoveCommand, RenameCommand};
use crate::config::Config;
use crate::robot::Movable;
use crate::robot::Robot;
use crate::world::World;
use anyhow::anyhow;

use tracing::{error, info, warn};

pub fn run(cli: Cli) {
    info!("run with cli {cli:?}", cli = cli);

    let config = Config::load()
        .inspect_err(|err| warn!("{err}"))
        .unwrap_or_default();

    // load state
    let robot_name = cli.robot_name;
    let mut world = World::load()
        .inspect_err(|err| error!("could not load world: {err}"))
        .unwrap_or(World::new(config));

    match cli.command {
        Command::Info(_) => {
            let robot = world
                .robot(&robot_name)
                .ok_or_else(|| anyhow!("Robot not found in world"));
            robot.map_or_else(
                |err| error!("robot {robot_name} does not exist: {err}"),
                |robot| println!("Robot position: {robot}"),
            )
        }
        Command::Rename(RenameCommand { new_name }) => {
            let robot = world
                .robot_mut(&robot_name)
                .ok_or_else(|| anyhow!("Robot not found in world"));
            robot.map_or_else(
                |err| error!("robot {robot_name} does not exist: {err}"),
                |robot| robot.rename(&new_name),
            );
        }
        Command::Delete(_) => {
            world
                .remove_robot(&robot_name)
                .inspect_err(|err| error!("robot {robot_name} could not be deleted: {err}"))
                .ok();
        }
        Command::Create(_) => {
            if world.robot(&robot_name).is_some() {
                warn!("Could not create robot: already exist");
            } else {
                world.add_robot(Robot::new(robot_name.clone()));
            }
        }
        Command::Move(MoveCommand { direction }) => {
            let robot = world
                .robot_mut(&robot_name)
                .ok_or_else(|| anyhow!("Robot not found in world"));
            robot.map_or_else(
                |err| error!("robot {robot_name} does not exist: {err}"),
                |robot| {
                    robot
                        .move_robot(direction.into())
                        .inspect(|_| println!("New robot position: {robot}"))
                        .inspect_err(|err| error!("could not move robot {robot_name}: {err:?}"))
                        .ok();
                },
            );
        }
    }
    world
        .store()
        .inspect_err(|err| error!("could not store world state: {err}",))
        .ok();
    info!("world state: {world}");
}
