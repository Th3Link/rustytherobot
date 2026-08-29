use crate::cli::{self, AttackCommand, Cli, Command, MoveCommand, RenameCommand, RobotCommand};
use crate::config::Config;
use crate::world::World;
use anyhow::anyhow;

use tracing::{error, info, warn};

pub fn run(cli: Cli) -> anyhow::Result<()> {
    info!("run with cli {cli:?}", cli = cli);

    let config = Config::load()
        .inspect_err(|err| warn!("{err}"))
        .unwrap_or_default();

    // load state
    let mut world = World::load()
        .inspect_err(|err| error!("could not load world: {err}"))
        .unwrap_or(World::new(config));

    match cli.command {
        Command::Robot(cli::Robot {
            robot_name,
            robot_command,
        }) => handle_robot(robot_name, &mut world, robot_command),
        Command::DBus(_) => {
            let robot = crate::dbus::Robot::new(world);
            let _handle = zbus::blocking::connection::Builder::session()?
                .name("dev.luehr.rustytherobot")?
                .serve_at("/", robot)?
                .build()?;

            loop {
                std::thread::sleep(core::time::Duration::from_secs(1));
            }
        }
    }
}

fn handle_robot(
    robot_name: String,
    world: &mut World,
    robot_command: RobotCommand,
) -> anyhow::Result<()> {
    let robot = world
        .robot_mut(&robot_name)
        .ok_or_else(|| anyhow!("Robot {robot_name} not found in world"))?;
    match robot_command {
        RobotCommand::Info(_) => println!("Robot position: {robot}"),
        RobotCommand::Rename(RenameCommand { new_name }) => robot.rename(&new_name),
        RobotCommand::Delete(_) => {
            world
                .remove_robot(&robot_name)
                .inspect_err(|err| error!("robot {robot_name} could not be deleted: {err}"))
                .ok();
        }
        RobotCommand::Create(_) => {
            if world.robot(&robot_name).is_some() {
                warn!("Could not create robot: already exist");
            } else {
                world.add_robot(robot_name.clone());
            }
        }
        RobotCommand::Move(MoveCommand { direction }) => {
            world
                .execute_move(&robot_name, direction.into())
                .inspect_err(|err| error!("{err}"))
                .ok();
        }
        RobotCommand::Attack(AttackCommand { power }) => {
            world
                .execute_attack(&robot_name, power)
                .inspect_err(|err| error!("{err}"))
                .ok();
        }
        RobotCommand::TransferCharge(_) => {
            world
                .execute_charge_transfer(&robot_name)
                .inspect_err(|err| error!("{err}"))
                .ok();
        }
        RobotCommand::Repair(_) => {
            world
                .execute_repair(&robot_name)
                .inspect_err(|err| error!("{err}"))
                .ok();
        }
    }
    world
        .store()
        .inspect_err(|err| error!("could not store world state: {err}",))
        .ok();
    info!("world state: {world}");
    Ok(())
}
