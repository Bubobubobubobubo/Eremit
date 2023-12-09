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

  
  pub fn is_running(&self) -> bool {
    return self.running;
  }

  pub fn set_tempo(&mut self, tempo: f64) {
    let time_stamp = self.link.clock_micros();
    self.session_state.set_tempo(tempo, time_stamp);
    self.link.commit_app_session_state(&self.session_state);
    self.commit_app_state();
  }
  

  // Make snapshots

  pub fn make_snapshot(&mut self) {
    self.snapshot = Some(self.get_clock_state());
  }

  pub fn print_snapshot(&self) {
    match &self.snapshot {
      Some(s) => {
        println!("Tempo: {}", &s.tempo);
        println!("Beats: {}", &s.beats);
        println!("Phase: {}", &s.phase);
        println!("Metro: {}", &s.metro);
      }
      None => println!("No snapshot available"),
    }
  }

  pub fn sync(&mut self) {
    self.sync = !self.sync;
    println!("Sync: {}", &self.sync);
    self.link.enable_start_stop_sync(self.sync);
    self.commit_app_state();
  }

  pub fn peers(&self) -> u64 {
    return self.link.num_peers();
  }

  pub fn play(&mut self) {
    let time_stamp = self.link.clock_micros();
    if self.session_state.is_playing() {
      self.session_state.set_is_playing(false, time_stamp as u64);
    } else {
      self.session_state.set_is_playing_and_request_beat_at_time(
        true,
        time_stamp as u64,
        0.,
        self.quantum);
    }
    self.commit_app_state();
  }

  pub fn report(&mut self) {
      self.capture_app_state();
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
  
      println!("{:<7} | {:<9} | {:<7} | {:<3}   {:<9} | {:<7.2} | {:<8.2} | {}",
           enabled, num_peers, state.quantum.trunc(), start_stop, playing, tempo, beats, metro);
  }

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
          // For debug purposes
          // self.print_snapshot();
          if !self.is_running() {
              return Ok(());
          }
          self.commit_app_state();
          self.link.commit_app_session_state(&self.session_state);
      }
  }

  pub fn capture_app_state(&mut self) {
    self.link.capture_app_session_state(&mut self.session_state);
  }

  pub fn commit_app_state(&mut self) {
    self.link.commit_app_session_state(&self.session_state);
  }
}