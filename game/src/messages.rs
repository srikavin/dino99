use crate::input::Input;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum PlayerState {
    Playing,
    Dead,
    Spectating,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub username: String,
    pub id: Uuid,
    pub state: PlayerState,
}

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum LobbyState {
    Waiting,
    InPlay,
    Ended,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum C2SMessage {
    LobbyJoinRequest { name: String, lobby_id: String },
    GameInput { tick: u64, input: Input },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum S2CMessage {
    LobbyJoinSuccess {
        player_id: Uuid,
        players: Vec<PlayerInfo>,
    },
    LobbyJoinFailureResponse {
        reason: String,
    },
    LobbyJoinEvent {
        player: PlayerInfo,
    },
    LobbyStateChangeEvent {
        new_state: LobbyState,
    },
    GameTickEvent {
        tick: u64,
        players: Vec<(Uuid, Input)>,
    },
    InvalidMessage {
        error: String,
    },
}
