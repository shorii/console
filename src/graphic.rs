pub struct Graphic {
    content: Vec<u8>,
    row_length: usize,
    cursor: usize,
}

impl Graphic {
    pub fn new(content: Vec<u8>, row_length: usize) -> Self {
        assert!(content.len() >= row_length);
        match content.len() % row_length {
            0 => Graphic {
                content,
                row_length,
                cursor: 0,
            },
            _ => panic!("invalid row_length"),
        }
    }
}

impl Iterator for Graphic {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.content.len() {
            return None;
        }
        let next_cursor = self.cursor + self.row_length;
        let item = self.content[self.cursor..next_cursor].to_vec();
        self.cursor += self.row_length;
        Some(item)
    }
}