use rusty_link::{AblLink, SessionState};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::thread::sleep;
use std::sync::mpsc::{Receiver, Sender};
use crate::streams::Stream;

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

pub struct Clock {
  pub link: AblLink,
  pub session_state: SessionState,
  pub running: bool,
  pub quantum: f64,
  pub snapshot: Option<ClockState>,
  pub sync: bool,
  receiver: Receiver<ClockControlMessage>,
  sender: Sender<ClockControlMessage>,
  subscribers: Vec<Stream>
}
#[derive(Debug)]
pub struct ClockControlMessage {
    pub name: String,
    pub args: Vec<String>
}

impl Clock {
  pub fn new(receiver: Receiver<ClockControlMessage>, sender: Sender<ClockControlMessage>) -> Self {
    Self {
      link: AblLink::new(120.0),
      session_state: SessionState::new(),
      sync: true,
      running: true,
      quantum: 4.0,
      snapshot: None,
      receiver: receiver,
      sender: sender,
      subscribers: Vec::new()
    }
  }

  pub fn add_subscriber(&mut self, stream: Stream) {
    self.subscribers.push(stream);
  }

  pub fn remove_subscriber(&mut self, stream: Stream) {
    let index = self.subscribers.iter().position(|x| *x == stream).unwrap();
    self.subscribers.remove(index);
  }

  pub fn clear_subs(&mut self) {
    self.subscribers.clear();
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
    self.link.enable_start_stop_sync(self.sync);
    self.commit_app_state();
    self.report();
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
    self.report();
  }

  pub fn report(&mut self) {
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
      };
      let playing = match self.session_state.is_playing() {
          true => "[playing]",
          false => "[stopped]",
      };
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
      println!("{:<7} | {:<9} | {:<7} | {:<3}   {:<9} | {:<7.2} | {:<8.2} | {}",
           enabled, num_peers, self.quantum.trunc(), start_stop, playing, tempo, beats, metro);
  }

  pub fn handle_messages(&mut self, recv: &ClockControlMessage) {
      self.capture_app_state();
      match recv.name.as_str() {
          "beats" => {
            self.sender.send(ClockControlMessage {
              name: "beats".to_string(),
              args: vec![self.session_state.beat_at_time(self.link.clock_micros(), self.quantum).to_string()],
            }).unwrap();
          }
          "subscribers" => {
            self.sender.send(ClockControlMessage {
              name: "subscribers".to_string(),
              args: vec![self.subscribers.len().to_string()],
            }).unwrap();
          },
          "add_subscriber" => {
            let stream = Stream::new(recv.args[0].clone());
            self.add_subscriber(stream);
          },
          "sync" => {
            self.sync();
          },
          "play" => {
            self.play();
            self.commit_app_state();
          },
          "peers" => {
            self.sender.send(ClockControlMessage {
              name: "peers".to_string(),
              args: vec![self.link.num_peers().to_string()],
            }).unwrap();
          },
          "get_tempo" => {
            self.sender.send(ClockControlMessage {
              name: "get_tempo".to_string(),
              args: vec![self.session_state.tempo().to_string()],
            }).unwrap();
          },
          "set_tempo" => {
            let tempo = recv.args[0].parse::<f64>().unwrap();
            self.set_tempo(tempo);
            self.commit_app_state();
          },
          "get_phase" => {
            self.sender.send(ClockControlMessage {
              name: "get_phase".to_string(),
              args: vec![self.session_state.phase_at_time(self.link.clock_micros(), self.quantum).to_string()],
            }).unwrap();
          },
          "report" => {
            self.report();
          },
          _ => {
            println!("Unknown command: {}", recv.name);
          }
      }
  }

  pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
      self.link.enable_start_stop_sync(true);
      self.link.enable(true);
      let interval = Duration::from_millis(20);
      let mut next_time = Instant::now() + interval;
      loop {
          let receive = self.receiver.try_recv();
          match receive {
              Ok(recv) => {
                  self.handle_messages(&recv);
              },
              Err(_) => {}
          }

          for sub in &mut self.subscribers {
            sub.notify_tick(self.session_state.phase_at_time(
              self.link.clock_micros(), 
              self.quantum)
            );
          }
          if !self.is_running() {
              return Ok(());
          }
          self.commit_app_state();
          self.link.commit_app_session_state(&self.session_state);
          sleep(next_time - Instant::now());
          next_time += interval;
      }
  }

  pub fn capture_app_state(&mut self) {
    self.link.capture_app_session_state(&mut self.session_state);
  }

  pub fn commit_app_state(&mut self) {
    self.link.commit_app_session_state(&self.session_state);
  }
}