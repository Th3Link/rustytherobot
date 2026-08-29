use crate::world::{self, World};
use zbus::object_server::SignalEmitter;

pub struct Robot {
    world: World,
}

impl Robot {
    pub fn new(world: World) -> Self {
        Self { world }
    }
}

#[zbus::interface(name = "dev.luehr.rustytherobot.Robot")]
impl Robot {
    fn create(&self, name: &str) -> bool {
        tracing::info!("create {name} called");
        true
    }

    fn delete(&self, name: &str) -> bool {
        tracing::info!("delete {name} called");
        true
    }

    fn rename(&self, name: &str, new_name: &str) -> bool {
        tracing::info!("rename {name} to {new_name} called");
        true
    }

    fn r#move(&self, name: &str, direction: &str, steps: i32) -> bool {
        tracing::info!("move {name} to {direction} steps: {steps} called");
        true
    }

    fn attack(&self, name: &str, power: i32) -> bool {
        tracing::info!("attack {name} with power {power} called");
        true
    }

    fn repair(&self, name: &str) -> bool {
        tracing::info!("repair {name} called");
        true
    }

    fn transfer_charge(&self, name: &str) -> bool {
        tracing::info!("transfer_charge {name} called");
        true
    }

    fn get_robot_state(&self, name: &str) -> (i32, i32, i32, i32) {
        tracing::info!("get_robot_state {name} called");
        (0, 0, 0, 0)
    }

    /// A signal; the implementation is provided by the macro.
    #[zbus(signal)]
    async fn robot_state(
        emitter: &SignalEmitter<'_>,
        name: &str,
        x: i32,
        y: i32,
        battery: i32,
        health: i32,
    ) -> zbus::Result<()>;
}
