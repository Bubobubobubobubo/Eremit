use core::fmt::Formatter;
use std::fmt::Display;
use std::sync::{Arc, Mutex};
use midir::MidiOutputConnection;
use std::fmt::Debug;

#[derive(Debug, PartialEq, Clone)]
pub enum BaseEventType {
    Tick,
    NoteOn,
    NoteOff,
    ControlChange,
    ProgramChange,
    PitchBend,
    Aftertouch,
    PolyAftertouch,
    SysEx,
    SysCommon,
    SysRealtime
}

#[derive(Debug, PartialEq, Clone)]
pub struct Event {
    begin: f64,
    end: f64,
    event_type: BaseEventType,
    event_data: Vec<u8>
}

impl Display for Event {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Event: {} {} {}", self.begin, self.end, self.event_type)
    }
}

impl Display for BaseEventType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BaseEventType::Tick => write!(f, "Tick"),
            BaseEventType::NoteOn => write!(f, "NoteOn"),
            BaseEventType::NoteOff => write!(f, "NoteOff"),
            BaseEventType::ControlChange => write!(f, "ControlChange"),
            BaseEventType::ProgramChange => write!(f, "ProgramChange"),
            BaseEventType::PitchBend => write!(f, "PitchBend"),
            BaseEventType::Aftertouch => write!(f, "Aftertouch"),
            BaseEventType::PolyAftertouch => write!(f, "PolyAftertouch"),
            BaseEventType::SysEx => write!(f, "SysEx"),
            BaseEventType::SysCommon => write!(f, "SysCommon"),
            BaseEventType::SysRealtime => write!(f, "SysRealtime")
        }
    }
}


impl Event {

    pub fn new(begin: f64, end: f64, event_type: BaseEventType, event_data: Vec<u8>) -> Self {
        Self {
            begin,
            end,
            event_type,
            event_data
        }
    }

    fn start_event(&self, beat: f64) {
        match self.event_type {
            BaseEventType::Tick => {
                println!("[Tick] Start event: {}", beat);
            },
            _ => {
                println!("Unknown event type: {}", self.event_type);
            }
        }
    }

    fn end_event(&self, beat: f64) {
        match self.event_type {
            BaseEventType::Tick => {
                println!("[Tick] End Event: {}", beat);
            },
            _ => {
                println!("Unknown event type: {}", self.event_type);
            }
        }
    }
}

#[derive(Clone)]
pub struct Stream {
    name: String,
    pattern: Vec<Event>,
    midi: Arc<Mutex<MidiOutputConnection>>
}

impl Stream {
    pub fn new(name: String, midi: Arc<Mutex<MidiOutputConnection>>) -> Self {
        Self {
            name,
            pattern: Vec::new(),
            midi: midi
        }
    }

    pub fn add_event(&mut self, event: Event) {
        self.pattern.push(event);
    }

    pub fn notify_tick(&mut self, beat: f64, _quantum: f64) {
        for event in self.pattern.iter() {
            if event.begin <= beat && event.end >= beat {
                event.start_event(beat);
            } else if event.end < beat {
                event.end_event(beat);
            }
        }

        // Nothing to do...
        if self.pattern.is_empty() {
            return;
        }
    }
}