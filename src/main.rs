#![allow(dead_code)]
use std::error::Error;
use std::sync::{Arc, Mutex};
use midir::MidiOutputConnection;
mod ascii;
mod midi;
mod clock;
mod interpreter;

#[tokio::main]
/// Entry point of the program
async fn main() -> Result<(), Box<dyn Error>> {
    println!("{}", ascii::BANNER);
    // Starting Ableton Link
    let clock = Arc::new(Mutex::new(clock::AbeLinkState::new()));

    // Setting up a MIDI connexion
    let mut conn_out: Arc<Mutex<Option<MidiOutputConnection>>> = Arc::new(Mutex::new(None));
    conn_out = midi::setup_midi_connection(conn_out);
    let mut interpreter = interpreter::Interpreter::new();
    let _ = interpreter.run();
    Ok(())
}
