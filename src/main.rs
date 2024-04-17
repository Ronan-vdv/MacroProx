use game::{GameObject, GameReadiness, GameState, Player};
use macroquad::audio::Sound;
use macroquad::telemetry::frame;
use macroquad::ui::{hash, root_ui};
use macroquad::{
    audio::{self, set_sound_volume},
    prelude::*,
};
use net_common::{NetBuilding, NetColour, NetPlayer, NetPosition};

use std::ops::Range;
use std::sync::{Arc, Mutex};
use std::thread;
mod client;
mod game;
mod net_common;
mod server;

enum MainMenuOption {
    HOST,
    CONNECT,
}

async fn load_main_menu() -> MainMenuOption {
    let size = Vec2 { x: 200.0, y: 100.0 };
    let position = Vec2 {
        x: screen_width() / 2.0 - size.x / 2.0,
        y: screen_height() / 2.0 - size.y / 2.0,
    };
    loop {
        clear_background(GRAY);

        let mut ret = MainMenuOption::HOST;
        let mut set = false;

        root_ui().window(hash!(), position, size, |ui| {
            //Create buttons and check if they get clicked
            if ui.button(None, "Host") {
                set = true;
            }

            if ui.button(None, "Connect") {
                set = true;
                ret = MainMenuOption::CONNECT;
            }
        });

        next_frame().await;
        if set {
            return ret;
        }
    }
}

async fn set_up_client() -> net_common::ClientSettings //Name, target ip
{
    let size = Vec2 { x: 300.0, y: 500.0 };
    let position = Vec2 {
        x: screen_width() / 2.0 - size.x / 2.0,
        y: screen_height() / 2.0 - size.y / 2.0,
    };

    let mut name = String::new();
    let mut ip = String::from("127.0.0.1");
    let mut port = String::from("5508");
    let mut colR = 1.0 / 200.0;
    let mut colG = 1.0 / 200.0;
    let mut colB = 1.0 / 200.0;

    let mut col = Color {
        r: colR,
        g: colG,
        b: colB,
        a: 1.0,
    };
    let mut portnum = port.parse::<u16>().unwrap();
    loop {
        col.r = colR;
        col.g = colG;
        col.b = colB;
        clear_background(col);

        let mut set = false;

        root_ui().window(hash!(), position, size, |ui| {
            //Create buttons and check if they get clicked

            ui.input_text(hash!(), "Display Name", &mut name);
            ui.input_text(hash!(), "Host IP", &mut ip);
            ui.input_text(hash!(), "Port", &mut port);
            ui.separator();
            ui.separator();
            ui.separator();

            ui.label(None, "Colour");

            ui.slider(
                hash!(),
                "Red",
                Range {
                    start: 0.0,
                    end: 1.0,
                },
                &mut colR,
            );
            ui.slider(
                hash!(),
                "Green",
                Range {
                    start: 0.0,
                    end: 1.0,
                },
                &mut colG,
            );
            ui.slider(
                hash!(),
                "Blue",
                Range {
                    start: 0.0,
                    end: 1.0,
                },
                &mut colB,
            );

            if ui.button(
                Vec2 {
                    x: 80.0,
                    y: size.y - 30.0,
                },
                "Apply",
            ) {
                if !ip.is_empty() && !name.is_empty() && !port.is_empty() {
                    set = true;
                }
            }
        });

        let t = port.parse::<u16>();
        if t.is_ok() {
            portnum = t.unwrap();
        } else if !port.is_empty() {
            port = portnum.to_string();
        }

        next_frame().await;
        if set {
            break;
        }
    }

    return net_common::ClientSettings {
        name,
        ip,
        port: portnum,
        colour: col,
    };
}

