#![allow(dead_code)]
use std::error::Error;
use mlua::Lua;
use std::sync::{Arc, Mutex};
use midir::MidiOutputConnection;
use mlua::Result as LuaResult;
mod ascii;
mod midi;
mod clock;
mod interpreter;
mod config;

#[tokio::main]
/// Entry point of the program
async fn main() -> Result<(), Box<dyn Error>> {
    let cfg: config::EremitConfig = confy::load("eremit", None)?;
    println!("{}", ascii::BANNER);

    let clock = Arc::new(Mutex::new(clock::AbeLinkState::new()));
    // clock.lock().unwrap().run().await?;

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
    let _ = interpreter.register_function("start", move|lua, ()| {
        let clock = clock.lock().unwrap();
        clock.link.enable(true);
        Ok(())
    });

    // Expose Unix Time
    fn get_unix_time(_lua: &Lua, _args: ()) -> LuaResult<f64> {
        Ok(clock::current_unix_time().as_secs_f64())
    }
    let _ = interpreter.register_function("unix", get_unix_time);

    let _ = interpreter.run();
    println!("{}", ascii::GOODBYE);
    Ok(())
}