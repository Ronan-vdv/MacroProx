use game::{GameObject, GameReadiness, GameState, Player};
use macroquad::audio::Sound;
use macroquad::telemetry::frame;
use macroquad::ui::{hash, root_ui};
use macroquad::{
    audio::{self, set_sound_volume},
    prelude::*,
};
use maps::load_map_1;
use menu::{main_menu, GameSettings, GameType};
use net_common::{NetBuilding, NetPlayer, NetPosition};
use std::sync::{Arc, Mutex};
use std::thread;
mod client;
mod game;
mod menu;
mod net_common;
mod server;
mod maps;

//Load map into object
async fn load_game_map(state_lock: &Arc<Mutex<GameState>>, player: &NetPlayer) {
    load_map_1(state_lock, player);
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

#[macroquad::main("MacroProx")]
async fn main() {

    //Initial things
    set_pc_assets_folder("assets");
    const BASESPEED:f32 = 250.0;

    // let font = load_ttf_font("./assets/fonts/Raleway-SemiBold.ttf").await.unwrap();

    let mut settings = GameSettings {
        ..Default::default()
    };
    main_menu(&mut settings).await;


    let mut game = GameObject {
        ..Default::default()
    };
    let game_state = GameState {
        spawn: NetPosition {
            x: screen_width() / 2.0,
            y: screen_height() / 2.0,
        },
        ..Default::default()
    };
    let mut state_lock = Arc::new(Mutex::new(game_state));
    let thread_mutex = Arc::clone(&state_lock);

    match settings.game_type {
        GameType::Host => {
            //Add first player
            let me = NetPlayer {
                colour: settings.player_colour,
                id: 0,
                name: settings.player_name.clone(),
                position: NetPosition { x: 0.0, y: 0.0 },
            };

            load_game_map(&state_lock, &me).await;
            //Start host
            thread::spawn(move || {
                server::run_host(thread_mutex);
            });
        }
        GameType::Client => {

            //Start client
            thread::spawn(move || {
                client::run_client(settings, thread_mutex);
            });
            load_client(&state_lock).await;
        }
    }

    //Start game

    loop {
        let delta = frame().full_frame_time;
        game_from_state(&mut game, &state_lock); //Load latest state to game

        let mut speed = BASESPEED * delta;
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
