use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct State {
    x: i32,
    y: i32,
    battery: i32,
    health: i32,
    team_id: i32,
}

impl From<(i32, i32, i32, i32, i32)> for State {
    fn from(value: (i32, i32, i32, i32, i32)) -> Self {
        Self {
            x: value.0,
            y: value.1,
            battery: value.2,
            health: value.3,
            team_id: value.4,
        }
    }
}

#[derive(Serialize, Clone)]
pub struct Event {
    name: String,
    event: String,
    state: State,
}

impl Event {
    pub fn new(
        name: String,
        event: String,
        x: i32,
        y: i32,
        battery: i32,
        health: i32,
        team_id: i32,
    ) -> Self {
        Self {
            name,
            event,
            state: State {
                x,
                y,
                battery,
                health,
                team_id,
            },
        }
    }
}
