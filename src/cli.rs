use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "robot")]
#[command(about = "rusty the robot")]
pub struct Cli {
    /// name of the robot
    pub robot_name: String,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// move the robot
    Move(MoveCommand),

    /// info about robot position
    Info,

    /// rename the robot
    Rename(RenameCommand),
}

#[derive(Args, Debug)]
pub struct RenameCommand {
    /// new name
    pub new_name: String,
}

#[derive(Args, Debug)]
pub struct MoveCommand {
    #[command(subcommand)]
    pub direction: MoveDirection,
}

#[derive(Subcommand, Debug)]
pub enum MoveDirection {
    /// Move up
    Up {
        /// number of steps
        steps: i32,
    },

    /// Move down
    Down,

    /// Move left
    Left,

    /// Move right
    Right,
}
