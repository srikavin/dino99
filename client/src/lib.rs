use crate::ClientStatus::Playing;
use game::game::{GameObstacle, GamePlayer, GameState};
use game::input::Input;
use game::messages::{C2SMessage, LobbyState, S2CMessage};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen(raw_module = "websocket")]
extern "C" {
    pub fn sendMessage(s: &str);
}

#[derive(Debug)]
pub enum ClientStatus {
    Connected,
    Playing(Uuid, LobbyState),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RenderState {
    pub additionalObstacles: VecDeque<GameObstacle>,
    pub player: GamePlayer,
}

#[wasm_bindgen]
pub struct GameClient {
    status: ClientStatus,
    lobby_name: String,
    game_states: HashMap<Uuid, GameState>,
}

#[wasm_bindgen]
impl GameClient {
    #[wasm_bindgen(constructor)]
    pub fn new(username: &str, lobby_name: &str) -> GameClient {
        sendMessage(
            serde_json::to_string(&C2SMessage::LobbyJoinRequest {
                lobby_id: lobby_name.to_string(),
                name: username.to_string(),
            })
            .unwrap()
            .as_str(),
        );
        GameClient {
            status: ClientStatus::Connected,
            lobby_name: lobby_name.to_string(),
            game_states: HashMap::new(),
        }
    }

    pub fn on_message(&mut self, s: &str) {
        let parsed = serde_json::from_str::<S2CMessage>(s);
        match parsed {
            Ok(message) => match message {
                S2CMessage::LobbyJoinSuccess { player_id, players } => {
                    self.status = Playing(player_id, LobbyState::Waiting);
                    console_log!("{:?}", &players);
                    for player_info in players {
                        self.game_states.insert(player_info.id, GameState::new());
                    }
                }
                S2CMessage::LobbyJoinEvent { player } => {
                    self.game_states.insert(player.id, GameState::new());
                }
                S2CMessage::LobbyJoinFailureResponse { .. } => {}
                S2CMessage::GameTickEvent { players, tick } => {
                    let ignore_uuid = match self.status {
                        Playing(uuid, _) => uuid,
                        _ => Uuid::nil(),
                    };
                    let mut seen = HashSet::new();
                    for (uuid, input) in players {
                        if uuid == ignore_uuid {
                            continue;
                        }
                        if let Some(state) = self.game_states.get_mut(&uuid) {
                            if tick > state.tick {
                                state.tick(input);
                            }
                        }

                        seen.insert(uuid);
                    }
                    for (uuid, state) in &mut self.game_states {
                        if uuid == &ignore_uuid {
                            continue;
                        }

                        if seen.contains(&uuid) {
                            continue;
                        }

                        if tick > state.tick {
                            state.tick(Input::None);
                        }
                    }
                }
                S2CMessage::InvalidMessage { error } => {
                    console_log!("Invalid message: {}", error);
                }
                S2CMessage::LobbyStateChangeEvent { new_state } => {
                    if let Playing(uuid, _old_state) = &self.status {
                        self.status = Playing(*uuid, new_state);
                    }
                }
            },
            Err(e) => {
                console_log!("Failed to parse ({}): {}", e, s);
            }
        }
    }

    pub fn tick(&mut self, input: Input) {
        console_log!("status={:?}", self.status);

        if let Playing(uuid, LobbyState::InPlay) = self.status {
            let state = self.game_states.get_mut(&uuid).unwrap();
            let current_tick = state.tick;

            state.tick(input);

            if !matches!(input, Input::None) {
                sendMessage(
                    serde_json::to_string(&C2SMessage::GameInput {
                        tick: current_tick,
                        input,
                    })
                    .unwrap()
                    .as_str(),
                );
            }
        }
    }

    pub fn game_state(&self) -> Result<JsValue, JsValue> {
        if let Playing(uuid, _) = self.status {
            Ok(serde_wasm_bindgen::to_value(&(uuid, &self.game_states))?)
        } else {
            Ok(JsValue::null())
        }
    }
}
