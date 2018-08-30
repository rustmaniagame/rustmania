extern crate ggez;
use ggez::graphics;

pub struct NoteLayout {
    pub column_positions: [i64; 4],
    pub arrows_sprite: graphics::Image,
    pub receptor_sprite: graphics::Image,
    pub judgment_sprite: graphics::Image,
    pub receptor_height: i64,
    pub scroll_speed: f32,
}

impl NoteLayout {
    pub fn new(
        column_positions: [i64; 4],
        arrows_sprite: graphics::Image,
        receptor_sprite: graphics::Image,
        judgment_sprite: graphics::Image,
        receptor_height: i64,
    ) -> NoteLayout {
        NoteLayout {
            column_positions,
            arrows_sprite,
            receptor_sprite,
            judgment_sprite,
            receptor_height,
            scroll_speed: 1.0,
        }
    }
    //noinspection RsUnresolvedReference
    pub fn set_scroll_speed(&mut self, new_speed: f32) -> Result<(), String> {
        if !new_speed.is_sign_positive() {
            return Err(String::from("scroll speed not positive"));
        }
        self.scroll_speed = new_speed;
        Ok(())
    }
    pub fn delta_to_position(&self, delta: i64) -> i64 {
        (delta as f32 * self.scroll_speed) as i64 + self.receptor_height
    }
    pub fn delta_to_offset(&self, delta: i64) -> f32 {
        (delta as f32 * self.scroll_speed)
    }
    pub fn add_note(
        &self,
        column: usize,
        position: i64,
        coords: graphics::Rect,
        batch: &mut graphics::spritebatch::SpriteBatch,
    ) {
        batch.add(graphics::DrawParam {
            src: coords,
            dest: graphics::Point2::new(self.column_positions[column] as f32, position as f32),
            ..Default::default()
        });
    }
    pub fn add_column_of_notes<'a>(
        &self,
        column: impl Iterator<Item = (i64, graphics::Rect)>,
        column_index: usize,
        batch: &mut graphics::spritebatch::SpriteBatch,
    ) {
        for (note, coords) in column {
            self.add_note(column_index, self.delta_to_position(note), coords, batch);
        }
    }
    pub fn draw_receptors(&self, ctx: &mut ggez::Context) -> Result<(), ggez::GameError> {
        for &column_position in self.column_positions.iter() {
            graphics::draw(
                ctx,
                &self.receptor_sprite,
                graphics::Point2::new(column_position as f32, self.receptor_height as f32),
                0.0,
            )?;
        }
        Ok(())
    }
    //this will likely be the method to draw receptors in the future, but it is not currently in use
    pub fn _add_receptors(
        &self,
        batch: &mut graphics::spritebatch::SpriteBatch,
    ) -> Result<(), ggez::GameError> {
        for &column_position in self.column_positions.iter() {
            batch.add(graphics::DrawParam {
                dest: graphics::Point2::new(column_position as f32, self.receptor_height as f32),
                ..Default::default()
            });
        }
        Ok(())
    }
    pub fn draw_judgment(&self, ctx: &mut ggez::Context) -> Result<(), ggez::GameError> {
        graphics::draw(
            ctx,
            &self.judgment_sprite,
            graphics::Point2::new(0.0, 0.0),
            0.0,
        )?;
        Ok(())
    }
}
