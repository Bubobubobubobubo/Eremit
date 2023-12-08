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
    println!("{}", ascii::BANNER);
    let cfg: config::EremitConfig = confy::load("eremit", None)?;
    // let clock = Arc::new(Mutex::new(clock::AbeLinkState::new()));
    let mut abe_link_state = clock::AbeLinkState::new();
    // Use tokio::spawn to run the run method concurrently
    let abe_link_handle = tokio::spawn(async move {
        abe_link_state.run().await.unwrap_or_else(|err| {
            eprintln!("Error in AbeLinkState run: {:?}", err);
        });
    });

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
    fn get_unix_time(_lua: &Lua, _args: ()) -> LuaResult<f64> {
        Ok(clock::current_unix_time().as_secs_f64())
    }
    let _ = interpreter.register_function("unix", get_unix_time);

    let _ = interpreter.run();
    println!("{}", ascii::GOODBYE);
    Ok(())
}