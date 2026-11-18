use crate::robot::Event as RobotEvent;
use crate::robot::State as RobotState;
use anyhow::Result;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::{
    Json,
    extract::State,
    http::StatusCode,
    routing::{delete, get, post, put},
};
use rust_embed::RustEmbed;
use rusty_common::dimension::Dimension;
use rusty_common::position::Position;
use serde::Deserialize;
use std::convert::Infallible;
use tokio::sync::broadcast;
use tokio_stream::{StreamExt, wrappers::BroadcastStream};

#[derive(RustEmbed)]
#[folder = "web-ui/"]
struct WebUi;

#[derive(Debug, Deserialize)]
struct RobotRequest {
    name: String,
}

#[derive(Debug, Deserialize)]
struct RenameRequest {
    name: String,
    new_name: String,
}

#[derive(Debug, Deserialize)]
struct MoveRequest {
    name: String,
    direction: String,
    steps: i32,
}

#[derive(Debug, Deserialize)]
struct AttackRequest {
    name: String,
    power: i32,
}

#[derive(Clone)]
struct AppState {
    robot_proxy: crate::dbus::RobotProxy<'static>,
    world_proxy: crate::dbus::WorldProxy<'static>,
    events: broadcast::Sender<RobotEvent>,
}

pub async fn start(
    robot_proxy: crate::dbus::RobotProxy<'static>,
    world_proxy: crate::dbus::WorldProxy<'static>,
    events: broadcast::Sender<RobotEvent>,
) -> Result<()> {
    let app = axum::Router::new()
        .route("/robot_list", get(robot_list))
        .route("/robot_state", post(robot_state))
        .route("/create", post(create))
        .route("/remove", delete(remove))
        .route("/rename", put(rename))
        .route("/move", put(r#move))
        .route("/attack", put(attack))
        .route("/repair", put(repair))
        .route("/transfer_charge", put(transfer_charge))
        .route("/board_dimensions", get(board_dimensions))
        .route("/chargepads", get(chargepads))
        .route("/event", get(event))
        .fallback(web_ui)
        .with_state(AppState {
            robot_proxy,
            world_proxy,
            events,
        });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn web_ui(uri: axum::http::Uri) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    let file = WebUi::get(path).ok_or(StatusCode::NOT_FOUND)?;
    let mime = mime_guess::from_path(path).first_or_octet_stream();

    Ok((
        [(axum::http::header::CONTENT_TYPE, mime.to_string())],
        file.data,
    ))
}

async fn robot_state(
    State(AppState { robot_proxy, .. }): State<AppState>,
    Json(body): Json<RobotRequest>,
) -> Result<Json<RobotState>, StatusCode> {
    let rs = robot_proxy
        .get_robot_state(&body.name)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(RobotState::from(rs).into())
}

async fn robot_list(
    State(AppState { world_proxy, .. }): State<AppState>,
) -> Result<Json<Vec<String>>, StatusCode> {
    let list = world_proxy
        .get_robot_list()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(list))
}

async fn board_dimensions(
    State(AppState { world_proxy, .. }): State<AppState>,
) -> Result<Json<Dimension>, StatusCode> {
    let dimension = world_proxy
        .get_board_dimensions()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(dimension))
}

async fn chargepads(
    State(AppState { world_proxy, .. }): State<AppState>,
) -> Result<Json<Vec<Position>>, StatusCode> {
    let list = world_proxy
        .get_chargepads()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(list))
}

async fn create(
    State(AppState { robot_proxy, .. }): State<AppState>,
    Json(body): Json<RobotRequest>,
) -> Result<(), StatusCode> {
    robot_proxy
        .create(&body.name)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(())
}
async fn remove(
    State(AppState { robot_proxy, .. }): State<AppState>,
    Json(body): Json<RobotRequest>,
) -> Result<(), StatusCode> {
    robot_proxy
        .delete(&body.name)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(())
}
async fn rename(
    State(AppState { robot_proxy, .. }): State<AppState>,
    Json(body): Json<RenameRequest>,
) -> Result<(), StatusCode> {
    robot_proxy
        .rename(&body.name, &body.new_name)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(())
}

async fn r#move(
    State(AppState { robot_proxy, .. }): State<AppState>,
    Json(body): Json<MoveRequest>,
) -> Result<(), StatusCode> {
    robot_proxy
        .r#move(&body.name, &body.direction, body.steps)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(())
}

async fn attack(
    State(AppState { robot_proxy, .. }): State<AppState>,
    Json(body): Json<AttackRequest>,
) -> Result<(), StatusCode> {
    robot_proxy
        .attack(&body.name, body.power)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(())
}

async fn repair(
    State(AppState { robot_proxy, .. }): State<AppState>,
    Json(body): Json<RobotRequest>,
) -> Result<(), StatusCode> {
    robot_proxy
        .repair(&body.name)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(())
}

async fn transfer_charge(
    State(AppState { robot_proxy, .. }): State<AppState>,
    Json(body): Json<RobotRequest>,
) -> Result<(), StatusCode> {
    robot_proxy
        .transfer_charge(&body.name)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(())
}

async fn event(
    State(AppState { events, .. }): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let rx = events.subscribe();

    let stream = BroadcastStream::new(rx).filter_map(|result| {
        let event = match result {
            Ok(event) => event,
            Err(err) => {
                tracing::warn!(%err, "SSE receiver lagged");
                return None;
            }
        };

        match Event::default().event("robot_state").json_data(event) {
            Ok(event) => Some(Ok::<_, Infallible>(event)),

            Err(err) => {
                tracing::error!(%err, "Failed to serialize SSE event");
                None
            }
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}
