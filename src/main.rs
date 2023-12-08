#![allow(dead_code)]
use std::{error::Error, thread::current};
use mlua::Lua;
use std::sync::{Arc, Mutex};
use midir::MidiOutputConnection;
use mlua::Result as LuaResult;

use crate::clock::AbeLinkState;
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

    // Registering somme dummy functions as a test!
    fn test_function(_lua: &Lua, _args: ()) -> LuaResult<()> {
        println!("Hello from Rust!");
        Ok(())
    }
    let _ = interpreter.register_function("baba", test_function);
    let _ = interpreter.register_value("clock", 1);

    let _ = interpreter.run();
    println!("{}", ascii::GOODBYE);
    Ok(())
}