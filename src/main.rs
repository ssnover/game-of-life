#![no_std]
#![no_main]

use gba::{mem_fns::__aeabi_memset, prelude::*};

mod keys;
use keys::StatefulKeys;
mod game_state;
use game_state::GameState;

const SCREEN_WIDTH: u16 = 240;
const SCREEN_HEIGHT: u16 = 160;

const BALL_SIZE: u16 = 2;

struct Ball {
    x: u16,
    y: u16,
    dx: i16,
    dy: i16,
}

impl Ball {
    fn new(x: u16, y: u16) -> Self {
        Self { x, y, dx: 1, dy: 1 }
    }

    fn update(&mut self) {
        if self.y <= 0 || self.y + BALL_SIZE >= SCREEN_HEIGHT {
            self.dy = -self.dy;
        }

        if self.x + BALL_SIZE >= 1 && self.x <= 1 {
            self.dx = -self.dx;
            self.dy = -self.dy;
        }

        if self.x + BALL_SIZE >= SCREEN_WIDTH && self.x <= SCREEN_WIDTH {
            self.dx = -self.dx;
            self.dy = -self.dy;
        }

        self.x = (self.x as i16 + self.dx) as u16;
        self.y = (self.y as i16 + self.dy) as u16;
    }
}

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
            .with_scale(TimerScale::_64)
            .with_enabled(true),
    );

    let mut keys = StatefulKeys::new();
    let mut ball = Ball::new(SCREEN_WIDTH as u16 / 2, SCREEN_HEIGHT as u16 / 2);

    let mut game_state = GameState::new();

    loop {
        if let GameState::Run = game_state.update(&mut keys) {
            ball.update();
        }

        draw_sprites(&ball);

        VBlankIntrWait();
    }
}

fn draw_sprites(ball: &Ball) {
    unsafe {
        let p = VIDEO3_VRAM.as_usize() as *mut u8;
        __aeabi_memset(p, 240 * 160 * 2, 0)
    }

    draw_rect(ball.x, ball.y, BALL_SIZE, BALL_SIZE, Color::WHITE);
}

fn draw_rect(x: u16, y: u16, width: u16, height: u16, color: Color) {
    for i in 0..width {
        for j in 0..height {
            VIDEO3_VRAM
                .index((x + i) as usize, (y + j) as usize)
                .write(color);
        }
    }
}
