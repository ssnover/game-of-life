#![no_std]
#![no_main]
#![feature(isa_attribute)]

use core::cmp::min;
use gba::prelude::*;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

const PIXELS_PER_CELL: usize = 2;
const CELLS_Y: usize = mode3::HEIGHT / PIXELS_PER_CELL;
const CELLS_X: usize = mode3::WIDTH / PIXELS_PER_CELL;

#[no_mangle]
pub fn main() -> ! {
    DISPCNT.write(
        DisplayControl::new()
            .with_display_mode(3)
            .with_display_bg2(true),
    );

    unsafe {
        USER_IRQ_HANDLER.write(Some(irq_handler_a32));
        IME.write(true);
        IE.write(InterruptFlags::new().with_vblank(true));
    };
    DISPSTAT.write(DisplayStatus::new().with_vblank_irq_enabled(true));

    let mut screen_buf = Screen::new();
    let mut current_game_state = GameState::Edit;
    let mut cursor = Cursor::new();
    let mut map_context = RunStateContext::new();

    loop {
        let keys = KEYINPUT.read().into();

        match current_game_state {
            GameState::Edit => {
                // process cursor movement
                cursor.process_keys(&keys);
                // render map
                map_context.render_map(&mut screen_buf);
                // render cursor
                render_cursor(&cursor, &mut screen_buf);
                if keys.start() {
                    current_game_state = GameState::Run;
                }
                if keys.a() {
                    map_context.toggle_cell(cursor.x(), cursor.y());
                }
            }
            GameState::Run => {
                // render map
                map_context.render_map(&mut screen_buf);
                // step forward automata
                map_context.step();
                if keys.start() {
                    current_game_state = GameState::Edit;
                }
            }
        }

        screen_buf.render();

        unsafe { VBlankIntrWait() };
    }
}

fn write_pixel(x: usize, y: usize, color: Color) {
    mode3::bitmap_xy(x, y).write(color);
}

pub struct Screen([[Color; CELLS_Y]; CELLS_X]);

impl Screen {
    pub fn new() -> Self {
        Screen([[Color::from_rgb(0, 0, 0); CELLS_Y]; CELLS_X])
    }
    pub fn write_cell(&mut self, x: usize, y: usize, color: Color) {
        self.0[x][y] = color;
    }

    pub fn render(&self) {
        for col in 0..CELLS_X {
            for row in 0..CELLS_Y {
                Screen::render_cell(col, row, self.0[col][row]);
            }
        }
    }

    fn render_cell(x: usize, y: usize, color: Color) {
        for row in 0..PIXELS_PER_CELL {
            for col in 0..PIXELS_PER_CELL {
                write_pixel(
                    (PIXELS_PER_CELL * x) + col,
                    (PIXELS_PER_CELL * y) + row,
                    color,
                );
            }
        }
    }
}

enum GameState {
    Edit,
    Run,
}

struct Cursor((usize, usize));

impl Cursor {
    pub fn new() -> Self {
        Cursor((CELLS_X / 2, CELLS_Y / 2))
    }

    pub fn x(&self) -> usize {
        self.0 .0
    }

    pub fn y(&self) -> usize {
        self.0 .1
    }

    pub fn process_keys(&mut self, input: &Keys) {
        if input.up() {
            self.move_up();
        } else if input.down() {
            self.move_down();
        }

        if input.left() {
            self.move_left();
        } else if input.right() {
            self.move_right();
        }
    }

    fn move_right(&mut self) {
        self.0 .0 = min(self.0 .0 + 1, CELLS_X - 1);
    }

    fn move_up(&mut self) {
        self.0 .1 = self.0 .1.saturating_sub(1);
    }

    fn move_left(&mut self) {
        self.0 .0 = self.0 .0.saturating_sub(1);
    }

    fn move_down(&mut self) {
        self.0 .1 = min(self.0 .1 + 1, CELLS_Y - 1)
    }
}

