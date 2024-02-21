static SPINNER_CHARS: &[char] = &['⦾', '⦿'];

#[derive(Default, Clone, Copy, Debug)]
pub struct Spinner {
    pub active: bool,
    pub index: usize,
}

impl Spinner {
    pub fn draw(&self) -> char {
        SPINNER_CHARS[self.index]
    }

    pub fn update(&mut self) {
        self.index += 1;
        self.index %= SPINNER_CHARS.len();
    }
}
