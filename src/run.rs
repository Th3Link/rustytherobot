use crate::cli::{self, AttackCommand, Cli, Command, MoveCommand, RenameCommand, RobotCommand};
use crate::config::Config;
use crate::dbus::{Robot, RobotSignals};
use crate::robot::Snapshot;
use crate::world::World;
use anyhow::anyhow;
use smol::channel;
use std::future::pending;
use strum::{Display, EnumString};
use tracing::{error, info, warn};
use zbus::object_server::InterfaceRef;

#[derive(EnumString, Display)]
pub enum Event {
    Changed,
    Rename,
    Removed,
    Created,
}

pub async fn run(cli: Cli) -> anyhow::Result<()> {
    info!("run with cli {cli:?}", cli = cli);

    let config = Config::load()
        .await
        .inspect_err(|err| warn!("{err}"))
        .unwrap_or_default();

    let (command_channel_tx, command_channel_rx) = channel::bounded(20);

    // load state
    let mut world = World::load()
        .await
        .inspect_err(|err| error!("could not load world: {err}"))
        .unwrap_or(World::new(config));

    match cli.command {
        Command::Robot(cli::Robot {
            robot_name,
            robot_command,
        }) => {
            handle_robot(robot_name, &mut world, robot_command)?;
            world
                .store()
                .await
                .inspect_err(|err| error!("could not store world state: {err}",))?;
            Ok(())
        }
        Command::DBus(_) => {
            let robot = Robot::new(command_channel_tx);
            let connection = zbus::connection::Builder::session()
                .name("dev.luehr.rustytherobot")
                .serve_at("/", robot)
                .build()
                .await?;
            info!("world state: {world}");
            let object_server: InterfaceRef<Robot> =
                connection.object_server().interface("/").await?;
            smol::spawn(async move {
                while let Ok(crate::dbus::Message {
                    robot_name,
                    command,
                    return_channel,
                }) = command_channel_rx.recv().await
                {
                    match command {
                        RobotCommand::Info(_) => {
                            let Some(robot) = world.robot(&robot_name) else {
                                break;
                            };
                            let Some(return_channel) = return_channel else {
                                break;
                            };
                            return_channel.send(Snapshot::from(robot)).await.ok();
                            info!("Robot position: {robot}");
                        }
                        _ => match handle_robot(robot_name, &mut world, command) {
                            Ok(snapshots) => {
                                for (e, s) in snapshots {
                                    object_server
                                        .signal_emitter()
                                        .robot_state(
                                            &e.to_string(),
                                            &s.name,
                                            s.position.x,
                                            s.position.y,
                                            s.battery_level,
                                            s.health,
                                            s.team_id,
                                        )
                                        .await
                                        .inspect_err(|err| error!("could not send signal: {err}"))
                                        .ok();
                                }
                            }
                            Err(err) => {
                                warn!("handle_robot failed: {err}");
                            }
                        },
                    }
                }
            })
            .detach();

            // endless loop
            pending::<()>().await;
            info!("end run");
            Ok(())
        }
    }
}

fn handle_robot(
    robot_name: String,
    world: &mut World,
    robot_command: RobotCommand,
) -> anyhow::Result<Vec<(Event, Snapshot)>> {
    let snapshots = match robot_command {
        RobotCommand::Info(_) => {
            let robot = world
                .robot(&robot_name)
                .ok_or_else(|| anyhow!("Robot {robot_name} not found in world"))?;
            println!("Robot position: {robot}");
            vec![]
        }
        RobotCommand::Rename(RenameCommand { new_name }) => {
            let robot = world
                .robot_mut(&robot_name)
                .ok_or_else(|| anyhow!("Robot {robot_name} not found in world"))?;
            let s1 = Snapshot::from(&*robot);
            robot.rename(&new_name);
            vec![
                (Event::Removed, s1),
                (Event::Rename, Snapshot::from(&*robot)),
            ]
        }
        RobotCommand::Delete(_) => {
            let robot = world
                .robot(&robot_name)
                .ok_or_else(|| anyhow!("Robot {robot_name} not found in world"))?;
            let s1 = Snapshot::from(robot);
            world
                .remove_robot(&robot_name)
                .inspect_err(|err| error!("robot {robot_name} could not be deleted: {err}"))
                .ok();
            vec![(Event::Removed, s1)]
        }
        RobotCommand::Create(_) => {
            if world.robot(&robot_name).is_some() {
                warn!("Could not create robot: already exist");
            } else {
                world.add_robot(robot_name.clone());
            }
            vec![(
                Event::Created,
                world
                    .robot(&robot_name)
                    .ok_or_else(|| anyhow!("Robot {robot_name} not found in world"))?
                    .into(),
            )]
        }
        RobotCommand::Move(MoveCommand { direction }) => vec![(
            Event::Changed,
            world
                .execute_move(&robot_name, direction.into())
                .inspect_err(|err| error!("{err}"))?,
        )],
        RobotCommand::Attack(AttackCommand { power }) => world
            .execute_attack(&robot_name, power)
            .inspect_err(|err| error!("{err}"))?
            .into_iter()
            .map(|snapshot| (Event::Changed, snapshot))
            .collect(),
        RobotCommand::TransferCharge(_) => world
            .execute_charge_transfer(&robot_name)
            .inspect_err(|err| error!("{err}"))?
            .into_iter()
            .map(|snapshot| (Event::Changed, snapshot))
            .collect(),
        RobotCommand::Repair(_) => world
            .execute_repair(&robot_name)
            .inspect_err(|err| error!("{err}"))?
            .into_iter()
            .map(|snapshot| (Event::Changed, snapshot))
            .collect(),
    };
    info!("world state: {world}");
    Ok(snapshots)
}
