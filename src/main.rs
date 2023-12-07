use mlua::{Error, Lua, MultiValue};
use rustyline::DefaultEditor;
use std::error::Error as OtherError;
use std::io::{stdin, stdout, Write};
use std::thread::sleep;
use std::time::Duration;
use midir::{MidiOutput, MidiOutputPort, MidiOutputConnection};
use mlua::{Result as LuaResult};
use std::sync::{Arc, Mutex};
use rusty_link::{AblLink, SessionState};

pub struct State {
    pub link: AblLink,
    pub session_state: SessionState,
    pub running: bool,
    pub quantum: f64,
}

impl State {
    pub fn new() -> Self {
        Self {
            link: AblLink::new(120.),
            session_state: SessionState::new(),
            running: true,
            quantum: 4.,
        }
    }

    pub fn capture_app_state(&mut self) {
        self.link.capture_app_session_state(&mut self.session_state);
    }

    pub fn commit_app_state(&mut self) {
        self.link.commit_app_session_state(&self.session_state);
    }
}


fn setup_midi() -> Result<MidiOutputConnection, Box<dyn OtherError>> {
    let midi_out = MidiOutput::new("My Test Output")?;

    // Get an output port (read from console if multiple are available)
    let out_ports = midi_out.ports();
    let out_port: &MidiOutputPort = match out_ports.len() {
        0 => return Err("no output port found".into()),
        1 => {
            println!(
                "Choosing the only available output port: {}",
                midi_out.port_name(&out_ports[0]).unwrap()
            );
            &out_ports[0]
        }
        _ => {
            println!("\nAvailable output ports:");
            for (i, p) in out_ports.iter().enumerate() {
                println!("{}: {}", i, midi_out.port_name(p).unwrap());
            }
            print!("Please select output port: ");
            stdout().flush()?;
            let mut input = String::new();
            stdin().read_line(&mut input)?;
            out_ports
                .get(input.trim().parse::<usize>()?)
                .ok_or("invalid output port selected")?
        }
    };
    println!("\nOpening connection");
    let mut conn_out = midi_out.connect(out_port, "midir-test")?;
    // Return the conn_out
  Ok(conn_out)
}

fn print_state(state: &mut State) {
    state.capture_app_state();

    let time = state.link.clock_micros();
    let enabled = match state.link.is_enabled() {
        true => "yes",
        false => "no ",
    }
    .to_string();
    let num_peers = state.link.num_peers();
    let start_stop = match state.link.is_start_stop_sync_enabled() {
        true => "yes",
        false => "no ",
    };
    let playing = match state.session_state.is_playing() {
        true => "[playing]",
        false => "[stopped]",
    };
    let tempo = state.session_state.tempo();
    let beats = state.session_state.beat_at_time(time, state.quantum);
    let phase = state.session_state.phase_at_time(time, state.quantum);
    let mut metro = String::with_capacity(state.quantum as usize);
    for i in 0..state.quantum as usize {
        if i > phase as usize {
            metro.push('O');
        } else {
            metro.push('X');
        }
    }
}

fn main() -> Result<(), Box<dyn OtherError>> {
    let exit: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));

    println!(r#"███████╗██████╗ ███████╗███╗   ███╗██╗████████╗
██╔════╝██╔══██╗██╔════╝████╗ ████║██║╚══██╔══╝
█████╗  ██████╔╝█████╗  ██╔████╔██║██║   ██║   
██╔══╝  ██╔══██╗██╔══╝  ██║╚██╔╝██║██║   ██║   
███████╗██║  ██║███████╗██║ ╚═╝ ██║██║   ██║   
╚══════╝╚═╝  ╚═╝╚══════╝╚═╝     ╚═╝╚═╝   ╚═╝"#);

    // Starting Ableton Link
    let mut state = State::new();
    let time = state.link.clock_micros();
    state.link.enable(true);

    // Creating Lua and setting up globals
    let lua = Lua::new();
    let globals = lua.globals();
    let mut editor = DefaultEditor::new().expect("Failed to create editor");

    // Setting up a MIDI connexion
    let mut conn_out: Arc<Mutex<Option<MidiOutputConnection>>> = Arc::new(Mutex::new(None));
    match setup_midi() {
        Ok(connection) => {
            conn_out = Arc::new(Mutex::new(Some(connection)));
        },
        Err(err) => println!("Error {}", err),
    }

    // TEST: sending a note (blocks the thread because of sleep)
    let note = lua.create_function(move |_, (note, velocity, channel): (Option<u8>, Option<u8>, Option<u8>)| -> LuaResult<()> {
        let mut note: Option<u8> = Some(note.unwrap_or(60));
        let mut velocity: Option<u8> = Some(velocity.unwrap_or(64));
        let mut channel: Option<u8> = Some(channel.unwrap_or(0));
        const NOTE_ON_MSG: u8 = 0x90;
        const NOTE_OFF_MSG: u8 = 0x80;
        const VELOCITY: u8 = 0x64;
        // We're ignoring errors in here
        if let Some(mut conn) = conn_out.lock().unwrap().as_mut() {
            let _ = conn.send(&[NOTE_ON_MSG, note.unwrap_or(60), VELOCITY]);
            sleep(Duration::from_millis(150));
            let _ = conn.send(&[NOTE_OFF_MSG, note.unwrap_or(60), VELOCITY]);
        } else {
            // Handle the case where conn_out is None
        }
        Ok(())
    })?;
    globals.set("note", note)?;

    let num_peers = lua.create_function(move |_, ()| -> LuaResult<u32> {
        Ok(state.link.num_peers().try_into().unwrap())
    })?;
    globals.set("num_peers", num_peers)?;

    let tempo = lua.create_function(move |_, ()| -> LuaResult<f64> {
        Ok(state.session_state.tempo())
    })?;
    globals.set("tempo", tempo)?;

    let quit = {
        let exit = exit.clone();
        lua.create_function(move |_, ()| -> LuaResult<()> {
            let mut exit = exit.lock().unwrap();
            *exit = true;
            Ok(())
        })?
    };
    globals.set("quit", quit)?;

    loop {
        let mut prompt = "=> ";
        let mut line = String::new();

        while !*exit.lock().unwrap() {
            match editor.readline(prompt) {
                Ok(input) => line.push_str(&input),
                Err(_) => return Err(From::from("Failed to read line")),
            }

            match lua.load(&line).eval::<MultiValue>() {
                Ok(values) => {
                    editor.add_history_entry(line.clone()).unwrap();
                    println!(
                        "{}",
                        values
                            .iter()
                            .map(|value| format!("{:#?}", value))
                            .collect::<Vec<_>>()
                            .join("\t")
                    );
                    continue;
                }
                Err(Error::SyntaxError {
                    incomplete_input: true,
                    ..
                }) => {
                    // continue reading input and append it to `line`
                    line.push_str("\n"); // separate input lines
                    prompt = ">> ";
                }
                Err(e) => {
                    eprintln!("error: {}", e);
                    return Ok(());
                }
            }
        }
        if *exit.lock().unwrap() {
            return Ok(());
        }
    }
}
