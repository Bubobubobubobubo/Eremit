use mlua::{Error, Lua, MultiValue};
use rustyline::DefaultEditor;
use std::error::Error as OtherError;
use std::io::{stdin, stdout, Write};
use std::thread::sleep;
use std::time::Duration;
use midir::{MidiOutput, MidiOutputPort, MidiOutputConnection};
use mlua::{Result as LuaResult};
use std::sync::{Arc, Mutex};

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

fn main() -> Result<(), Box<dyn OtherError>> {
    println!(r#"███████╗██████╗ ███████╗███╗   ███╗██╗████████╗
██╔════╝██╔══██╗██╔════╝████╗ ████║██║╚══██╔══╝
█████╗  ██████╔╝█████╗  ██╔████╔██║██║   ██║   
██╔══╝  ██╔══██╗██╔══╝  ██║╚██╔╝██║██║   ██║   
███████╗██║  ██║███████╗██║ ╚═╝ ██║██║   ██║   
╚══════╝╚═╝  ╚═╝╚══════╝╚═╝     ╚═╝╚═╝   ╚═╝"#);

    let lua = Lua::new();
    let globals = lua.globals();
    let mut editor = DefaultEditor::new().expect("Failed to create editor");
    let mut conn_out: Arc<Mutex<Option<MidiOutputConnection>>> = Arc::new(Mutex::new(None));

    match setup_midi() {
        Ok(connection) => {
            conn_out = Arc::new(Mutex::new(Some(connection)));
        },
        Err(err) => println!("Error {}", err),
    }

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


    loop {
        let mut prompt = "> ";
        let mut line = String::new();

        loop {
            match editor.readline(prompt) {
                Ok(input) => line.push_str(&input),
                Err(_) => return Err(From::from("Failed to read line")),
            }

            match lua.load(&line).eval::<MultiValue>() {
                Ok(values) => {
                    editor.add_history_entry(line).unwrap();
                    println!(
                        "{}",
                        values
                            .iter()
                            .map(|value| format!("{:#?}", value))
                            .collect::<Vec<_>>()
                            .join("\t")
                    );
                    break;
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
                    break;
                }
            }
        }
    }
}
