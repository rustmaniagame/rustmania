use ggez::graphics;

pub struct NoteLayout {
    pub column_positions: [u32; 4],
    pub sprite: graphics::Image,
}

impl NoteLayout {
    pub fn new(sprite: graphics::Image) -> NoteLayout {
        NoteLayout {
            column_positions: [300, 350, 400, 450],
            sprite,
        }
    }
}