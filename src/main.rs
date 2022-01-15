#![no_std]
#![no_main]

use gba::prelude::*;

#[panic_handler]
#[allow(unused)]
fn panic(info: &core::panic::PanicInfo) -> ! {
    loop {
        DISPCNT.read();
    }
}

#[no_mangle]
pub fn main() -> ! {
    const SETTING: DisplayControl = DisplayControl::new()
        .with_display_mode(3)
        .with_display_bg2(true);
    DISPCNT.write(SETTING);

    for row in 0..mode3::HEIGHT {
        for col in 0..mode3::WIDTH {
            let keys: Keys = KEYINPUT.read().into();
            //if keys.a() {
                const GREEN: Color = Color::from_rgb(0, 31, 0);
                write_pixel(col, row, GREEN);
            //}
            spin_cycle();
        }
    }

    loop {}
}

fn write_pixel(x: usize, y: usize, color: Color) {
    mode3::bitmap_xy(x, y).write(color);
}

fn spin_cycle() {
    for _ in 0..8 {
        while VCOUNT.read() < 160 {}
        while VCOUNT.read() >= 160 {}
    }
}