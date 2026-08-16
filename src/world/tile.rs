use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Tile {
    Wall,
    ChargingPad,
}
