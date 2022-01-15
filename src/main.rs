#![no_std]
#![no_main]

use gba::prelude::*;
use core::cmp::min;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        DISPCNT.read();
    }
}

const PIXELS_PER_CELL: usize = 2;
const CELLS_Y: usize = mode3::HEIGHT / PIXELS_PER_CELL;
const CELLS_X: usize = mode3::WIDTH / PIXELS_PER_CELL;

const RED: Color = Color::from_rgb(255, 0, 0);
const GREEN: Color = Color::from_rgb(0, 255, 0);
const BLUE: Color = Color::from_rgb(0, 0 , 255);

#[no_mangle]
pub fn main() -> ! {
    const SETTING: DisplayControl = DisplayControl::new()
        .with_display_mode(3)
        .with_display_bg2(true);
    DISPCNT.write(SETTING);
    let mut keys = RisingEdgeKeyDetection::new();

    let mut game_state_a = GameState::new();
    let mut game_state_b = GameState::new();

    // Initialize A with some state
    game_state_a.0[50][50] = true;
    game_state_a.0[51][50] = true;
    game_state_a.0[52][50] = true;
    //game_state_a.0[CELLS_X - 5][CELLS_Y - 4] = true;
    //game_state_a.0[CELLS_X - 4][CELLS_Y - 3] = true;
    game_state_a.0[51][49] = true;

    let mut display_a = true;

    loop {
        keys.read();
        if keys.inner().a() {
            write_cell(40, 20, Color::from_rgb(255, 0, 0));
        }

        if display_a {
            render(&game_state_a);
            game_state_a.step(&mut game_state_b);
            //write_cell(0, 0, RED);
        } else {
            render(&game_state_b);
            game_state_b.step(&mut game_state_a);
            //write_cell(0, 0, BLUE);
        }
        display_a = !display_a;

        spin_cycle();
    }
}

fn write_pixel(x: usize, y: usize, color: Color) {
    mode3::bitmap_xy(x, y).write(color);
}

fn write_cell(x: usize, y: usize, color: Color) {
    for row in 0..PIXELS_PER_CELL {
        for col in 0..PIXELS_PER_CELL {
            write_pixel((PIXELS_PER_CELL * x) + col, (PIXELS_PER_CELL * y) + row, color);
        }
    }
}

fn spin_cycle() {
    for _ in 0..8 {
        while VCOUNT.read() < 160 {}
        while VCOUNT.read() >= 160 {}
    }
}

struct RisingEdgeKeyDetection {
    prev_key_state: Keys,
    current_key_state: Keys,
}

impl RisingEdgeKeyDetection {
    pub fn new() -> Self {
        Self {
            prev_key_state: KEYINPUT.read().into(),
            current_key_state: KEYINPUT.read().into(),
        }
    }

    pub fn read(&mut self) {
        self.prev_key_state = self.current_key_state;
        self.current_key_state = KEYINPUT.read().into();
    }

    pub fn inner(&self) -> &Keys {
        &self.current_key_state
    }

    pub fn a_rising(&self) -> bool {
        !self.prev_key_state.a() && self.current_key_state.a()
    }
    
    pub fn start_rising(&self) -> bool {
        !self.prev_key_state.start() && self.current_key_state.start()
    }
}

struct GameState([[bool; CELLS_Y]; CELLS_X]);

impl GameState {
    pub fn new() -> GameState {
        GameState([[false; CELLS_Y]; CELLS_X])
    }

    fn neighbors_alive(&self, x: usize, y: usize) -> u8 {
        let mut count = 0u8;
        for col in x.saturating_sub(1)..=min(x+1, CELLS_X-1) {
            for row in y.saturating_sub(1)..=min(y+1, CELLS_Y-1) {
                if col == x && row == y {
                    continue;
                } else if self.0[col][row] {
                    count += 1;
                }
            }
        }

        count
    }

    pub fn step(&self, next: &mut GameState) {
        for col in 0..CELLS_X {
            for row in 0..CELLS_Y {
                match (self.0[col][row], self.neighbors_alive(col, row)) {
                    (true, 2..=3) => { 
                        next.0[col][row] = true;
                    },
                    (false, 3) => { 
                        next.0[col][row] = true;
                    }
                    (_, _) => {next.0[col][row] = false; }
                }
            }
        }
    }
}

fn render(game_state: &GameState) {
    for col in 0..CELLS_X {
        for row in 0..CELLS_Y {
            if game_state.0[col][row] {
                write_cell(col, row, Color::from_rgb(0, 31, 0));
            } else {
                write_cell(col, row, Color::from_rgb(0, 0, 0));
            }
        }
    }
}