use crate::direction::Direction;
use argh::FromArgs;

#[derive(FromArgs, Debug)]
#[argh(description = "rusty the robot")]
pub struct Cli {
    #[argh(subcommand)]
    pub command: Command,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum Command {
    Robot(Robot),
    World(World),
    DBus(DBus),
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "dbus", description = "run the game in dbus mode")]
pub struct DBus {}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "robot", description = "command a robot")]
pub struct Robot {
    #[argh(positional, description = "name of the robot")]
    pub robot_name: String,

    #[argh(subcommand)]
    pub robot_command: RobotCommand,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "world", description = "commands on the world")]
pub struct World {
    #[argh(subcommand)]
    pub world_command: WorldCommand,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum WorldCommand {
    ListRobots(ListRobotsCommand),
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "list", description = "list all robots")]
pub struct ListRobotsCommand {}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum RobotCommand {
    Move(MoveCommand),
    Attack(AttackCommand),
    TransferCharge(TransferChargeCommand),
    Repair(RepairCommand),
    Info(InfoCommand),
    Rename(RenameCommand),
    Delete(DeleteCommand),
    Create(CreateCommand),
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(
    subcommand,
    name = "attack",
    description = "attacks all hostile robots near by"
)]
pub struct AttackCommand {
    #[argh(positional)]
    pub power: i32,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(
    subcommand,
    name = "repair",
    description = "repairs all friendly robots near by"
)]
pub struct RepairCommand {}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(
    subcommand,
    name = "tranfer",
    description = "split power between all friendly robots near by"
)]
pub struct TransferChargeCommand {}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "rename", description = "rename the robot")]
pub struct RenameCommand {
    #[argh(positional)]
    pub new_name: String,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "info", description = "info about robot position")]
pub struct InfoCommand {}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "delete", description = "delete the robot")]
pub struct DeleteCommand {}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(
    subcommand,
    name = "create",
    description = "create robot with given name"
)]
pub struct CreateCommand {}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "move", description = "move the robot")]
pub struct MoveCommand {
    #[argh(subcommand)]
    pub direction: MoveDirection,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum MoveDirection {
    Up(Up),
    Down(Down),
    Left(Left),
    Right(Right),
}

impl TryFrom<(&str, i32)> for MoveDirection {
    type Error = String;

    fn try_from((direction, steps): (&str, i32)) -> Result<Self, Self::Error> {
        match direction.to_ascii_lowercase().as_str() {
            "up" => Ok(Self::Up(Up { steps })),
            "down" => Ok(Self::Down(Down {})),
            "left" => Ok(Self::Left(Left {})),
            "right" => Ok(Self::Right(Right {})),
            _ => Err(format!("invalid move direction: {direction}")),
        }
    }
}

impl From<MoveDirection> for Direction {
    fn from(direction: MoveDirection) -> Self {
        match direction {
            MoveDirection::Up(up) => Direction::Up(up.steps),
            MoveDirection::Down(_) => Direction::Down,
            MoveDirection::Left(_) => Direction::Left,
            MoveDirection::Right(_) => Direction::Right,
        }
    }
}

#[derive(FromArgs, PartialEq, Debug)]
/// Move up.
#[argh(subcommand, name = "up")]
pub struct Up {
    /// number of steps
    #[argh(positional)]
    pub steps: i32,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Move down.
#[argh(subcommand, name = "down")]
pub struct Down {}

#[derive(FromArgs, PartialEq, Debug)]
/// Move left.
#[argh(subcommand, name = "left")]
pub struct Left {}

#[derive(FromArgs, PartialEq, Debug)]
/// Move right.
#[argh(subcommand, name = "right")]
pub struct Right {}
