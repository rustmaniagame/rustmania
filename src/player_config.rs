use ggez::graphics;

pub struct NoteLayout {
    pub column_positions: [i64; 4],
    pub arrow_sprite: graphics::Image,
    pub receptor_sprite: graphics::Image,
    pub receptor_height: i64,
}

impl NoteLayout {
    pub fn new(column_positions: [i64; 4], arrow_sprite: graphics::Image, receptor_sprite: graphics::Image, receptor_height: i64) -> NoteLayout {
        NoteLayout {
            column_positions,
            arrow_sprite,
            receptor_sprite,
            receptor_height,
        }
    }
}