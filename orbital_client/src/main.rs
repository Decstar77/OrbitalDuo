#![allow(unused_variables)]
#![allow(dead_code)]
mod gameplay;
mod graphics;
mod types;
mod audio;
mod network;
mod config;

use std::hash::Hash;

use config::config_parse_command_line;

use crate::graphics::renderer::{RenderState};
use crate::graphics::window::{Window};
use crate::graphics::window::FrameState;
use crate::gameplay::game_state::GameState;
use crate::audio::AudioState;
use crate::network::NetworkState;

pub struct State {
    rs : RenderState,
    fs : FrameState,
    ad : AudioState,
    ns : NetworkState,
}

fn main() {
    config_parse_command_line();

    let mut window: Window = Window::new(";e");

    let rs = RenderState::new(&mut window);
    let fs = FrameState::new();
    let aud = AudioState::new();
    let ns = NetworkState::new();
    
    let mut state = State {
        rs : rs,
        fs : fs,
        ad : aud,
        ns : ns,
    };

    let mut gs = GameState::new(&mut state);

    while !window.should_close() {
        gs.update_and_render(&mut state);
        state.rs.draw_submit();
        window.update(&mut state.rs, &mut state.fs);

        //println!("{}", state.fs.delta_time * 1000.0)
    }
}
