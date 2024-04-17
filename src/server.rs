use super::game;
use super::net_common;
use crate::net_common::Commands;
use crate::net_common::Map;
use crate::net_common::NetPlayer;
use crate::net_common::NetPosition;
use crate::net_common::PositionMap;
use message_io::network::Endpoint;
use message_io::network::{NetEvent, Transport};
use message_io::node::NodeEvent;
use message_io::node::{self};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// struct PlayerConnections {
//     con_endpoint: Endpoint,
// }

enum Signal {
    UpdateClients,
}

//Run a listener for any new connections
pub fn run_host(state_lock: Arc<Mutex<game::GameState>>) {
    let (handler, listener) = node::split::<Signal>();
    let mut clients: HashMap<Endpoint, u8> = HashMap::new();
    let mut player_count = 1; //Start from index 1, since index 0 is own player

    // Listen for TCP, UDP and WebSocket messages at the same time.
    handler
        .network()
        .listen(Transport::FramedTcp, "0.0.0.0:5508")
        .unwrap();
    // handler
    //     .network()
    //     .listen(Transport::Udp, "0.0.0.0:3043")
    //     .unwrap();

    let mut positions: HashMap<u8, NetPosition> = HashMap::new();

    handler
        .signals()
        .send_with_timer(Signal::UpdateClients, Duration::from_millis(500)); //Wait then send signal to start client update loop

    println!("Running");
    // Read incoming network events.
    listener.for_each( move  |event| match event {
        NodeEvent::Network(net_event) => match net_event {
            NetEvent::Connected(_, _) => unreachable!(), // Used for explicit connections.
            NetEvent::Accepted(_endpoint, _listener) => {
                let state = state_lock.lock().unwrap();
                println!("Client connected");
                //TODO: Mutex?
                clients.insert(_endpoint, player_count);
                player_count += 1;
                drop(state);
                
            }
            NetEvent::Message(endpoint, data) => {
                let res = bincode::deserialize(&data);

                match res {
                    Ok(dat) => {
                        match dat {
                            Commands::Move(dat) => {
                                let mut state = state_lock.lock().unwrap();
                                let player_num = clients.get(&endpoint).unwrap();
                                let t = state.players.get_mut(player_num);
                                match t {
                                    None => {
                                        println!("Trying to move non-existent player with unlinked endpoint!");
                                    },
                                    Some(player) => {
                                        player.position.x = dat.x;
                                        player.position.y = dat.y;
                                    }
                                }
                                drop(state);
                            }
                            Commands::SendMap(_) => (), //Not for server
                            Commands::SendPlayerInfo(_) => (), //Not for server
                            Commands::AllowClientReady(_) => (), //Not for server
                            Commands::MovedPlayers(_) => (), //Not for server
                            Commands::AddPlayer(_) => (), //Not for server
                            Commands::RemovePlayer(_) => (), //Not for server
                            Commands::RegisterPlayer(player_info) => {
                                
                                println!("Attempting to register");
                                let mut state = state_lock.lock().unwrap();
                                let id = clients.get(&endpoint).unwrap();
                                let new_player = NetPlayer {
                                    colour: player_info.colour,
                                    id: *id,
                                    name: player_info.name,
                                    position: state.spawn,
                                };
                                state.players.insert(*id, new_player.clone());

                                //Serialise now to avoid borrowing issues
                                let np_serial = bincode::serialize(&Commands::AddPlayer(new_player)).unwrap();
                
                                //Send info to new client               
                                let mut players = Vec::new();
                                for (_, p) in &state.players {
                                    players.push((*p).clone());
                                }
                                let tosendb = bincode::serialize(&Commands::SendMap(Map { buildings: state.buildings.clone() })).unwrap();
                                let tosendp =
                                    bincode::serialize(&Commands::SendPlayerInfo(net_common::NetPlayerInfo {
                                        players,//: state.players,
                                        your_num: *id,
                                    }))
                                    .unwrap();
                                //Serialised, now inc player count and release lock
                                
                                drop(state);
                
                                //Send map and players
                                let _status = handler.network().send(endpoint, &tosendb);
                                let _status = handler.network().send(endpoint, &tosendp);
                
                                //Update other clients
                                for (c, _) in &clients {
                                    if *c != endpoint {
                                        let _status = handler.network().send(*c, &np_serial);                        
                                    }
                                }
                
                                //Client ready
                                let tosend = bincode::serialize(&Commands::AllowClientReady(*id)).unwrap();
                                let _status = handler.network().send(endpoint, &tosend);
                            }, //Add a new player
                        }
                        // let state = state_lock.lock().unwrap();
                        //TODO: Change position of specific player
                        // state.position.x = dat.pX;
                        // state.position.y = dat.pY;
                        // drop(state);
                    }
                    Err(err) => {
                        println!("{}", err);
                    }
                }
            }
            NetEvent::Disconnected(_endpoint) => {
                //TODO: Notify other clients
                let p = clients.get(&_endpoint).unwrap();
                let mut game = state_lock.lock().unwrap();
                game.players.remove(&p); //Remove player from game
                drop(game);
                let tosend = bincode::serialize(&Commands::RemovePlayer(*p)).unwrap();
                clients.remove(&_endpoint); //Remove client from list of endpoints

                for (c, _) in &clients {
                    handler.network().send(*c, &tosend);   
                }
                println!("Client disconnected");
            } //Tcp or Ws
        },
        NodeEvent::Signal(signal) => match signal  {
            Signal::UpdateClients =>  {
                //Try and update clients
                let mut new_positions: Vec<PositionMap> = Vec::new();

                let game = state_lock.lock().unwrap();
                for (id, player) in &game.players {
                    if !positions.contains_key(id)
                        || !player.position.equals(positions.get(&id).unwrap())
                    {
                        positions.insert(*id, player.position); //Update position
                        new_positions.push(PositionMap {
                            id: *id,
                            pos: player.position,
                        }); //Add to list to send to clients
                    }
                }

                // if !positions.contains_key(&0)
                //     || game.own_player.position != *positions.get(&0).unwrap()
                // {
                //     //Add own player
                //     positions.insert(0, game.own_player.position);
                //     new_positions.push(PositionMap {
                //         id: 0,
                //         pos: NetPosition::from_vec2(game.own_player.position),
                //     });
                // }

                drop(game); //Release lock before send

                let tosend = bincode::serialize(&Commands::MovedPlayers(new_positions)).unwrap();
                // let mut handles = Vec::new();

                if !tosend.is_empty() {
                    for (i, _) in &clients { //Will this work?
                        // let _status = handler.network().send(*i, &tosend);
                        //  handles.push(async {
                        // }) ;
                        handler.network().send(*i, &tosend);
                    }
                }
                // for i in handles {
                //     // i.await;
                // }

                handler.signals().send_with_timer(Signal::UpdateClients, Duration::from_millis(15)); //Wait before next update
            }
        },
    });
}
