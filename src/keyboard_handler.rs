use std::io::stdin;
use termion::event::Key;
use std::thread;
use std::sync::mpsc;
use termion::input::TermRead;

pub enum KeyEvent {
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    QKey,
}

pub fn handle_keyboard(keyboard_events_tx: mpsc::Sender<KeyEvent>) {

    let _t = thread::spawn(move || {
        let stdin = stdin();
        for c in stdin.keys() {
            match c.unwrap() {
                Key::Char('q') => { keyboard_events_tx.send(KeyEvent::QKey).unwrap() },
                Key::Up => { keyboard_events_tx.send(KeyEvent::ArrowUp).unwrap() },
                Key::Down => { keyboard_events_tx.send(KeyEvent::ArrowDown).unwrap() },
                Key::Left => { keyboard_events_tx.send(KeyEvent::ArrowLeft).unwrap() },
                Key::Right => { keyboard_events_tx.send(KeyEvent::ArrowRight).unwrap() },
                _ => {}
            }
        }
    });
}