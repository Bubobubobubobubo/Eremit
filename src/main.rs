use mlua::{Lua, MultiValue};
use rustyline::DefaultEditor;
use std::error::Error as OtherError;
use std::io::{stdin, stdout, Write};
use std::thread::sleep;
use std::time::{Duration,  SystemTime, UNIX_EPOCH};
use midir::{MidiOutput, MidiOutputPort, MidiOutputConnection};
use mlua::{Result as LuaResult};
use std::sync::{Arc, Mutex};
use rusty_link::{AblLink, SessionState};


/// Return the current unix time as a std::time::Duration
fn current_unix_time() -> Duration {
  let current_unix_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
  return current_unix_time;
}

pub struct AbeLinkStateContainer {
  pub abelink: Arc<Mutex<AbeLinkState>>,
}

pub struct AbeLinkState {
  pub link: AblLink,
  pub session_state: SessionState,
  pub running: bool,
  pub quantum: f64,
}

impl AbeLinkState {
  pub fn new() -> Self {
    Self {
      link: AblLink::new(120.0),
      session_state: SessionState::new(),
      running: true,
      quantum: 4.0,
    }
  }

  pub fn unix_time_at_next_phase(&self) -> u64 {
    let link_time_stamp = self.link.clock_micros();
    let quantum = self.quantum;
    let beat = self.session_state.beat_at_time(link_time_stamp, quantum);
    let phase = self.session_state.phase_at_time(link_time_stamp, quantum);
    let internal_time_at_next_phase = self.session_state.time_at_beat(beat + (quantum - phase), quantum);
    let time_offset = Duration::from_micros((internal_time_at_next_phase - link_time_stamp) as u64);
    let current_unix_time = current_unix_time();
    let unix_time_at_next_phase = (current_unix_time + time_offset).as_millis();
    return unix_time_at_next_phase as u64;
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
    let conn_out = midi_out.connect(out_port, "midir-test")?;
    // Return the conn_out
  Ok(conn_out)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn OtherError>> {
    // let mut exit: bool = false;
    let mut exit = Arc::new(Mutex::new(false));


    println!(r#"███████╗██████╗ ███████╗███╗   ███╗██╗████████╗
██╔════╝██╔══██╗██╔════╝████╗ ████║██║╚══██╔══╝
█████╗  ██████╔╝█████╗  ██╔████╔██║██║   ██║   
██╔══╝  ██╔══██╗██╔══╝  ██║╚██╔╝██║██║   ██║   
███████╗██║  ██║███████╗██║ ╚═╝ ██║██║   ██║   
╚══════╝╚═╝  ╚═╝╚══════╝╚═╝     ╚═╝╚═╝   ╚═╝"#);

    // Starting Ableton Link
    let clock = Arc::new(Mutex::new(AbeLinkState::new()));


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
        Err(err) => {
            println!("Error {}", err);
            let mut success = false;
            while !success {
                match setup_midi() {
                    Ok(connection) => {
                        conn_out = Arc::new(Mutex::new(Some(connection)));
                        success = true;
                    },
                    Err(err) => {
                        println!("Error {}", err);
                    }
                }
            }
        }
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

    // Pass a clone of the Arc<Mutex<>> to the Lua function
    let exit_clone = exit.clone();

    // Create the quit Lua function
    let quit = lua.create_function(move |_, ()| -> LuaResult<()> {
        // Lock the mutex to modify the boolean
        let mut exit = exit_clone.lock().unwrap();
        *exit = true;
        Ok(())
    })?;
    globals.set("quit", quit)?;

    loop {
        let prompt = "=> ";
        let mut line = String::new();

        while !*exit.lock().unwrap() {
            line = String::new();
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
                Err(e) => {
                    eprintln!("error: {}", e);
                    continue;
                }
            }
        }
        if *exit.lock().unwrap() {
            return Ok(());
        }
    }
}
