use ggez::graphics;

pub struct NoteLayout {
    pub column_positions: [u32; 4],
    pub arrow_sprite: graphics::Image,
    pub receptor_sprite: graphics::Image
}

impl NoteLayout {
    pub fn new(arrow_sprite: graphics::Image, receptor_sprite: graphics::Image) -> NoteLayout {
        NoteLayout {
            column_positions: [300, 350, 400, 450],
            arrow_sprite,
            receptor_sprite
        }
    }
}