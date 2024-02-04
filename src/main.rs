#![no_std]
#![no_main]

use conway::ConwaysState;
use gba::prelude::*;

mod conway;
mod cursor;
use cursor::Cursor;
mod keys;
use keys::StatefulKeys;
mod game_state;
use game_state::GameState;

const SCREEN_WIDTH: usize = 240;
const SCREEN_HEIGHT: usize = 160;
const CELL_SIZE: usize = 2;
const BOARD_WIDTH: usize = SCREEN_WIDTH / CELL_SIZE;
const BOARD_WIDTH_BYTES: usize = BOARD_WIDTH / 8;
const BOARD_HEIGHT: usize = SCREEN_HEIGHT / CELL_SIZE;

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
fn main() -> ! {
    DISPCNT.write(
        DisplayControl::new()
            .with_video_mode(VideoMode::_3)
            .with_show_bg2(true),
    );

    DISPSTAT.write(DisplayStatus::new().with_irq_vblank(true));
    IE.write(IrqBits::VBLANK);
    IME.write(true);

    // CPU frequency is 16.78 MHz, so a tick with a prescale of 64 is a little less than 2.5 ms
    TIMER0_CONTROL.write(
        TimerControl::new()
            .with_scale(TimerScale::_1)
            .with_enabled(true),
    );

    let mut keys = StatefulKeys::new();
    let mut cursor = Cursor::new();
    let mut game_state = GameState::new();
    let mut conways_state_a = ConwaysState::<BOARD_WIDTH_BYTES, BOARD_HEIGHT>::new();
    let mut conways_state_b = ConwaysState::<BOARD_WIDTH_BYTES, BOARD_HEIGHT>::new();
    let mut current_is_a = true;

    conways_state_b.set_cell(0, 1, true);
    conways_state_b.set_cell(1, 1, true);
    conways_state_b.set_cell(2, 1, true);

    let mut dirty_row_buffer = [0u8; BOARD_HEIGHT / 8];

    let mut counter = 0;
    const STEP_PERIOD: u32 = 10;

    loop {
        let (current_state, last_state) = if current_is_a {
            (&mut conways_state_a, &mut conways_state_b)
        } else {
            (&mut conways_state_b, &mut conways_state_a)
        };

        game_state.update(&mut keys);
        if let GameState::Run = game_state {
            if counter == 0 {
                current_state.step(&last_state, &mut dirty_row_buffer);
            }
        } else {
            cursor.update(&mut keys, current_state, &mut dirty_row_buffer);
        }

        draw_sprites(game_state, &cursor, current_state, &dirty_row_buffer);
        dirty_row_buffer.iter_mut().for_each(|elem| *elem = 0x00);

        VBlankIntrWait();

        if let GameState::Run = game_state {
            counter += 1;
            if counter == STEP_PERIOD {
                counter = 0;
                current_is_a = !current_is_a;
            }
        }
    }
}

fn draw_sprites(
    game_state: GameState,
    cursor: &Cursor,
    board: &ConwaysState<BOARD_WIDTH_BYTES, BOARD_HEIGHT>,
    dirty_row_buffer: &[u8],
) {
    let cursor_color = if let GameState::Edit = game_state {
        Color::WHITE
    } else {
        Color::BLACK
    };
    if cursor.x > 0 {
        // Draw the left side
        draw_cell(cursor.x - 1, cursor.y, cursor_color)
    }
    if cursor.x < BOARD_WIDTH as u16 - 1 {
        // Draw the right side
        draw_cell(cursor.x + 1, cursor.y, cursor_color)
    }
    if cursor.y > 0 {
        // Draw the top side
        draw_cell(cursor.x, cursor.y - 1, cursor_color)
    }
    if cursor.y < BOARD_HEIGHT as u16 - 1 {
        // Draw the bottom side
        draw_cell(cursor.x, cursor.y + 1, cursor_color)
    }

    for row in 0..BOARD_HEIGHT {
        if dirty_row_buffer[row / 8] & (1 << (row % 8)) != 0 {
            for col in 0..BOARD_WIDTH {
                draw_cell(
                    col as u16,
                    row as u16,
                    if board.get_cell(col, row) {
                        Color::GREEN
                    } else {
                        Color::BLACK
                    },
                );
            }
        }
    }
}

#[inline]
fn draw_cell(x: u16, y: u16, color: Color) {
    for i in 0..CELL_SIZE as u16 {
        for j in 0..CELL_SIZE as u16 {
            VIDEO3_VRAM
                .index(
                    ((x * CELL_SIZE as u16) + i) as usize,
                    ((y * CELL_SIZE as u16) + j) as usize,
                )
                .write(color);
        }
    }
}
