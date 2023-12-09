#![allow(dead_code)]
use std::clone;
use std::error::Error;
use mlua::Lua;
use std::sync::{Arc, Mutex};
use std::error::Error as StdError;
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
    let clock = Arc::new(Mutex::new(clock::AbeLinkState::new()));
    let mut abe_link_state = clock::AbeLinkState::new();
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
    let _ = interpreter.register_function("snapshot", {
        let cloned_clock = Arc::clone(&clock);
        move |_lua: &Lua, _args: ()| -> LuaResult<String> {
        cloned_clock.lock().unwrap().make_snapshot();
        let clock_mutex = cloned_clock.lock().unwrap();
        let snapshot = &clock_mutex.snapshot;
        match snapshot {
            Some(s) => {
                let tempo = s.tempo; let beat = s.beats; let phase = s.phase;
                let result = String::from("{ tempo = ") + &tempo.to_string() + ", beat = " + &beat.to_string() + ", phase = " + &phase.to_string() + " }";
                Ok(result)
            }
            None => Err(mlua::Error::RuntimeError("No snapshot available".to_string()))
        }
    }});
    let _ = interpreter.register_function("set_tempo", {
        let cloned_clock = Arc::clone(&clock);
        move |_lua: &Lua, args: (f64,)| -> LuaResult<()> {
            let mut clock_mutex = cloned_clock.lock().unwrap();
            clock_mutex.set_tempo(args.0);
            Ok(())
        }
    });
    let _ = interpreter.register_function("sync", {
        let cloned_clock = Arc::clone(&clock);
        move |_lua: &Lua, _args: ()| -> LuaResult<()> {
            let mut clock_mutex = cloned_clock.lock().unwrap();
            clock_mutex.sync();
            Ok(())
        }
    });
    let _ = interpreter.register_function("peers", {
        let cloned_clock = Arc::clone(&clock);
        move |_lua: &Lua, _args: ()| -> LuaResult<u64> {
            let clock_mutex = cloned_clock.lock().unwrap();
            Ok(clock_mutex.peers())
        }
    });
    let _ = interpreter.register_function("phase", {
        let clock = Arc::clone(&clock);
        move |_lua: &Lua, _args: ()| -> LuaResult<f64> {
            let mut guard = clock.lock().unwrap();
            guard.make_snapshot();
            Ok(guard.snapshot.as_ref().unwrap().phase)
        }
    });
    let _ = interpreter.register_function("beat", {
        let clock = Arc::clone(&clock);
        move |_lua: &Lua, _args: ()| -> LuaResult<f64> {
            let mut guard = clock.lock().unwrap();
            guard.make_snapshot();
            Ok(guard.snapshot.as_ref().unwrap().beats)
        }
    });
    let _ = interpreter.register_function("unix", |_lua: &Lua, _args: ()| -> LuaResult<f64> {
        Ok(clock::current_unix_time().as_secs_f64())
    });
    let _ = interpreter.run();
    println!("{}", ascii::GOODBYE);
    clock.lock().unwrap().running = false;
    Ok(())
}