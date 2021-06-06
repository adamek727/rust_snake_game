extern crate termion;

use std::sync::mpsc;

use rust_snake_game::keyboard_handler::handle_keyboard;
use rust_snake_game::snake_engine::run_game;

fn main() {
    let (keyboard_events_tx, keyboard_events_rx) = mpsc::channel();
    handle_keyboard(keyboard_events_tx);
    run_game(keyboard_events_rx);
}