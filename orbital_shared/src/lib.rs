use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct JoinPacket{
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum GamePacket {
    Join(JoinPacket),
    Leave,
    Move,
    Shoot,
}