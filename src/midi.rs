use midir::{MidiOutput, MidiOutputPort, MidiOutputConnection};
use std::io::{stdin, stdout, Write};
use std::error::Error;
use std::result::Result as StdResult;

pub fn _setup_midi() -> StdResult<MidiOutputConnection, Box<dyn Error>> {
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

pub fn setup_midi_connection() -> MidiOutputConnection {
    loop {
        match _setup_midi() {
            Ok(connection) => return connection,
            Err(err) => {
                println!("Error: {}", err);
            }
        }
    }
}


pub enum MidiMessage {
    NoteOn(u8, u8, u8),
    NoteOff(u8, u8),
    ControlChange(u8, u8, u8),
    ProgramChange(u8, u8),
    PitchBend(u8, u8),
    Aftertouch(u8, u8, u8),
    MidiClock,
    MidiStart,
    MidiContinue,
    MidiStop,
    Reset,
}

pub struct MidiConnexion {
    conn_out: MidiOutputConnection,
}

impl MidiConnexion {
    pub fn new() -> Self {
        MidiConnexion {
            conn_out: setup_midi_connection(),
        }
    }

    pub fn send(&mut self, message_type: MidiMessage) -> Result<(), Box<dyn Error>> {
        let _ = match message_type {
            MidiMessage::NoteOn(note, velocity, channel) => self.note_on(note, velocity, channel),
            MidiMessage::NoteOff(note, channel) => self.note_off(note, channel),
            MidiMessage::ControlChange(control, value, channel) => self.control_change(control, value, channel),
            MidiMessage::MidiClock => self.midi_clock(),
            MidiMessage::MidiStart => self.midi_clock_start(),
            MidiMessage::MidiContinue => self.midi_clock_continue(),
            MidiMessage::MidiStop => self.midi_clock_stop(),
            MidiMessage::ProgramChange(program, channel) => self.program_change(program, channel),
            MidiMessage::PitchBend(pitch, channel) => self.pitch_bend(pitch, channel),
            MidiMessage::Aftertouch(note, value, channel) => self.aftertouch(note, value, channel),
            MidiMessage::Reset => self.reset(),
        };
        Ok(())
    }

    fn note_on(&mut self, note: u8, velocity: u8, channel: u8) -> Result<(), Box<dyn Error>> {
        self.conn_out.send(&[0x90 | channel, note, velocity])?;
        Ok(())
    }

    fn note_off(&mut self, note: u8, channel: u8) -> Result<(), Box<dyn Error>> {
        self.conn_out.send(&[0x80 | channel, note, 0])?;
        Ok(())
    }

    fn control_change(&mut self, control: u8, value: u8, channel: u8) -> Result<(), Box<dyn Error>> {
        self.conn_out.send(&[0xB0 | channel, control, value])?;
        Ok(())
    }

    fn program_change(&mut self, program: u8, channel: u8) -> Result<(), Box<dyn Error>> {
        self.conn_out.send(&[0xC0 | channel, program])?;
        Ok(())
    }

    fn pitch_bend(&mut self, value: u8, channel: u8) -> Result<(), Box<dyn Error>> {
        self.conn_out.send(&[0xE0 | channel, value])?;
        Ok(())
    }

    fn aftertouch(&mut self, note: u8, value: u8, channel: u8) -> Result<(), Box<dyn Error>> {
        self.conn_out.send(&[0xA0 | channel, note, value])?;
        Ok(())
    }


    fn midi_clock(&mut self) -> Result<(), Box<dyn Error>> {
        self.conn_out.send(&[0xF8])?;
        Ok(())
    }

    fn midi_clock_start(&mut self) -> Result<(), Box<dyn Error>> {
        self.conn_out.send(&[0xFA])?;
        Ok(())
    }

    fn midi_clock_stop(&mut self) -> Result<(), Box<dyn Error>> {
        self.conn_out.send(&[0xFC])?;
        Ok(())
    }

    fn midi_clock_continue(&mut self) -> Result<(), Box<dyn Error>> {
        self.conn_out.send(&[0xFB])?;
        Ok(())
    }

    fn reset(&mut self) -> Result<(), Box<dyn Error>> {
        self.conn_out.send(&[0xFF])?;
        Ok(())
    }

}