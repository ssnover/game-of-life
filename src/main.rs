#![no_std]
#![no_main]

use core::cmp::min;
use core::mem::size_of;
use gba::keys::KeyInput;
use gba::prelude::*;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

const SCREEN_HEIGHT: usize = 160;
const SCREEN_WIDTH: usize = 240;
const PIXELS_PER_CELL: usize = 2;
const CELLS_Y: usize = SCREEN_HEIGHT / PIXELS_PER_CELL;
const CELLS_X: usize = SCREEN_WIDTH / PIXELS_PER_CELL;

type Screen = [Color; CELLS_Y * CELLS_X];

#[no_mangle]
pub fn main() -> ! {
    let mut screen = [Color::from_rgb(0, 0, 0); CELLS_Y * CELLS_X];

    DISPCNT.write(
        DisplayControl::new()
            .with_video_mode(VideoMode::_3)
            .with_show_bg2(true),
    );

    RUST_IRQ_HANDLER.write(Some(irq_blank));
    IME.write(true);
    IE.write(IrqBits::VBLANK);
    DISPSTAT.write(DisplayStatus::new().with_irq_vblank(true));

    let mut current_game_state = GameState::Edit;
    let mut cursor = Cursor::new();
    let mut map_context = RunStateContext::new();

    loop {
        let keys = KEYINPUT.read();

        match current_game_state {
            GameState::Edit => {
                cursor.process_keys(&keys);
                map_context.render_map(&mut screen);
                cursor.render();

                if keys.start() {
                    current_game_state = GameState::Run;
                }
                if keys.a() {
                    map_context.toggle_cell(cursor.x(), cursor.y());
                }
            }
            GameState::Run => {
                map_context.render_map(&mut screen);
                map_context.step();

                if keys.start() {
                    current_game_state = GameState::Edit;
                }
            }
        }

        VBlankIntrWait();
    }
}

#[inline]
fn update_screen(screen: &mut Screen, cell_x: usize, cell_y: usize, color: Color) {
    for pixel_y in (cell_y * 2)..((cell_y * 2) + PIXELS_PER_CELL) {
        for pixel_x in (cell_x * 2)..((cell_x * 2) + PIXELS_PER_CELL) {
            screen[pixel_x + pixel_y * CELLS_X] = color;
        }
    }
}

#[inline]
fn render_cell(cell_x: usize, cell_y: usize, color: Color) {
    for pixel_y in (cell_y * 2)..((cell_y * 2) + PIXELS_PER_CELL) {
        for pixel_x in (cell_x * 2)..((cell_x * 2) + PIXELS_PER_CELL) {
            VIDEO3_VRAM.index(pixel_x, pixel_y).write(color);
        }
    }
}

pub fn render_screen(screen: &mut Screen) {
    for cell_y in 0..CELLS_Y {
        for cell_x in 0..CELLS_X {
            render_cell(cell_x, cell_y, screen[cell_x + cell_y * CELLS_X]);
        }
    }
}

#[derive(Clone, Copy)]
enum GameState {
    Edit,
    Run,
}

struct Cursor {
    x: u16,
    y: u16,
}

impl Cursor {
    pub fn new() -> Self {
        Cursor {
            x: SCREEN_WIDTH as u16 / 2,
            y: SCREEN_HEIGHT as u16 / 2,
        }
    }

    pub fn x(&self) -> usize {
        self.x as usize
    }

    pub fn y(&self) -> usize {
        self.y as usize
    }

    pub fn process_keys(&mut self, input: &KeyInput) {
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
        self.x = min(self.x + 1, CELLS_X as u16 - 1);
    }

    fn move_up(&mut self) {
        self.y = self.y.saturating_sub(1);
    }

    fn move_left(&mut self) {
        self.x = self.x.saturating_sub(1);
    }

    fn move_down(&mut self) {
        self.y = min(self.y + 1, CELLS_Y as u16 - 1);
    }

    fn render(&self) {
        const WHITE: Color = Color::from_rgb(255, 255, 255);
        if self.x != 0 {
            render_cell(self.x as usize - 1, self.y as usize, WHITE);
        }
        if self.y != 0 {
            render_cell(self.x as usize, self.y as usize - 1, WHITE);
        }
        if self.x as usize != CELLS_X - 1 {
            render_cell(self.x as usize + 1, self.y as usize, WHITE);
        }
        if self.y as usize != CELLS_Y - 1 {
            render_cell(self.x as usize, self.y as usize + 1, WHITE);
        }
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

    pub fn render_map(&self, screen: &mut Screen) {
        if self.primary_map_is_a {
            render_game_state(&self.map_state_a, screen);
        } else {
            render_game_state(&self.map_state_b, screen);
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

fn render_game_state(game_state: &MapState, screen: &mut Screen) {
    for col in 0..CELLS_X {
        for row in 0..CELLS_Y {
            if game_state.0[col][row] {
                update_screen(screen, col, row, Color::from_rgb(0, 255, 0));
            } else {
                update_screen(screen, col, row, Color::from_rgb(0, 0, 0));
            }
        }
    }
}

extern "C" fn irq_blank(_bits: IrqBits) {
    unsafe {
        let p = VIDEO3_VRAM.as_usize() as *mut u8;
        gba::mem_fns::__aeabi_memset(p, SCREEN_WIDTH * SCREEN_HEIGHT * size_of::<u16>(), 0)
    }
}