fn render_cursor(cursor: &Cursor, screen_buf: &mut Screen) {
    const WHITE: Color = Color::from_rgb(255, 255, 255);
    if cursor.0 .0 != 0 {
        screen_buf.write_cell(cursor.0 .0 - 1, cursor.0 .1, WHITE);
    }
    if cursor.0 .1 != 0 {
        screen_buf.write_cell(cursor.0 .0, cursor.0 .1 - 1, WHITE);
    }
    if cursor.0 .0 != CELLS_X - 1 {
        screen_buf.write_cell(cursor.0 .0 + 1, cursor.0 .1, WHITE);
    }
    if cursor.0 .1 != CELLS_Y - 1 {
        screen_buf.write_cell(cursor.0 .0, cursor.0 .1 + 1, WHITE);
    }
}

pub struct RunStateContext {
    map_state_a: MapState,
    map_state_b: MapState,
    primary_map_is_a: bool,
}

impl RunStateContext {
    pub fn new() -> Self {
        Self {
            map_state_a: MapState::new(),
            map_state_b: MapState::new(),
            primary_map_is_a: true,
        }
    }

    pub fn step(&mut self) {
        if self.primary_map_is_a {
            self.map_state_a.step(&mut self.map_state_b);
        } else {
            self.map_state_b.step(&mut self.map_state_a);
        }
        self.primary_map_is_a = !self.primary_map_is_a;
    }

    pub fn render_map(&self, screen_buf: &mut Screen) {
        if self.primary_map_is_a {
            render(&self.map_state_a, screen_buf);
        } else {
            render(&self.map_state_b, screen_buf);
        }
    }

    pub fn set_cell(&mut self, x: usize, y: usize, state: bool) {
        self.map_state_a.0[x][y] = state;
        self.map_state_b.0[x][y] = state;
    }

    pub fn toggle_cell(&mut self, x: usize, y: usize) {
        let new_state = !if self.primary_map_is_a {
            self.map_state_a.0[x][y]
        } else {
            self.map_state_b.0[x][y]
        };
        self.set_cell(x, y, new_state);
    }
}

struct MapState([[bool; CELLS_Y]; CELLS_X]);

impl MapState {
    pub fn new() -> MapState {
        MapState([[false; CELLS_Y]; CELLS_X])
    }

    fn neighbors_alive(&self, x: usize, y: usize) -> u8 {
        let mut count = 0u8;
        for col in x.saturating_sub(1)..=min(x + 1, CELLS_X - 1) {
            for row in y.saturating_sub(1)..=min(y + 1, CELLS_Y - 1) {
                if col == x && row == y {
                    continue;
                } else if self.0[col][row] {
                    count += 1;
                }
            }
        }

        count
    }

    pub fn step(&self, next: &mut MapState) {
        for col in 0..CELLS_X {
            for row in 0..CELLS_Y {
                match (self.0[col][row], self.neighbors_alive(col, row)) {
                    (true, 2..=3) => {
                        next.0[col][row] = true;
                    }
                    (false, 3) => {
                        next.0[col][row] = true;
                    }
                    (_, _) => {
                        next.0[col][row] = false;
                    }
                }
            }
        }
    }
}

fn render(game_state: &MapState, screen_buf: &mut Screen) {
    for col in 0..CELLS_X {
        for row in 0..CELLS_Y {
            if game_state.0[col][row] {
                screen_buf.write_cell(col, row, Color::from_rgb(0, 31, 0));
            } else {
                screen_buf.write_cell(col, row, Color::from_rgb(0, 0, 0));
            }
        }
    }
}

#[instruction_set(arm::a32)]
extern "C" fn irq_handler_a32() {
    irq_handler_t32();
}

fn irq_handler_t32() {
    unsafe { IME.write(false) };

    let mut intr_wait_flags = INTR_WAIT_ACKNOWLEDGE.read();
    intr_wait_flags.set_vblank(true);
    IRQ_ACKNOWLEDGE.write(intr_wait_flags);
    unsafe { INTR_WAIT_ACKNOWLEDGE.write(intr_wait_flags) };

    unsafe { IME.write(true) };
}
