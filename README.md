# MacroProx

MacroProx is a multiplayer game built in Rust using [Macroquad](https://macroquad.rs/). The idea is to create a basic 2D world with proximity chat, with buildings to run into and possibly items to interact with one day. For the time being, it's more an experiment with proximity chat.

Any optimisations and general improvements to the codebase will eventually be reflected on this repo. Several things aren't ideal at the moment.

![MacroProx Image](https://github.com/Ronan-vdv/MacroProx/blob/master/GHAssets/MP1.png?raw=true)

# Features

The main purposes of this project are:

- To learn Rust
- To experiment with multiplayer game architecture
- To implement proximity chat and possibly add voice effects (e.g being in a room adds an echo)

## Implemented Features

- Client/server architecture is working. Can run a server and connect multiple clients
- Rectangular collision detection. Collision detection for complex polygons may or may not happen in the future
- The map loaded by the server is sent to the client over the network

## Planned Features

- Message queue to display on screen (player joins, leaves etc)
- Interpolation between states for other players, to emulate smooth movement with fewer updates instead of constantly sharing changed state over the network almost every frame (likely to be inspired by [Valve's architecture](https://developer.valvesoftware.com/wiki/Source_Multiplayer_Networking))
- And of course the actual proximity chat
- Then anything else I might feel like when I've reached those goals, like items or events or whatever

## Running

You should be able to run this with an installation of Rust 1.76 or higher. `cargo run` should download all dependencies, compile and run it. Tested on Ubuntu 20.04.
