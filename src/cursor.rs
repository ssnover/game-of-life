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
        dirty_row_buffer: &mut [u8],
    ) {
        self.counter += 1;
        if self.counter == MAX_COUNTER {
            self.counter = 0;
        }

        if self.counter == 0 {
            self.mark_rows_dirty(dirty_row_buffer);

            if keys.left() {
                self.x = self.x.saturating_sub(1);
            } else if keys.right() {
                self.x = core::cmp::min(self.x.saturating_add(1), BOARD_WIDTH as u16 - 1);
            }
            if keys.up() {
                dirty_row_buffer[(self.y - 1) as usize / 8] |= 1 << ((self.y - 1) % 8);
                self.y = self.y.saturating_sub(1);
            } else if keys.down() {
                self.y = core::cmp::min(self.y.saturating_add(1), BOARD_HEIGHT as u16 - 1);
            }

            self.mark_rows_dirty(dirty_row_buffer);
        }

        if let Some(edge) = keys.a().change() {
            if let Edge::Rising = edge {
                board.toggle_cell(self.x as usize, self.y as usize);
                dirty_row_buffer[self.y as usize / 8] |= 1 << (self.y % 8);
            }
        }
    }

    fn mark_rows_dirty(&self, dirty_row_buffer: &mut [u8]) {
        for row in (self.y.saturating_sub(1) as usize)
            ..=core::cmp::min(self.y.saturating_add(1) as usize, BOARD_WIDTH - 1)
        {
            dirty_row_buffer[row / 8] |= 1 << (row % 8);
        }
    }
}
