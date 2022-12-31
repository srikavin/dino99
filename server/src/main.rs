use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use actix::Addr;
use actix_web::middleware::Logger;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use web::Data;

use game::game::GameState;
use game::input::Input;

use crate::lobby::{LobbyActor, LobbyId};
use crate::server::game_websocket;

mod lobby;
mod server;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    let mut body = GameState::new();
    body.tick(Input::Duck);
    body.tick(Input::Jump);
    HttpResponse::Ok().body(serde_json::to_string(&body).unwrap())
}

pub(crate) struct AppState {
    lobbies: HashMap<LobbyId, Addr<LobbyActor>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");

    let data = Data::new(Arc::new(Mutex::new(AppState {
        lobbies: HashMap::new(),
    })));

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(data.clone())
            .service(hello)
            .service(echo)
            .route("/ws/", web::get().to(game_websocket))
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
