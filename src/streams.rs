#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
struct Pattern {}

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
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