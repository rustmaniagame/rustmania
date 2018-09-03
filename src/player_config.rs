extern crate ggez;
use ggez::graphics;
use notefield::Judgement;
use timingdata::GameplayInfo;

#[derive(PartialEq)]
pub struct NoteLayout {
    pub arrows_sprite: graphics::Image,
    pub receptor_sprite: graphics::Image,
    pub judgment_sprite: graphics::Image,
    pub column_positions: [i64; 4],
    pub receptor_height: i64,
    pub judgment_position: graphics::Point2,
    pub scroll_speed: f32,
}

impl NoteLayout {
    pub fn new(
        column_positions: [i64; 4],
        arrows_sprite: graphics::Image,
        receptor_sprite: graphics::Image,
        judgment_sprite: graphics::Image,
        receptor_height: i64,
        judgment_position: graphics::Point2,
    ) -> NoteLayout {
        NoteLayout {
            column_positions,
            arrows_sprite,
            receptor_sprite,
            judgment_sprite,
            receptor_height,
            judgment_position,
            scroll_speed: 1.0,
        }
    }
    //noinspection RsUnresolvedReference
    pub fn set_scroll_speed(&mut self, new_speed: f32) {
        self.scroll_speed = new_speed;
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
        column: impl Iterator<Item = GameplayInfo>,
        column_index: usize,
        batch: &mut graphics::spritebatch::SpriteBatch,
    ) {
        for GameplayInfo(note, coords) in column {
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
    pub fn draw_judgment(
        &self,
        ctx: &mut ggez::Context,
        judge: Judgement,
    ) -> Result<(), ggez::GameError> {
        let src = match judge {
            Judgement::Hit(0) => graphics::Rect::new(0.0, 0.0, 1.0, 0.1666),
            Judgement::Hit(1) => graphics::Rect::new(0.0, 0.1666, 1.0, 0.1666),
            Judgement::Hit(2) => graphics::Rect::new(0.0, 0.3333, 1.0, 0.1666),
            Judgement::Hit(3) => graphics::Rect::new(0.0, 0.5, 1.0, 0.1666),
            Judgement::Hit(_) => graphics::Rect::new(0.0, 0.6666, 1.0, 0.1666),
            Judgement::Miss => graphics::Rect::new(0.0, 0.8333, 1.0, 1.666),
        };
        let param = graphics::DrawParam {
            src,
            dest: self.judgment_position,
            ..Default::default()
        };
        graphics::draw_ex(ctx, &self.judgment_sprite, param)?;
        Ok(())
    }
}
