use rand_xoshiro::rand_core::{RngCore, SeedableRng};
use rand_xoshiro::Xoshiro128StarStar;

type BoardState<const X: usize, const Y: usize> = [[u8; X]; Y];

pub struct ConwaysState<const WIDTH: usize, const HEIGHT: usize> {
    states: BoardState<WIDTH, HEIGHT>,
}

impl<const WIDTH: usize, const HEIGHT: usize> ConwaysState<WIDTH, HEIGHT> {
    pub fn new() -> Self {
        Self {
            states: [[0; WIDTH]; HEIGHT],
        }
    }

    #[inline]
    pub fn get_cell(&self, col: usize, row: usize) -> bool {
        (self.states[row][col / 8] & (1 << (col % 8))) != 0
    }

    #[inline]
    pub fn set_cell(&mut self, col: usize, row: usize, val: bool) {
        if val {
            self.states[row][col / 8] |= 1 << (col % 8);
        } else {
            self.states[row][col / 8] &= !(1 << (col % 8));
        }
    }

    #[inline]
    pub fn toggle_cell(&mut self, col: usize, row: usize) {
        self.set_cell(col, row, !self.get_cell(col, row));
    }

    pub fn step(&mut self, old: &ConwaysState<WIDTH, HEIGHT>, dirty_row_buffer: &mut [u8]) {
        for col in 0..(WIDTH * 8) {
            for row in 0..HEIGHT {
                match (old.get_cell(col, row), old.neighbors_alive(col, row)) {
                    (true, 2..=3) => {
                        self.set_cell(col, row, true);
                    }
                    (false, 3) => {
                        self.set_cell(col, row, true);
                        dirty_row_buffer[row / 8] |= 1 << (row % 8);
                    }
                    (true, _) => {
                        self.set_cell(col, row, false);
                        dirty_row_buffer[row / 8] |= 1 << (row % 8);
                    }
                    (false, _) => {
                        self.set_cell(col, row, false);
                    }
                }
            }
        }
    }

    pub fn neighbors_alive(&self, x: usize, y: usize) -> u8 {
        let lower_x = x.saturating_sub(1);
        let upper_x = core::cmp::min(x + 1, (WIDTH * 8) - 1);
        let lower_y = y.saturating_sub(1);
        let upper_y = core::cmp::min(y + 1, HEIGHT - 1);

        let mut count = 0;
        for col in lower_x..=upper_x {
            for row in lower_y..=upper_y {
                if col == x && row == y {
                    continue;
                }
                if self.get_cell(col, row) {
                    count += 1;
                }
            }
        }

        count
    }

    pub fn randomize(&mut self, seed: u32) {
        let mut rng = Xoshiro128StarStar::seed_from_u64(seed as u64);
        let mut rand_val = rng.next_u32();
        let mut rand_counter = 0;

        for col in 0..(WIDTH * 8) {
            for row in 0..HEIGHT {
                self.set_cell(col, row, ((rand_val >> (rand_counter * 2)) & 0b11) == 0);
                rand_counter = (rand_counter + 1) % 16;
                if rand_counter == 0 {
                    rand_val = rng.next_u32();
                }
            }
        }
    }
}
