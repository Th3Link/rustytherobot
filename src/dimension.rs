use serde::{Deserialize, Serialize};
use zbus::zvariant::Type;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct Dimension {
    pub x: i32,
    pub y: i32,
}

impl std::fmt::Display for Dimension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}/{})", self.x, self.y)
    }
}
