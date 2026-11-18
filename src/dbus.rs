#![allow(clippy::too_many_arguments)]

use crate::cli::*;
use crate::dimension::Dimension;
use crate::position::Position;
use crate::robot::Snapshot;
use zbus::object_server::SignalEmitter;

pub enum Message {
    Robot(RobotMessage),
    World(WorldMessage),
}

pub struct RobotMessage {
    pub robot_name: String,
    pub command: RobotCommand,
    pub return_channel: Option<smol::channel::Sender<Snapshot>>,
}

impl RobotMessage {
    pub fn new_command(robot_name: String, command: RobotCommand) -> Self {
        Self {
            robot_name,
            command,
            return_channel: None,
        }
    }
    pub fn new_request(
        robot_name: String,
        command: RobotCommand,
        return_channel: Option<smol::channel::Sender<Snapshot>>,
    ) -> Self {
        Self {
            robot_name,
            command,
            return_channel,
        }
    }
}

pub enum WorldReturn {
    RobotList(Vec<String>),
    BoardDimensions(Dimension),
    ChargepadList(Vec<Position>),
}

pub struct WorldMessage {
    pub command: WorldCommand,
    pub return_channel: Option<smol::channel::Sender<WorldReturn>>,
}

impl WorldMessage {
    pub fn new_request(
        command: WorldCommand,
        return_channel: Option<smol::channel::Sender<WorldReturn>>,
    ) -> Self {
        Self {
            command,
            return_channel,
        }
    }
}

pub struct Robot {
    sender: smol::channel::Sender<RobotMessage>,
}

impl Robot {
    pub fn new(sender: smol::channel::Sender<RobotMessage>) -> Self {
        Self { sender }
    }
}

pub struct World {
    sender: smol::channel::Sender<WorldMessage>,
}

impl World {
    pub fn new(sender: smol::channel::Sender<WorldMessage>) -> Self {
        Self { sender }
    }
}

#[zbus::interface(name = "dev.luehr.rustytherobot.Robot")]
impl Robot {
    async fn create(&self, name: &str) -> bool {
        tracing::info!("create {name} called");
        self.sender
            .send(RobotMessage::new_command(
                name.to_owned(),
                RobotCommand::Create(CreateCommand {}),
            ))
            .await
            .is_ok()
    }

    async fn delete(&self, name: &str) -> bool {
        tracing::info!("delete {name} called");
        self.sender
            .send(RobotMessage::new_command(
                name.to_owned(),
                RobotCommand::Delete(DeleteCommand {}),
            ))
            .await
            .is_ok()
    }

    async fn rename(&self, name: &str, new_name: &str) -> bool {
        tracing::info!("rename {name} to {new_name} called");
        self.sender
            .send(RobotMessage::new_command(
                name.to_owned(),
                RobotCommand::Rename(RenameCommand {
                    new_name: new_name.to_owned(),
                }),
            ))
            .await
            .is_ok()
    }

