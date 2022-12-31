use actix::prelude::*;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use uuid::Uuid;

use crate::server::ClientConnection;
use game::game::GameState;
use game::input::Input;
use game::messages::S2CMessage::{
    InvalidMessage, LobbyJoinEvent, LobbyJoinFailureResponse, LobbyJoinSuccess,
    LobbyStateChangeEvent,
};
use game::messages::{C2SMessage, LobbyState, PlayerInfo, PlayerState, S2CMessage};

pub(crate) type LobbyId = String;

#[derive(Message)]
#[rtype("()")]
pub(crate) struct ServerMessage {
    pub(crate) server_message: S2CMessage,
}

#[derive(Message)]
#[rtype("()")]
pub(crate) struct PlayerMessage {
    pub(crate) client_id: Uuid,
    pub(crate) client_message: C2SMessage,
    pub(crate) recipient: Addr<ClientConnection>,
}

#[derive(Debug)]
pub(crate) struct LobbyActor {
    state: LobbyState,
    players: HashMap<Uuid, Player>,
    current_tick: AtomicU64,
    server_delay: u64,
}

impl LobbyActor {
    pub(crate) fn new() -> LobbyActor {
        LobbyActor {
            state: LobbyState::Waiting,
            players: HashMap::new(),
            current_tick: AtomicU64::new(0),
            server_delay: 10,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Player {
    info: PlayerInfo,
    game_state: GameState,
    connection: Addr<ClientConnection>,
    future_inputs: VecDeque<(u64, Input)>,
}

impl Player {
    fn get_input_for_tick(&mut self, expected_tick: u64) -> Input {
        while !self.future_inputs.is_empty() {
            let (input_tick, _) = self.future_inputs.front().unwrap();

            if *input_tick < expected_tick {
                self.future_inputs.pop_front();
                continue;
            } else if *input_tick == expected_tick {
                let (_, input) = self.future_inputs.pop_front().unwrap();
                return input;
            } else {
                break;
            }
        }

        return Input::None;
    }

    fn send_message(&self, msg: S2CMessage) {
        self.connection.do_send(ServerMessage {
            server_message: msg,
        });
    }
}

impl Actor for LobbyActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(Duration::from_millis(50), |lobby, _ctx| {
            lobby.on_tick();
        });
    }
}

impl LobbyActor {
    fn broadcast(&mut self, message: S2CMessage) {
        for (_uuid, player) in &self.players {
            player.send_message(message.clone());
        }
    }

    fn on_tick(&mut self) {
        if self.state != LobbyState::InPlay {
            return;
        }

        if self.server_delay > 0 {
            self.server_delay -= 1;
            return;
        }

        let current_tick = self.current_tick.fetch_add(1, Ordering::Relaxed);

        let mut player_info = vec![];
        for (uuid, player) in &mut self.players {
            let input = player.get_input_for_tick(current_tick);

            player.game_state.tick(input);

            if input != Input::None {
                player_info.push((uuid.clone(), input));
            }
        }

        self.broadcast(S2CMessage::GameTickEvent {
            tick: current_tick,
            players: player_info,
        });
    }

    fn do_game_start(&mut self) {
        if !matches!(self.state, LobbyState::Waiting) {
            return;
        }

        self.state = LobbyState::InPlay;
        self.broadcast(LobbyStateChangeEvent {
            new_state: LobbyState::InPlay,
        });
    }
}

impl Handler<PlayerMessage> for LobbyActor {
    type Result = ();

    fn handle(&mut self, msg: PlayerMessage, _ctx: &mut Self::Context) -> Self::Result {
        if let C2SMessage::LobbyJoinRequest { lobby_id, name } = msg.client_message {
            dbg!("lobby_id={}: player '{}' joined", lobby_id, &name);
            if !matches!(self.state, LobbyState::Waiting) {
                msg.recipient.do_send(ServerMessage {
                    server_message: LobbyJoinFailureResponse {
                        reason: "Lobby already started!".to_string(),
                    },
                });
                return;
            }

            let mut player_infos = vec![];
            for (_uuid, player) in &self.players {
                player_infos.push(player.info.clone());
            }

            self.players.insert(
                msg.client_id,
                Player {
                    info: PlayerInfo {
                        username: name,
                        id: msg.client_id,
                        state: PlayerState::Playing,
                    },
                    future_inputs: VecDeque::new(),
                    connection: msg.recipient.clone(),
                    game_state: GameState::new(),
                },
            );

            msg.recipient.do_send(ServerMessage {
                server_message: LobbyJoinSuccess {
                    player_id: msg.client_id.clone(),
                    players: player_infos,
                },
            });

            self.broadcast(LobbyJoinEvent {
                player: self.players.get(&msg.client_id).unwrap().info.clone(),
            });

            if self.players.len() == 2 {
                self.do_game_start();
            }
            return;
        }

        let player_opt = self.players.get_mut(&msg.client_id);
        if let None = player_opt {
            msg.recipient.do_send(ServerMessage {
                server_message: InvalidMessage {
                    error: "Not a player in the lobby!".to_string(),
                },
            });
            return;
        }

        let player = player_opt.unwrap();

        match msg.client_message {
            C2SMessage::GameInput { input, tick } => {
                let expected_tick = match player.future_inputs.back() {
                    None => player.game_state.tick,
                    Some((input_tick, _)) => *input_tick + 1,
                };

                if tick < expected_tick {
                    return;
                }

                player.future_inputs.push_back((tick, input));
            }
            C2SMessage::LobbyJoinRequest { .. } => {
                unreachable!();
            }
        }
    }
}
