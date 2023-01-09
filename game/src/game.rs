use crate::input::Input;
use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::collections::VecDeque;
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[derive(Debug, Serialize, Deserialize)]
pub struct GamePlayer {
    pub y: i32,
    pub jump_tick: u8,
    pub peak_jump_tick: u8,
    pub is_ducked: bool,
    pub speed: u32,
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum GameObstacleCategory {
    Cactus,
    Bird,
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct BoundingBox {
    pub w: u32,
    pub h: u32,
}

trait Collidable {
    fn collision_box(&self) -> BoundingBox;
    fn position(&self) -> Position;
}

fn is_colliding(a: &dyn Collidable, b: &dyn Collidable) -> bool {
    let a_collision = a.collision_box();
    let a_position = a.position();

    let (ax0, ax1, ay0, ay1) = (
        a_position.x,
        a_position.x + a_collision.w as i32,
        a_position.y,
        a_position.y + a_collision.h as i32,
    );

    let b_collision = b.collision_box();
    let b_position = b.position();

    let (bx0, bx1, by0, by1) = (
        b_position.x,
        b_position.x + b_collision.w as i32,
        b_position.y,
        b_position.y + b_collision.h as i32,
    );

    return ax0 <= bx1 && ax1 >= bx0 && ay0 <= by1 && ay1 >= by0;
}

impl Collidable for GameObstacle {
    fn collision_box(&self) -> BoundingBox {
        match self.category {
            GameObstacleCategory::Cactus => BoundingBox { w: 32, h: 32 },
            GameObstacleCategory::Bird => BoundingBox { w: 46, h: 30 },
        }
    }

    fn position(&self) -> Position {
        return self.position;
    }
}

impl Collidable for GamePlayer {
    fn collision_box(&self) -> BoundingBox {
        return BoundingBox { w: 45, h: 48 };
    }

    fn position(&self) -> Position {
        return Position { x: 50, y: self.y };
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[derive(Debug, Serialize, Deserialize)]
pub struct GameObstacle {
    pub category: GameObstacleCategory,
    pub position: Position,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameState {
    pub score: u64,
    pub player: GamePlayer,
    pub obstacles: VecDeque<GameObstacle>,
    pub tick: u64,
    pub is_game_over: bool,
}

impl GamePlayer {
    fn is_on_ground(&self) -> bool {
        self.y == 0
    }

    fn handle_input(&mut self, input: Input) {
        if !self.is_on_ground() || self.jump_tick != 0 {
            return;
        }

        match input {
            Input::Jump => {
                self.peak_jump_tick = 5;
                self.jump_tick = 1;
                self.is_ducked = false;
            }
            Input::Duck => {
                self.is_ducked = true;
            }
            Input::Unduck => {
                self.is_ducked = false;
            }
            Input::None => {}
        };
    }
}

impl GameState {
    pub fn new() -> GameState {
        return GameState {
            score: 0,
            player: GamePlayer {
                y: 0,
                jump_tick: 0,
                peak_jump_tick: 0,
                is_ducked: false,
                speed: 5,
            },
            obstacles: VecDeque::new(),
            tick: 0,
            is_game_over: false,
        };
    }

    fn handle_collisions(&mut self) {
        for x in &self.obstacles {
            if is_colliding(&self.player, x) {
                self.is_game_over = true;
                break;
            }
        }
    }

    pub fn tick(&mut self, input: Input) {
        if self.is_game_over {
            return;
        }

        self.player.handle_input(input);

        if self.player.jump_tick < self.player.peak_jump_tick {
            self.player.y += 10;
            self.player.jump_tick += 1;
        } else if self.player.jump_tick + 1 < self.player.peak_jump_tick * 2 {
            self.player.y -= 10;
            self.player.jump_tick += 1;
        } else {
            self.player.y = 0;
            self.player.jump_tick = 0;
            self.player.peak_jump_tick = 0;
        }

        while let Some(x) = self.obstacles.front() {
            if x.position.x < -(x.collision_box().w as i32) {
                self.obstacles.pop_front();
                continue;
            }
            break;
        }

        for x in &mut self.obstacles {
            x.position.x = x.position.x - (self.player.speed as i32);
        }

        while self.obstacles.len() < 16 {
            self.obstacles.push_back(GameObstacle {
                category: GameObstacleCategory::Bird,
                position: Position {
                    x: (512 + 100 * self.obstacles.len()) as i32,
                    y: 70,
                },
            })
        }

        self.handle_collisions();

        self.tick += 1;
    }
}
