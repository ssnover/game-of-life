use gba::prelude::*;

pub struct StatefulKeys {
    keys: DebouncedKeys,
    last_start_state: bool,
}

impl StatefulKeys {
    pub fn new() -> Self {
        Self {
            keys: DebouncedKeys::new(),
            last_start_state: false,
        }
    }

    pub fn start(&mut self) -> KeyState {
        self.keys.update();
        let state = KeyState {
            last_state: self.last_start_state,
            current_state: self.keys.start(),
        };
        self.last_start_state = self.keys.start();
        state
    }
}

pub enum Edge {
    Rising,
    Falling,
}

pub struct KeyState {
    last_state: bool,
    current_state: bool,
}

impl KeyState {
    pub fn change(&self) -> Option<Edge> {
        match (self.last_state, self.current_state) {
            (true, false) => Some(Edge::Falling),
            (false, true) => Some(Edge::Rising),
            _ => None,
        }
    }
}

struct DebouncedKeyState {
    last_state: bool,
    last_debounce_time: u16,
    debounced_state: bool,
}

struct DebouncedKeys {
    start: DebouncedKeyState,
}

impl DebouncedKeys {
    const DEBOUNCE_DELAY: u16 = 10;

    pub fn new() -> Self {
        let now = TIMER0_COUNT.read();
        let keys = KEYINPUT.read();
        Self {
            start: DebouncedKeyState {
                last_state: keys.start(),
                last_debounce_time: now,
                debounced_state: keys.start(),
            },
        }
    }

    pub fn update(&mut self) {
        let now = TIMER0_COUNT.read();
        let keys = KEYINPUT.read();

        if self.start.last_state != keys.start() {
            self.start.last_debounce_time = now;
        }

        if now.wrapping_sub(self.start.last_debounce_time) > Self::DEBOUNCE_DELAY {
            if keys.start() != self.start.debounced_state {
                self.start.debounced_state = keys.start();
            }
        }

        self.start.last_state = keys.start();
    }

    pub fn start(&self) -> bool {
        self.start.debounced_state
    }
}
