use crate::graphics::renderer::{Camera, RenderState};
use crate::graphics::window::{FrameState, KeyCode};

use crate::gameplay::map::*;
use crate::{types::*, State};
/*
   Server notes:
   - Verify client moves, check team is correct, check move is valid
*/

pub struct Tick {
    pub tick_number: i32,
    pub player_a_move: PlayerAction,
    pub player_b_move: PlayerAction,
}

pub fn ticks_every_x_seconds(x: i32) -> f32 {
    1.0 / (x as f32)
}

pub struct GameState {
    current_map: GameMap,
    tick_rate: f32,
    tick_count: i32,
    tick_timer: f32,
}

impl GameState {
    pub fn new(state : &mut State) -> GameState {
        GameState {
            current_map: GameMap::main_menu(Vec2i::new(250 * 2, 150 * 2), &mut state.rs),
            tick_rate: ticks_every_x_seconds(24),
            tick_count: 0,
            tick_timer: 0.0,
        }
    }

    pub fn update_and_render(&mut self, state : &mut State) {
        let rs: &mut RenderState = &mut state.rs;
        let fs: &mut FrameState = &mut state.fs;

        self.tick_timer += fs.delta_time;
        if self.tick_timer > self.tick_rate {
            self.tick_timer -= self.tick_rate;

            let tick = Tick {
                tick_number: self.tick_count,
                player_a_move: self.current_map.get_next_action(),
                player_b_move: PlayerAction::None,
            };
            self.current_map.tick(&tick);
            self.tick_count += 1;
            //println!("tick {}", self.tick_count);
        }

        self.current_map.frame_update_and_render(state);
    }
}
