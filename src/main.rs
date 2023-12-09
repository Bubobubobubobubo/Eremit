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
use std::thread;

fn main() -> Result<(), Box<dyn Error>> {
    println!("{}", ascii::BANNER);
    let cfg: config::EremitConfig = confy::load("eremit", None)?;
    let clock = Arc::new(Mutex::new(clock::Clock::new()));
    let clock_clone = Arc::clone(&clock);
    thread::spawn(move || {
        let mut clock_mutex = clock_clone.lock().unwrap();
        let _ = clock_mutex.run();
    });
    let mut conn_out: Arc<Mutex<Option<MidiOutputConnection>>> = Arc::new(Mutex::new(None));
    conn_out = midi::setup_midi_connection(conn_out);
    let mut interpreter = interpreter::Interpreter::new();
    let _ = interpreter.register_function("report", {
        let cloned_clock = Arc::clone(&clock);
            move |_lua: &Lua, _args: ()| -> LuaResult<()> {
                cloned_clock.lock().unwrap().report();
            Ok(())
        }
    });
    // let _ = interpreter.register_function("tempo", {
    //     let cloned_clock = Arc::clone(&abe_link_state);
    //     move |_lua: &Lua, _args: ()| -> LuaResult<f64> {
    //         let mut clock_mutex = cloned_clock.lock().unwrap();
    //         clock_mutex.make_snapshot();
    //         Ok(clock_mutex.session_state.tempo() as f64)
    //     }
    // });
    // let _ = interpreter.register_function("set_tempo", {
    //     let cloned_clock = Arc::clone(&abe_link_state);
    //     move |_lua: &Lua, args: (f64,)| -> LuaResult<()> {
    //         let mut clock_mutex = cloned_clock.lock().unwrap();
    //         clock_mutex.set_tempo(args.0);
    //         Ok(())
    //     }
    // });
    // let _ = interpreter.register_function("sync", {
    //     let cloned_clock = Arc::clone(&abe_link_state);
    //     move |_lua: &Lua, _args: ()| -> LuaResult<()> {
    //         let mut clock_mutex = cloned_clock.lock().unwrap();
    //         clock_mutex.sync();
    //         Ok(())
    //     }
    // });
    // let _ = interpreter.register_function("peers", {
    //     let cloned_clock = Arc::clone(&abe_link_state);
    //     move |_lua: &Lua, _args: ()| -> LuaResult<u64> {
    //         let clock_mutex = cloned_clock.lock().unwrap();
    //         Ok(clock_mutex.peers())
    //     }
    // });
    // let _ = interpreter.register_function("phase", {
    //     let clock = Arc::clone(&abe_link_state);
    //     move |_lua: &Lua, _args: ()| -> LuaResult<f64> {
    //         let mut guard = clock.lock().unwrap();
    //         guard.make_snapshot();
    //         Ok(guard.snapshot.as_ref().unwrap().phase)
    //     }
    // });
    // let _ = interpreter.register_function("beat", {
    //     let clock = Arc::clone(&abe_link_state);
    //     move |_lua: &Lua, _args: ()| -> LuaResult<f64> {
    //         let mut guard = clock.lock().unwrap();
    //         guard.make_snapshot();
    //         Ok(guard.snapshot.as_ref().unwrap().beats)
    //     }
    // });
    // let _ = interpreter.register_function("unix", |_lua: &Lua, _args: ()| -> LuaResult<f64> {
    //     Ok(clock::current_unix_time().as_secs_f64())
    // });
    // let _ = interpreter.register_function("play", {
    //     let cloned_clock = Arc::clone(&abe_link_state);
    //     move |_lua: &Lua, _args: ()| -> LuaResult<()> {
    //         let mut clock_mutex = cloned_clock.lock().unwrap();
    //         clock_mutex.play();
    //         Ok(())
    //     }
    // });

    let _ = interpreter.run();
    println!("{}", ascii::GOODBYE);
    Ok(())
}