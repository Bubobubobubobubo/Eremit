use rusty_link::{AblLink, SessionState};
use std::time::{Duration,  SystemTime, UNIX_EPOCH};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::interval;

#[derive(Debug)]
#[derive(Clone)]
pub struct ClockState {
  pub enabled: String,
  pub num_peers: u64,
  pub start_stop: String,
  pub playing: String,
  pub tempo: f64,
  pub beats: f64,
  pub phase: f64,
  pub metro: String,
}

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
  pub snapshot: Option<ClockState>,
  pub sync: bool,
}

impl AbeLinkState {
  pub fn new() -> Self {
    Self {
      link: AblLink::new(120.0),
      session_state: SessionState::new(),
      sync: true,
      running: true,
      quantum: 4.0,
      snapshot: None
    }
  }

  pub fn get_clock_state(&mut self) -> ClockState {
    self.capture_app_state();
    let time = self.link.clock_micros();
    let enabled = match self.link.is_enabled() {
      true => "yes",
      false => "no ",
    }
    .to_string();
    let num_peers = self.link.num_peers();
    let start_stop = match self.link.is_start_stop_sync_enabled() {
      true => "yes",
      false => "no ",
    }
    .to_string();
    let playing = match self.session_state.is_playing() {
      true => "[playing]",
      false => "[stopped]",
    }
    .to_string();
    let tempo = self.session_state.tempo();
    let beats = self.session_state.beat_at_time(time, self.quantum);
    let phase = self.session_state.phase_at_time(time, self.quantum);
    let mut metro = String::with_capacity(self.quantum as usize);
    for i in 0..self.quantum as usize {
      if i > phase as usize {
        metro.push('O');
      } else {
        metro.push('X');
      }
    }

    ClockState {
      enabled,
      num_peers,
      start_stop,
      playing,
      tempo,
      beats,
      phase,
      metro,
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
  
  pub fn is_running(&self) -> bool {
    return self.running;
  }

  pub fn set_tempo(&mut self, tempo: f64) {
    let time_stamp = self.link.clock_micros();
    self.session_state.set_tempo(tempo, time_stamp);
    self.commit_app_state();
  }
  
  pub fn make_snapshot(&mut self) {
    self.snapshot = Some(self.get_clock_state());
  }

  pub fn sync(&mut self) {
    self.sync = !self.sync;
    println!("Sync: {}", &self.sync);
    self.link.enable_start_stop_sync(self.sync)
  }

  pub fn peers(&self) -> u64 {
    return self.link.num_peers();
  }

  pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
      // Create a recurring timer that triggers every 10ms
      let mut interval = interval(Duration::from_millis(20));
      self.link.enable(true);
      self.link.enable_start_stop_sync(true);

      // Loop that captures the session state at regular intervals
      loop {
          // Wait for the timer to trigger
          interval.tick().await;
          self.make_snapshot();
          if !self.is_running() {
              return Ok(());
          }
      }
  }


}