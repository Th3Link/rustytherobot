use crate::direction::Direction;
use argh::FromArgs;

#[derive(FromArgs, Debug)]
#[argh(description = "rusty the robot")]
pub struct Cli {
    #[argh(positional, description = "name of the robot")]
    pub robot_name: String,

    #[argh(subcommand)]
    pub command: Command,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum Command {
    Move(MoveCommand),
    Info(InfoCommand),
    Rename(RenameCommand),
    Delete(DeleteCommand),
    Create(CreateCommand),
}

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
