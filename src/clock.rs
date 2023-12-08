use rusty_link::{AblLink, SessionState};
use std::time::{Duration,  SystemTime, UNIX_EPOCH};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Return the current unix time as a std::time::Duration
pub fn current_unix_time() -> Duration {
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

