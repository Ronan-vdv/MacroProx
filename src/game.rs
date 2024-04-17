use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use macroquad::{
    audio::{self, set_sound_volume},
    prelude::*,
    text,
};

use crate::net_common::{NetBuilding, NetPlayer, NetPosition};

pub struct Player {
    pub id: u8,
    pub name: String,
    pub position: Vec2,
    pub colour: Color,
}

impl Player {
    fn draw(&self, offset: Vec2) {
        let head_height = 7.0;

        let pos = self.position + offset;
        draw_rectangle(
            pos.x,
            pos.y + head_height,
            self.get_width(),
            self.get_height() - head_height,
            self.colour,
        );
        let font_size = 17;
        let t_size = measure_text(&self.name, None, font_size, 1.0);
        draw_text_ex(
            &self.name,
            pos.x - t_size.width / 2.0 + self.get_width() / 2.0,
            pos.y - 10.0,
            TextParams {
                font_size,
                color: WHITE,
                ..Default::default()
            },
        );
        draw_rectangle(pos.x, pos.y, self.get_width(), head_height, BEIGE);
    }

    //Get min and max extents
    fn get_min_max(&self, position: Vec2) -> (Vec2, Vec2) {
        let min = Vec2 {
            x: position.x,
            y: position.y + self.get_height(),
        };
        let max = Vec2 {
            x: position.x + self.get_width(),
            y: position.y,
        };

        (min, max)
    }

    const fn get_height(&self) -> f32 {
        20.0
    }

    const fn get_width(&self) -> f32 {
        10.0
    }
}

pub struct Building {
    pub position: Vec2,
    pub width: f32,
    pub height: f32,
    pub colour: Color,
}

impl Building {
    fn draw(&self, offset: Vec2) {
        let pos = self.position + offset;
        draw_rectangle(pos.x, pos.y, self.width, self.height, self.colour);
    }

    //Get min and max extents
    fn get_min_max(&self) -> (Vec2, Vec2) {
        let min = Vec2 {
            x: self.position.x,
            y: self.position.y + self.height,
        };
        let max = Vec2 {
            x: self.position.x + self.width,
            y: self.position.y,
        };

        (min, max)
    }
}

//Mostly for testing out audio system before implementing chat etc
pub struct Audio {
    pub position: Vec2,
    pub radius: f32,
    pub sound: audio::Sound,
}

impl Audio {
    fn draw(&self, offset: Vec2) {
        let pos = self.position + offset;
        draw_circle(pos.x, pos.y, self.radius, GREEN);
    }

    pub fn play(&self) {
        audio::play_sound_once(&(self.sound));
    }
}

//To check while loading, check for error etc
#[derive(Clone)]
pub enum GameReadiness {
    Ready,
    Loading,
    Error(String),
}

//Shareable game state used by the network and the GameObject
//The idea is that info is copied to the gameobject before it does e.g collision calculation,
//freeing up the state to be usd by the network
pub struct GameState {
    pub spawn: NetPosition,
    pub own_player: u8,
    pub ready: GameReadiness,
    pub players: HashMap<u8, NetPlayer>,
    pub buildings: Vec<NetBuilding>,
}

impl Default for GameState {
    fn default() -> GameState {
        GameState {
            ready: GameReadiness::Loading,
            spawn: NetPosition { x: 0.0, y: 0.0 },
            own_player: 0,
            // ready: GameReadiness::Loading,
            players: HashMap::new(),
            buildings: Vec::new(),
        }
    }
}

pub struct GameObject {
    pub ready: GameReadiness,
    pub spawn: Vec2,
    pub own_player: u8,
    pub players: HashMap<u8, Player>,
    pub buildings: Vec<Building>,
    pub audio_sources: Vec<Audio>,
}

impl GameObject {
    //Move the player by moving the world, pass in current player pos
    pub fn draw(&self, player_pos: Vec2) {
        let centre = Vec2 {
            x: screen_width() / 2.0,
            y: screen_height() / 2.0,
        };
        let diff = centre - player_pos;

        for i in &self.buildings {
            i.draw(diff);
        }

        for i in &self.audio_sources {
            i.draw(diff);
        }

        for (n, i) in &self.players {
            //Draw other players
            if n != &self.own_player {
                i.draw(diff);
            }
        }

        //Draw own player on top
        let op = self.players.get(&self.own_player).unwrap();
        op.draw(diff);
    }

    //Set audio effects on all sources
    pub fn resolve_audio(&mut self) {
        let base_intensity = 0.7;
        for a in &self.audio_sources {
            //Get dist from player
            let mut op = self.players.get_mut(&self.own_player).unwrap();
            let dist = op.position.distance(a.position) / 300.0;

            set_sound_volume(&a.sound, base_intensity / (dist * dist).max(1.0));
        }
    }

    //Return position of player after resolving collision
    //Will eventually return closer position if collision detected instead of 'cancelling'
    pub fn resolve_collide(&self, player: &Player, player_pos: Vec2) -> Vec2 {
        //Calculate min and max extents of player
        let (min_player, max_player) = player.get_min_max(player_pos);
        let mut ret_value = player_pos;

        for i in &self.buildings {
            let (min, max) = i.get_min_max();
            let d1x = min.x - max_player.x;
            let d1y = max_player.y - min.y;
            let d2x = min_player.x - max.x;
            let d2y = max.y - min_player.y;

            //Check if definitely not colliding
            if d1x > 0.0 || d1y > 0.0 {
                continue;
            }

            if d2x > 0.0 || d2y > 0.0 {
                continue;
            }

            //Check origin of movement

            let (omin_player, omax_player) = player.get_min_max(player.position);

            let od1x = min.x - omax_player.x;
            let od1y = omax_player.y - min.y;
            let od2x = omin_player.x - max.x;
            let od2y = max.y - omin_player.y;

            //If player was previously in x bounds safely, then it's the x change responsible for collision
            //So disable x component, keep y (checked afterwards)
            if od1x > 0.0 || od2x > 0.0 {
                ret_value.x = player.position.x;
            }

            if od1y > 0.0 || od2y > 0.0 {
                ret_value.y = player.position.y;
            }

            //Don't return value yet, disabling one movement component but keeping another could interfere with other object collisions
        }

        //Return requested position, possibly with some axis movement reset (for sliding along walls)
        ret_value
    }

    pub fn add_player(&mut self, player: Player) {
        self.players.insert(player.id, player);
    }
}

impl Default for GameObject {
    fn default() -> GameObject {
        GameObject {
            spawn: Vec2 { x: 0.0, y: 0.0 },
            own_player: 0,
            ready: GameReadiness::Loading,
            players: HashMap::new(),
            buildings: Vec::new(),
            audio_sources: Vec::new(),
        }
    }
}
