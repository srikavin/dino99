use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
enum PlayerState {
    Playing,
    Spectating,
}

#[derive(Debug, Serialize, Deserialize)]
struct Player {
    index: u16,
    state: PlayerState,
    username: String,
    // game_state: GameState,
}

#[derive(Debug, Serialize, Deserialize)]
enum LobbyState {
    Waiting,
    InPlay,
    Ended,
}

#[derive(Debug, Serialize, Deserialize)]
struct Lobby {
    state: LobbyState,
    players: Vec<Player>,
}
