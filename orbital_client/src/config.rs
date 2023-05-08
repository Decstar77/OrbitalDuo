use std::{
    env,
    sync::{Arc, Mutex},
};

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::types::{Vec2, Vec2i};

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub window_size: Option<Vec2i>,
    pub window_pos: Option<Vec2i>,
    pub assets_path: String,
}

impl Config {
    pub fn new() -> Config {
        Config {
            window_size: Some(Vec2i::new(1280, 720)),
            window_pos: None,
            assets_path: String::from("assets/"),
        }
    }
}

lazy_static! {
    static ref SINGLETON_INSTANCE: Mutex<Arc<Config>> = Mutex::new(Arc::new(Config::new()));
}

pub fn config_parse_command_line() {
    let args: Vec<String> = env::args().collect();

    for i in 0..args.len() {
        if args[i] == "-config" {
            load_config_from_file(&args[i + 1]);
        }
    }
}

pub fn set_config(config: Config) {
    let mut singleton = SINGLETON_INSTANCE.lock().unwrap();
    *singleton = Arc::new(config);
}

pub fn get_config() -> Arc<Config> {
    SINGLETON_INSTANCE.lock().unwrap().clone()
}

pub fn save_config_to_file(path: &str) {
    let cfg = get_config();
    let json = serde_json::to_string(cfg.as_ref()).expect("Failed to serialize config file");
    std::fs::write(path, json).expect("Failed to write config file");
}

pub fn load_config_from_file(path: &str) {
    let path = std::fs::read_to_string(path).expect("Failed to read config file");
    let config: Config = serde_json::from_str(&path).expect("Failed to parse config file");
    set_config(config);
}
