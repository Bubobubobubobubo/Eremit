use core::fmt::Formatter;
use std::fmt::Display;

enum BaseEventType {
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

struct Event {
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
    fn new(begin: f64, end: f64, event_type: BaseEventType, event_data: Vec<u8>) -> Self {
        Self {
            begin,
            end,
            event_type,
            event_data
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Pattern {}

#[derive(Clone, Debug, PartialEq)]
pub struct Stream {
    name: String,
    pattern: Option<Pattern>
}

impl Stream {
    pub fn new(name: String) -> Self {
        Self {
            name,
            pattern: None
        }
    }

    pub fn notify_tick(&mut self, phase: f64) {
        if self.pattern.is_none() {
            return
        }
    }
}