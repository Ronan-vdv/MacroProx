//Menus for the game
//Men pages call each other recursively to go forward, unwind to go back

use std::net::Ipv4Addr;
use std::ops::Range;
use std::str::FromStr;

use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui};

use crate::net_common::NetColour;

//Type of game to start
pub enum GameType {
    Host,
    Client,
}

//Settings to be modified by menus
pub struct GameSettings {
    pub game_type: GameType,
    pub port: u16,
    pub player_name: String,
    pub player_colour: NetColour,
    pub host: Option<Ipv4Addr>, //For use by client
}

impl Default for GameSettings {
    fn default() -> GameSettings {
        GameSettings {
            game_type: GameType::Host,
            host: None,
            player_colour: NetColour::from_col(WHITE),
            player_name: String::from("Player 1"),
            port: 0,
        }
    }
}

//Result returned by a menu
pub struct MenuResult {
    //If true, the menu returning the value wants the previous menu to exit
    //Used if the menu trail is followed to an actual game start, so all the menus end
    pub should_exit: bool,
}

//Main menu of the game at startup
pub async fn main_menu(settings: &mut GameSettings) -> MenuResult {
    let size = Vec2 { x: 200.0, y: 100.0 };
    loop {
        let position = Vec2 {
            //Set position on screen
            x: screen_width() / 2.0 - size.x / 2.0,
            y: screen_height() / 2.0 - size.y / 2.0,
        };

        clear_background(GRAY);

        let mut set = false;
        let mut host = false;

        root_ui().window(hash!(), position, size, |ui| {
            //Create buttons and check if they get clicked
            if ui.button(None, "Host") {
                set = true;
                host = true;
            }

            if ui.button(None, "Connect") {
                set = true;
            }
        });

        if set {
            //Load next menu, then return result
            let ret: MenuResult;
            if host {
                ret = host_game(settings).await;
            } else {
                ret = client_game(settings).await;
            }

            //A game has been selected, get rid of menu before game starts
            if ret.should_exit {
                return ret;
            }
            //If not, then loop and show menu again
        }
        next_frame().await;
    }
}

//Menu to set up host
pub async fn host_game(settings: &mut GameSettings) -> MenuResult {
    let size = Vec2 { x: 300.0, y: 500.0 };

    let mut name = String::new();
    let mut port = String::from("5508");
    let mut col_r = 1.0 / 200.0;
    let mut col_g = 1.0 / 200.0;
    let mut col_b = 1.0 / 200.0;

    let mut col = NetColour {
        r: col_r,
        g: col_g,
        b: col_b,
        a: 1.0,
    };
    let mut portnum = port.parse::<u16>().unwrap();
    let mut ret = MenuResult { should_exit: true };

    loop {
        let position = Vec2 {
            x: screen_width() / 2.0 - size.x / 2.0,
            y: screen_height() / 2.0 - size.y / 2.0,
        };

        col.r = col_r;
        col.g = col_g;
        col.b = col_b;

        clear_background(col.to_col());

        let mut set = false;
        let mut back = false;

        root_ui().window(hash!(), position, size, |ui| {
            //Create buttons and check if they get clicked

            ui.input_text(hash!(), "Display Name", &mut name);
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
                &mut col_r,
            );
            ui.slider(
                hash!(),
                "Green",
                Range {
                    start: 0.0,
                    end: 1.0,
                },
                &mut col_g,
            );
            ui.slider(
                hash!(),
                "Blue",
                Range {
                    start: 0.0,
                    end: 1.0,
                },
                &mut col_b,
            );

            if ui.button(
                Vec2 {
                    x: 0.,
                    y: size.y - 30.0,
                },
                "Back",
            ) {
                set = true;
                back = true;
            }

            if ui.button(
                Vec2 {
                    x: 80.0,
                    y: size.y - 30.0,
                },
                "Apply",
            ) {
                if !name.is_empty() && !port.is_empty() {
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
            ret.should_exit = !back;
            break;
        }
    }

    settings.player_colour = col;
    settings.player_name = name;
    settings.game_type = GameType::Host;

    return ret;
}

//Menu to set up client
pub async fn client_game(settings: &mut GameSettings) -> MenuResult {
    let size = Vec2 { x: 300.0, y: 500.0 };

    let mut name = String::new();
    let mut ip = String::from("127.0.0.1");
    let mut port = String::from("5508");
    let mut col_r = 1.0 / 200.0;
    let mut col_g = 1.0 / 200.0;
    let mut col_b = 1.0 / 200.0;

    let mut col = NetColour {
        r: col_r,
        g: col_g,
        b: col_b,
        a: 1.0,
    };
    let mut portnum = port.parse::<u16>().unwrap();
    let mut address = Ipv4Addr::from_str(&ip).unwrap();
    let mut ret = MenuResult { should_exit: true };

    loop {
        let position = Vec2 {
            x: screen_width() / 2.0 - size.x / 2.0,
            y: screen_height() / 2.0 - size.y / 2.0,
        };

        col.r = col_r;
        col.g = col_g;
        col.b = col_b;

        clear_background(col.to_col());

        let mut set = false;
        let mut back = false;

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
                &mut col_r,
            );
            ui.slider(
                hash!(),
                "Green",
                Range {
                    start: 0.0,
                    end: 1.0,
                },
                &mut col_g,
            );
            ui.slider(
                hash!(),
                "Blue",
                Range {
                    start: 0.0,
                    end: 1.0,
                },
                &mut col_b,
            );

            if ui.button(
                Vec2 {
                    x: 0.,
                    y: size.y - 30.0,
                },
                "Back",
            ) {
                set = true;
                back = true;
            }

            if ui.button(
                Vec2 {
                    x: 80.0,
                    y: size.y - 30.0,
                },
                "Apply",
            ) {
                if !ip.is_empty() && !name.is_empty() && !port.is_empty() {
                    let s = Ipv4Addr::from_str(&ip);

                    if s.is_ok() {
                        //Check if ip valid first (port made valid while typing)
                        address = s.unwrap();
                        set = true;
                    } else {
                        ip = address.to_string();
                    }
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
            ret.should_exit = !back;
            break;
        }
    }

    settings.port = portnum;
    settings.player_colour = col;
    settings.player_name = name;
    settings.game_type = GameType::Client;
    settings.host = Some(address);

    return ret;
}
