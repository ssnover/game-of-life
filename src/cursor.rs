use crate::{
    conway::ConwaysState,
    keys::{Edge, StatefulKeys},
    BOARD_HEIGHT, BOARD_WIDTH, BOARD_WIDTH_BYTES,
};

const MAX_COUNTER: u8 = 4;

pub struct Cursor {
    counter: u8,
    pub x: u16,
    pub y: u16,
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            counter: 0,
            x: BOARD_WIDTH as u16 / 2,
            y: BOARD_HEIGHT as u16 / 2,
        }
    }

    pub fn update(
        &mut self,
        keys: &mut StatefulKeys,
        board: &mut ConwaysState<BOARD_WIDTH_BYTES, BOARD_HEIGHT>,
    ) {
        self.counter += 1;
        if self.counter == MAX_COUNTER {
            self.counter = 0;
        }

        if self.counter == 0 {
            if keys.left() {
                self.x = self.x.saturating_sub(1);
            } else if keys.right() {
                self.x = core::cmp::min(self.x.saturating_add(1), BOARD_WIDTH as u16 - 1);
            }
            if keys.up() {
                self.y = self.y.saturating_sub(1);
            } else if keys.down() {
                self.y = core::cmp::min(self.y.saturating_add(1), BOARD_HEIGHT as u16 - 1);
            }
        }

        if let Some(edge) = keys.a().change() {
            if let Edge::Rising = edge {
                board.toggle_cell(self.x as usize, self.y as usize);
            }
        }
    }
}
