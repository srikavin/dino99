use std::collections::VecDeque;
use serde::{Deserialize, Serialize};
use crate::input::Input;

#[derive(Debug, Serialize, Deserialize)]
struct GamePlayer {
    y: u16,
    jump_tick: u8,
    peak_jump_tick: u8,
    is_ducked: bool,
    speed: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum GameObstacleCategory {
    Tree,
    Bush,
}

#[derive(Debug, Serialize, Deserialize)]
struct GameObstacle {
    category: GameObstacleCategory,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameState {
    score: u64,
    player: GamePlayer,
    obstacles: VecDeque<GameObstacle>
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
            Input::ShortJump => {
                self.peak_jump_tick = 5;
                self.jump_tick = 1;
                self.is_ducked = false;
            }
            Input::HighJump => {
                self.peak_jump_tick = 5;
                self.jump_tick = 1;
                self.is_ducked = false;
            }
            Input::Duck{pressed} => {
                self.is_ducked = pressed;
            }
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
                speed: 0,
            },
            obstacles: VecDeque::new(),
        }
    }

    pub fn tick(&mut self, input: Input) {
        self.player.handle_input(input);

        if self.player.jump_tick < self.player.peak_jump_tick {
            self.player.y += 10;
            self.player.jump_tick += 1;
        } else if self.player.jump_tick < self.player.peak_jump_tick * 2 {
            self.player.y -= 10;
            self.player.jump_tick += 1;
        } else {
            self.player.y = 0;
            self.player.jump_tick = 0
        }
    }
}
