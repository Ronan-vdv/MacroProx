//Will eventually be replaced by something like textfiles

use std::sync::{Arc, Mutex};

use macroquad::color::{ORANGE, WHITE};

use crate::{game::{GameReadiness, GameState}, net_common::{NetBuilding, NetColour, NetPlayer, NetPosition}};

pub fn load_map_1(state_lock: &Arc<Mutex<GameState>>, player: &NetPlayer) {
    let mut game = state_lock.lock().unwrap();

    let mut p = player.clone();
    p.position = game.spawn;

    game.players.insert(player.id, p);
    //TODO: Make generic eventually
    game.buildings.push(NetBuilding {
        position: NetPosition { x: 5.0, y: 5.0 },
        width: 50.0,
        height: 20.0,
        colour: NetColour::from_col(WHITE),
    });
    game.buildings.push(NetBuilding {
        position: NetPosition { x: 798.0, y: 15.0 },
        width: 80.0,
        height: 10.0,
        colour: NetColour::from_col(ORANGE),
    });
    game.buildings.push(NetBuilding {
        position: NetPosition { x: 436.0, y: 70.0 },
        width: 100.0,
        height: 54.0,
        colour: NetColour::from_col(WHITE),
    });
    game.buildings.push(NetBuilding {
        position: NetPosition { x: 55.0, y: 58.0 },
        width: 10.0,
        height: 68.0,
        colour: NetColour::from_col(WHITE),
    });
    game.buildings.push(NetBuilding {
        position: NetPosition { x: 846.0, y: 375.0 },
        width: 90.0,
        height: 24.0,
        colour: NetColour::from_col(WHITE),
    });
    game.buildings.push(NetBuilding {
        position: NetPosition { x: 600.0, y: 458.0 },
        width: 120.0,
        height: 14.0,
        colour: NetColour::from_col(WHITE),
    });
    game.buildings.push(NetBuilding {
        position: NetPosition {
            x: 140.0,
            y: 9534.0,
        },
        width: 200.0,
        height: 19.0,
        colour: NetColour::from_col(WHITE),
    });
    game.buildings.push(NetBuilding {
        position: NetPosition { x: 20.0, y: 79.0 },
        width: 205.0,
        height: 94.0,
        colour: NetColour::from_col(WHITE),
    });

    // let mut sounds: Vec<(Vec2, _)> = Vec::new(); //Try to load concurrently
    // sounds.push((
    //     Vec2 {
    //         x: screen_width() / 2.0 - 50.0,
    //         y: screen_height() / 2.0 + 20.0,
    //     },
    //     audio::load_sound("ThePretender.wav"),
    // ));

    // for (p, s) in sounds {
    //     game.audio_sources.push(game::Audio {
    //         position: p,
    //         radius: 10.0,
    //         sound: s.await.unwrap(),
    //     });
    // }

    // for a in &game.audio_sources {
    //     a.play();
    // }

    game.ready = GameReadiness::Ready;

    drop(game);
}