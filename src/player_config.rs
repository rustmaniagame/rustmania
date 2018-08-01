use ggez::graphics;

pub struct NoteLayout {
    pub column_positions: [i64; 4],
    pub arrow_sprite: graphics::Image,
    pub receptor_sprite: graphics::Image,
    pub receptor_height: i64,
    pub scroll_speed: f32,
}

impl NoteLayout {
    pub fn new(column_positions: [i64; 4], arrow_sprite: graphics::Image, receptor_sprite: graphics::Image, receptor_height: i64) -> NoteLayout {
        NoteLayout {
            column_positions,
            arrow_sprite,
            receptor_sprite,
            receptor_height,
            scroll_speed: 1.0,
        }
    }
    pub fn set_scroll_speed(&mut self, new_speed: f32) -> Result<(), String> {
        if !new_speed.is_sign_positive() {
            return Err(String::from("scroll speed not positive"));
        }
        self.scroll_speed = new_speed;
        Ok(())
    }
}