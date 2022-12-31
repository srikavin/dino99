use std::sync::{Arc, Mutex};

use actix::prelude::*;
use actix::{Actor, Addr, AsyncContext, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use uuid::Uuid;
use web::{Data, Payload};

use game::messages::C2SMessage;
use game::messages::C2SMessage::LobbyJoinRequest;
use game::messages::S2CMessage::InvalidMessage;

use crate::lobby::{LobbyActor, PlayerMessage, ServerMessage};
use crate::AppState;

pub(crate) struct ClientConnection {
    lobbies: Arc<Mutex<AppState>>,
    lobby: Option<Addr<LobbyActor>>,
    id: Uuid,
}

impl Actor for ClientConnection {
    type Context = ws::WebsocketContext<Self>;
}

// Handle incoming websocket messages
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ClientConnection {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                let parsed = serde_json::from_str::<C2SMessage>(&text);
                match parsed {
                    Ok(LobbyJoinRequest { lobby_id, name }) if self.lobby.is_none() => {
                        let lobby_map = &mut self.lobbies.lock().unwrap().lobbies;
                        self.lobby = Option::from(match lobby_map.get_mut(lobby_id.as_str()) {
                            Some(found) => found.clone(),
                            None => {
                                let new_lobby = LobbyActor::new().start();
                                lobby_map.insert(lobby_id.clone(), new_lobby.clone());
                                dbg!("Making new lobby");
                                new_lobby
                            }
                        });
                        self.lobby.as_ref().unwrap().do_send(PlayerMessage {
                            client_id: self.id,
                            client_message: { LobbyJoinRequest { lobby_id, name } },
                            recipient: ctx.address(),
                        });
                    }
                    Ok(message) if self.lobby.is_some() => {
                        self.lobby.as_ref().unwrap().do_send(PlayerMessage {
                            client_id: self.id,
                            client_message: message,
                            recipient: ctx.address(),
                        });
                    }
                    Err(error) => ctx.text(
                        serde_json::to_string(&InvalidMessage {
                            error: error.to_string(),
                        })
                        .unwrap(),
                    ),
                    _ => ctx.text(
                        serde_json::to_string(&InvalidMessage {
                            error: "No handlers".to_string(),
                        })
                        .unwrap(),
                    ),
                }
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

// Handle outgoing websocket messages
impl Handler<ServerMessage> for ClientConnection {
    type Result = ();

    fn handle(&mut self, msg: ServerMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(serde_json::to_string(&msg.server_message).unwrap());
    }
}

pub(crate) async fn game_websocket(
    req: HttpRequest,
    stream: Payload,
    data: Data<Arc<Mutex<AppState>>>,
) -> Result<HttpResponse, Error> {
    let resp = ws::start(
        ClientConnection {
            lobbies: data.get_ref().clone(),
            lobby: None,
            id: Uuid::new_v4(),
        },
        &req,
        stream,
    );
    println!("{:?}", resp);
    resp
}
