use crate::keys::{Edge, StatefulKeys};

#[derive(Clone, Debug, Copy)]
pub enum GameState {
    Edit,
    Run,
}

impl GameState {
    pub fn new() -> Self {
        GameState::Run
    }

    pub fn update(&mut self, keys: &mut StatefulKeys) -> Self {
        if let Some(edge) = keys.start().change() {
            if let Edge::Rising = edge {
                *self = match *self {
                    GameState::Edit => GameState::Run,
                    GameState::Run => GameState::Edit,
                };
            }
        }

        *self
    }
}
