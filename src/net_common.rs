use crate::game::{self, Building, GameReadiness, Player};
use macroquad::{color::Color, math::Vec2};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

//Alternative to Vec2 which is sendable over threads and can be serialised for network
#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct NetPosition {
    pub x: f32,
    pub y: f32,
}

impl NetPosition {
    pub const fn from_vec2(vec: Vec2) -> NetPosition {
        NetPosition { x: vec.x, y: vec.y }
    }

    pub const fn to_vec2(&self) -> Vec2 {
        Vec2 {
            x: self.x,
            y: self.y,
        }
    }

    //Check if two positions equal
    pub fn equals(&self, other: &NetPosition) -> bool {
        self.x == other.x && self.y == other.y
    }
}

//Same for colour
#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct NetColour {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl NetColour {
    pub const fn from_col(col: Color) -> NetColour {
        NetColour {
            a: col.a,
            r: col.r,
            g: col.g,
            b: col.b,
        }
    }

    pub const fn to_col(&self) -> Color {
        Color {
            a: self.a,
            r: self.r,
            g: self.g,
            b: self.b,
        }
    }
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct NetBuilding {
    pub position: NetPosition,
    pub width: f32,
    pub height: f32,
    pub colour: NetColour,
}

impl NetBuilding {
    pub const fn from_building(b: &Building) -> NetBuilding {
        NetBuilding {
            position: NetPosition::from_vec2(b.position),
            colour: NetColour::from_col(b.colour),
            height: b.height,
            width: b.width,
        }
    }

    pub const fn to_building(&self) -> Building {
        Building {
            position: self.position.to_vec2(),
            colour: self.colour.to_col(),
            height: self.height,
            width: self.width,
        }
    }
}

//Info for client initialisation
#[derive(Serialize, Deserialize)]
pub struct NetPlayerInfo {
    pub players: Vec<NetPlayer>,
    pub your_num: u8,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NetPlayer {
    pub position: NetPosition,
    pub id: u8,
    pub name: String,
    pub colour: NetColour,
}

impl NetPlayer {
    pub fn from_player(p: &Player) -> NetPlayer {
        NetPlayer {
            position: NetPosition::from_vec2(p.position),
            colour: NetColour::from_col(p.colour),
            id: p.id,
            name: p.name.clone(),
        }
    }
}

// //Game state to be constantly shared between players
// #[derive(Serialize, Deserialize)]
// pub struct NetGameState {
//     pub players: Vec<NetPlayer>,
// }

//Map to send to clients
#[derive(Serialize, Deserialize)]
pub struct Map {
    pub buildings: Vec<NetBuilding>,
}

//Player id and position
#[derive(Serialize, Deserialize)]
pub struct PositionMap {
    pub id: u8,
    pub pos: NetPosition,
}

#[derive(Serialize, Deserialize)]
pub struct RegistrationInfo {
    pub name: String,
    pub colour: NetColour,
}

#[derive(Serialize, Deserialize)]
pub enum Commands {
    RegisterPlayer(RegistrationInfo),
    Move(NetPosition),
    MovedPlayers(Vec<PositionMap>),
    SendMap(Map),
    SendPlayerInfo(NetPlayerInfo),
    AddPlayer(NetPlayer),
    RemovePlayer(u8),
    AllowClientReady(u8),
}

pub struct ClientSettings {
    pub name: String,
    pub colour: Color,
    pub ip: String,
    pub port: u16,
}

//Set an error on game state
pub fn set_error(state_lock: &Arc<Mutex<game::GameState>>, error: String) {
    let mut game = state_lock.lock().unwrap();
    game.ready = GameReadiness::Error(error);
    drop(game);
}
