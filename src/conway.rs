pub struct ConwaysState<const WIDTH: usize, const HEIGHT: usize> {
    pub states: [[bool; WIDTH]; HEIGHT],
}

impl<const WIDTH: usize, const HEIGHT: usize> ConwaysState<WIDTH, HEIGHT> {
    pub fn new() -> Self {
        Self {
            states: [[false; WIDTH]; HEIGHT],
        }
    }

    pub fn toggle_cell(&mut self, x: u16, y: u16) {
        self.states[usize::from(y)][usize::from(x)] = !self.states[usize::from(y)][usize::from(x)];
    }

    pub fn step(&mut self) {}
}
