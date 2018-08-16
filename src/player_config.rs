extern crate ggez;
use ggez::{graphics, Context};

pub struct NoteLayout {
    pub column_positions: [i64; 4],
    pub arrows_sprite: graphics::Image,
    pub receptor_sprite: graphics::Image,
    pub receptor_height: i64,
    pub scroll_speed: f32,
}

impl NoteLayout {
    pub fn new(
        column_positions: [i64; 4],
        arrows_sprite: graphics::Image,
        receptor_sprite: graphics::Image,
        receptor_height: i64,
    ) -> NoteLayout {
        NoteLayout {
            column_positions,
            arrows_sprite,
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
    pub fn delta_to_position(&self, delta: i64) -> i64 {
        (delta as f32 * self.scroll_speed) as i64 + self.receptor_height
    }
    pub fn draw_note_at_position<'a>(
        &self,
        ctx: &mut Context,
        column: usize,
        position: i64,
        sprite: &'a graphics::Image,
    ) -> Result<(), ggez::GameError> {
        graphics::draw(
            ctx,
            sprite,
            graphics::Point2::new(self.column_positions[column] as f32, position as f32),
            0.0,
        )?;
        Ok(())
    }
    pub fn draw_column_of_notes<'a>(
        &self,
        ctx: &mut ggez::Context,
        column: impl Iterator<Item = (i64, &'a graphics::Image)>,
        column_index: usize,
    ) -> Result<(), ggez::GameError> {
        for (note, sprite) in column {
            self.draw_note_at_position(ctx, column_index, self.delta_to_position(note), sprite)?;
        }
        Ok(())
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
}