    async fn r#move(&self, name: &str, direction: &str, steps: i32) -> bool {
        tracing::info!("move {name} to {direction} steps: {steps} called");
        let Ok(direction) = MoveDirection::try_from((direction, steps)) else {
            return false;
        };
        self.sender
            .send(RobotMessage::new_command(
                name.to_owned(),
                RobotCommand::Move(MoveCommand { direction }),
            ))
            .await
            .is_ok()
    }

    async fn attack(&self, name: &str, power: i32) -> bool {
        tracing::info!("attack {name} with power {power} called");
        self.sender
            .send(RobotMessage::new_command(
                name.to_owned(),
                RobotCommand::Attack(AttackCommand { power }),
            ))
            .await
            .is_ok()
    }

    async fn repair(&self, name: &str) -> bool {
        tracing::info!("repair {name} called");
        self.sender
            .send(RobotMessage::new_command(
                name.to_owned(),
                RobotCommand::Repair(RepairCommand {}),
            ))
            .await
            .is_ok()
    }

    async fn transfer_charge(&self, name: &str) -> bool {
        tracing::info!("transfer_charge {name} called");
        self.sender
            .send(RobotMessage::new_command(
                name.to_owned(),
                RobotCommand::TransferCharge(TransferChargeCommand {}),
            ))
            .await
            .is_ok()
    }

    #[zbus(out_args("x", "y", "battery", "health", "team_id"))]
    async fn get_robot_state(&self, name: &str) -> (i32, i32, i32, i32, i32) {
        tracing::info!("get_robot_state {name} called");
        let (return_tx, return_rx) = smol::channel::bounded(1);
        if self
            .sender
            .send(RobotMessage::new_request(
                name.to_owned(),
                RobotCommand::Info(InfoCommand {}),
                Some(return_tx),
            ))
            .await
            .is_err()
        {
            return (0, 0, 0, 0, 0);
        }

        let Ok(response) = return_rx.recv().await else {
            return (0, 0, 0, 0, 0);
        };

        (
            response.position.x,
            response.position.y,
            response.battery_level,
            response.health,
            response.team_id,
        )
    }

    /// A signal; the implementation is provided by the macro.
    #[zbus(signal)]
    async fn robot_state(
        emitter: &SignalEmitter<'_>,
        event: &str,
        name: &str,
        x: i32,
        y: i32,
        battery: i32,
        health: i32,
        team_id: i32,
    ) -> zbus::Result<()>;
}

#[zbus::interface(name = "dev.luehr.rustytherobot.World")]
impl World {
    #[zbus(out_args("robot_list"))]
    async fn get_robot_list(&self) -> Vec<String> {
        tracing::info!("get_robot_list called");
        let (return_tx, return_rx) = smol::channel::bounded(1);
        if self
            .sender
            .send(WorldMessage::new_request(
                WorldCommand::ListRobots(ListRobotsCommand {}),
                Some(return_tx),
            ))
            .await
            .is_err()
        {
            tracing::warn!("get_robot_list failed (1)");
            return vec![];
        }

        let Ok(WorldReturn::RobotList(response)) = return_rx.recv().await else {
            tracing::warn!("get_robot_list failed (2)");
            return vec![];
        };

        response
    }
    #[zbus(out_args("board_dimensions"))]
    async fn get_board_dimensions(&self) -> zbus::fdo::Result<Dimension> {
        tracing::info!("get_board_dimensions called");
        let (return_tx, return_rx) = smol::channel::bounded(1);
        if self
            .sender
            .send(WorldMessage::new_request(
                WorldCommand::BoardDimensions(BoardDimensionsCommand {}),
                Some(return_tx),
            ))
            .await
            .is_err()
        {
            tracing::warn!("get_board_dimensions failed (1)");
            return Err(zbus::fdo::Error::Failed("Request could not be send".into()));
        }

        let Ok(WorldReturn::BoardDimensions(response)) = return_rx.recv().await else {
            tracing::warn!("get_board_dimensions failed (2)");
            return Err(zbus::fdo::Error::Failed("Channel closed unexpected".into()));
        };

        Ok(response)
    }

    #[zbus(out_args("chargepad_list"))]
    async fn get_chargepads(&self) -> zbus::fdo::Result<Vec<Position>> {
        tracing::info!("chargepad_list called");
        let (return_tx, return_rx) = smol::channel::bounded(1);
        if self
            .sender
            .send(WorldMessage::new_request(
                WorldCommand::ListChargepads(ListChargepadsCommand {}),
                Some(return_tx),
            ))
            .await
            .is_err()
        {
            tracing::warn!("chargepad_list failed (1)");
            return Err(zbus::fdo::Error::Failed("Request could not be send".into()));
        }

        let Ok(WorldReturn::ChargepadList(response)) = return_rx.recv().await else {
            tracing::warn!("chargepad_list failed (2)");
            return Err(zbus::fdo::Error::Failed("Channel closed unexpected".into()));
        };

        Ok(response)
    }
}
