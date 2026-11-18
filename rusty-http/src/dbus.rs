use crate::robot::Event as RobotEvent;
use futures_lite::stream::StreamExt;
use rusty_common::dimension::Dimension;
use rusty_common::position::Position;
use tracing::trace;
use zbus::Connection;

#[zbus::proxy(
    interface = "dev.luehr.rustytherobot.Robot",
    default_service = "dev.luehr.rustytherobot",
    default_path = "/"
)]
pub trait Robot {
    fn create(&self, name: &str) -> zbus::Result<bool>;

    fn delete(&self, name: &str) -> zbus::Result<bool>;

    fn rename(&self, name: &str, new_name: &str) -> zbus::Result<bool>;

    fn r#move(&self, name: &str, direction: &str, steps: i32) -> zbus::Result<bool>;

    fn attack(&self, name: &str, power: i32) -> zbus::Result<bool>;

    fn repair(&self, name: &str) -> zbus::Result<bool>;

    fn transfer_charge(&self, name: &str) -> zbus::Result<bool>;

    fn get_robot_state(&self, name: &str) -> zbus::Result<(i32, i32, i32, i32, i32)>;

    #[zbus(signal)]
    fn robot_state(
        &self,
        event: &str,
        name: &str,
        x: i32,
        y: i32,
        battery: i32,
        health: i32,
        team_id: i32,
    ) -> zbus::Result<()>;
}

#[zbus::proxy(
    interface = "dev.luehr.rustytherobot.World",
    default_service = "dev.luehr.rustytherobot",
    default_path = "/"
)]
pub trait World {
    fn get_robot_list(&self) -> zbus::Result<Vec<String>>;
    fn get_board_dimensions(&self) -> zbus::Result<Dimension>;
    fn get_chargepads(&self) -> zbus::Result<Vec<Position>>;
}

pub async fn watch_robot_state(
    connection: Connection,
    events: tokio::sync::broadcast::Sender<RobotEvent>,
) -> zbus::Result<()> {
    let robot_proxy = RobotProxy::new(&connection).await?;
    let mut robot_state_stream = robot_proxy.receive_robot_state().await?;

    while let Some(msg) = robot_state_stream.next().await {
        // struct `JobNewArgs` is generated from `job_new` signal function arguments
        let args: RobotStateArgs = msg.args().expect("Error parsing message");

        events
            .send(RobotEvent::new(
                args.name.to_string(),
                args.event.to_string(),
                args.x,
                args.y,
                args.battery,
                args.health,
                args.team_id,
            ))
            .ok();
        trace!(
            "RobotState received: event={} name={} x={} y={} battery={} health={} team_id={}",
            args.event, args.name, args.x, args.y, args.battery, args.health, args.team_id
        );
    }

    panic!("Stream ended unexpectedly");
}