//Load map into object
async fn load_game_map(state_lock: &Arc<Mutex<GameState>>, player: &NetPlayer) {
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

//Load a client game
async fn load_client(game: &Arc<Mutex<GameState>>) {
    let mut done = false;
    let mut error = false;
    let mut message = String::from("Loading Map...");

    while !done && !error {
        let game_state = game.lock().unwrap();

        //Check if done loading
        match &game_state.ready {
            GameReadiness::Error(er) => {
                message = String::from("Encountered error: ") + er;
                error = true;
            }
            GameReadiness::Loading => (),
            GameReadiness::Ready => {
                done = true;
            }
        }
        clear_background(BLACK);
        draw_text(
            &message,
            screen_width() / 2.0 - 25.0,
            screen_height() / 2.0,
            20.0,
            WHITE,
        );
        drop(game_state);
        next_frame().await;
    }

    //Force user to restart after error
    if error {
        loop {
            clear_background(BLACK);
            draw_text(
                &message,
                screen_width() / 2.0 - 25.0,
                screen_height() / 2.0,
                20.0,
                WHITE,
            );
            next_frame().await;
        }
    }
}

//Transfer state to the game object
fn game_from_state(game: &mut GameObject, state_lock: &Arc<Mutex<GameState>>) {
    game.buildings.clear();
    game.players.clear();
    let state = state_lock.lock().unwrap();
    for i in &state.buildings {
        game.buildings.push(i.to_building());
    }

    for (i, p) in &state.players {
        game.players.insert(
            *i,
            Player {
                id: *i,
                name: p.name.clone(),
                position: p.position.to_vec2(),
                colour: p.colour.to_col(),
            },
        );
    }

    game.own_player = state.own_player;
    game.ready = state.ready.clone();
    drop(state);
}

//Transfer all game object state to state
fn game_to_state(game: &GameObject, state_lock: &Arc<Mutex<GameState>>) {
    let mut state = state_lock.lock().unwrap();
    state.buildings.clear();
    state.players.clear();
    for i in &game.buildings {
        state.buildings.push(NetBuilding::from_building(i));
    }

    for (i, p) in &game.players {
        state.players.insert(*i, NetPlayer::from_player(p));
    }

    state.own_player = game.own_player;
    drop(state);
}

//Transfer game object state to state, that was changed by self (don't overwrite client data if server)
fn own_changes_to_state(game: &GameObject, state_lock: &mut Arc<Mutex<GameState>>) {
    let player = game.players.get(&game.own_player).unwrap();
    let mut state = state_lock.lock().unwrap();
    state
        .players
        .insert(game.own_player, NetPlayer::from_player(&player));

    drop(state);
}

#[macroquad::main("Testing Some Things")]
async fn main() {
    set_pc_assets_folder("assets");
    let base_speed = 250.0;

    // let font = load_ttf_font("./assets/fonts/Raleway-SemiBold.ttf").await.unwrap();

    let player_name = String::from("Player 1"); //Input box eventually

    let game_type = load_main_menu().await;
    let mut game = GameObject {
        ..Default::default()
    };
    let game_state = GameState {
        spawn: NetPosition { x: screen_width()/2.0, y: screen_height()/2.0 },
        ..Default::default()
    };
    let mut state_lock = Arc::new(Mutex::new(game_state));
    let thread_mutex = Arc::clone(&state_lock);

    match game_type {
        MainMenuOption::HOST => {
            //Add first player
            let me = NetPlayer {
                colour: NetColour::from_col(YELLOW),
                id: 0,
                name: player_name,
                position: NetPosition{x: 0.0, y: 0.0},
            };

            load_game_map(&state_lock, &me).await;
            //Start host
            thread::spawn(move || {
                server::run_host(thread_mutex);
            });
        }
        MainMenuOption::CONNECT => {
            let client_settings = set_up_client().await;

            //Start client
            thread::spawn(move || {
                client::run_client(client_settings, thread_mutex);
            });
            load_client(&state_lock).await;
        }
    }

    //Start game

    loop {
        let delta = frame().full_frame_time;
        game_from_state(&mut game, &state_lock); //Load latest state to game

        let mut speed = base_speed * delta;
        let mut movement_vec = Vec2 { x: 0.0, y: 0.0 };

        //Sprint
        if is_key_down(KeyCode::LeftShift) {
            speed *= 2.0;
        }

        if is_key_down(KeyCode::A) {
            movement_vec.x -= 1.0;
        }

        if is_key_down(KeyCode::D) {
            movement_vec.x += 1.0;
        }

        if is_key_down(KeyCode::W) {
            movement_vec.y -= 1.0;
        }

        if is_key_down(KeyCode::S) {
            movement_vec.y += 1.0;
        }

        match (&mut game).ready {
            //Check for any errors
            GameReadiness::Error(ref er) => {
                clear_background(BLACK);
                draw_text(
                    &er,
                    screen_width() / 2.0 - 25.0,
                    screen_height() / 2.0,
                    20.0,
                    WHITE,
                );
            }
            GameReadiness::Loading => (),
            GameReadiness::Ready => {
                // println!("I am {} out of {} players", game.own_player, game.players.len());
                let player = game.players.get(&game.own_player).unwrap();
                //Normalise movement to ensure diagonal movement speed is the same as other directions
                movement_vec = movement_vec.normalize();
                let mut pos = player.position;
                if !movement_vec.is_nan() {
                    pos += movement_vec * speed;
                }

                pos = game.resolve_collide(player, pos);
                // game.resolve_audio();
                let player = game.players.get_mut(&game.own_player).unwrap();
                player.position = pos;

                clear_background(BLACK);
                game.draw(pos);

                //Write new changes to state
                own_changes_to_state(&game, &mut state_lock);
            }
        }

        next_frame().await;
    }
}
