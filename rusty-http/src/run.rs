use std::future::pending;

use crate::dbus::watch_robot_state;
use crate::dbus::{RobotProxy, WorldProxy};
use tracing::info;
use zbus::Connection;

pub async fn run() -> anyhow::Result<()> {
    info!("rusty on http started");
    let connection = Connection::session().await?;
    let (event_tx, _) = tokio::sync::broadcast::channel(128);

    tokio::spawn(watch_robot_state(connection.clone(), event_tx.clone()));

    let robot_proxy: RobotProxy<'static> = RobotProxy::new(&connection).await?;
    let world_proxy: WorldProxy<'static> = WorldProxy::new(&connection).await?;
    robot_proxy.delete("rustyMega").await?;
    robot_proxy.create("rustyMega").await?;

    tokio::spawn(async move { crate::http::start(robot_proxy, world_proxy, event_tx).await });

    tokio::time::sleep(core::time::Duration::from_secs(3)).await;

    pending::<()>().await;
    info!("done");
    Ok(())
}
