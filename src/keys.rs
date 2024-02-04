use gba::prelude::*;

pub struct StatefulKeys {
    keys: DebouncedKeys,
    last_start_state: bool,
    last_select_state: bool,
    last_a_state: bool,
}

impl StatefulKeys {
    pub fn new() -> Self {
        Self {
            keys: DebouncedKeys::new(),
            last_start_state: false,
            last_select_state: false,
            last_a_state: false,
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

    pub fn select(&mut self) -> KeyState {
        self.keys.update();
        let state = KeyState {
            last_state: self.last_select_state,
            current_state: self.keys.select(),
        };
        self.last_select_state = self.keys.select();
        state
    }

    pub fn a(&mut self) -> KeyState {
        self.keys.update();
        let state = KeyState {
            last_state: self.last_a_state,
            current_state: self.keys.a(),
        };
        self.last_a_state = self.keys.a();
        state
    }

    pub fn left(&self) -> bool {
        self.keys.left()
    }

    pub fn right(&self) -> bool {
        self.keys.right()
    }

    pub fn up(&self) -> bool {
        self.keys.up()
    }

    pub fn down(&self) -> bool {
        self.keys.down()
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

impl DebouncedKeyState {
    pub fn new(initial_state: bool, time: u16) -> Self {
        Self {
            last_state: initial_state,
            last_debounce_time: time,
            debounced_state: initial_state,
        }
    }

    pub fn update(&mut self, new_state: bool, time: u16) {
        if self.last_state != new_state {
            self.last_debounce_time = time;
        }

        if time.wrapping_sub(self.last_debounce_time) > DebouncedKeys::DEBOUNCE_DELAY {
            if new_state != self.debounced_state {
                self.debounced_state = new_state;
            }
        }

        self.last_state = new_state;
    }
}

struct DebouncedKeys {
    start: DebouncedKeyState,
    select: DebouncedKeyState,
    a: DebouncedKeyState,
}

impl DebouncedKeys {
    const DEBOUNCE_DELAY: u16 = 40;

    pub fn new() -> Self {
        let now = TIMER0_COUNT.read();
        let keys = KEYINPUT.read();
        Self {
            start: DebouncedKeyState::new(keys.start(), now),
            select: DebouncedKeyState::new(keys.select(), now),
            a: DebouncedKeyState::new(keys.a(), now),
        }
    }

    pub fn update(&mut self) {
        let now = TIMER0_COUNT.read();
        let keys = KEYINPUT.read();

        self.start.update(keys.start(), now);
        self.a.update(keys.a(), now);
        self.select.update(keys.select(), now);
    }

    pub fn start(&self) -> bool {
        self.start.debounced_state
    }

    pub fn select(&self) -> bool {
        self.select.debounced_state
    }

    pub fn a(&self) -> bool {
        self.a.debounced_state
    }

    pub fn left(&self) -> bool {
        KEYINPUT.read().left()
    }

    pub fn right(&self) -> bool {
        KEYINPUT.read().right()
    }

    pub fn up(&self) -> bool {
        KEYINPUT.read().up()
    }

    pub fn down(&self) -> bool {
        KEYINPUT.read().down()
    }
}
