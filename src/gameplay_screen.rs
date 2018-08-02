extern crate ggez;
extern crate chrono;

use std::result::Result;
use ggez::graphics;
use std::time::{Instant, Duration};
use super::player_config;

pub struct GameplayScreen {
    notefield: Notefield,
    p2notefield: Notefield,
}

pub struct Notefield {
    layout: super::player_config::NoteLayout,
    notes: [Vec<Duration>; 4],
    start_time: Option<Instant>,
}

impl Notefield {
    pub fn new(layout: player_config::NoteLayout, notes: [Vec<Duration>; 4]) -> Self {
        Notefield {
            layout,
            notes,
            start_time: None,
        }
    }
    pub fn add_notes(&mut self, new_notes: &mut [Vec<Duration>]) {
        for (column, new_notes) in self.notes.iter_mut().zip(new_notes.iter_mut()) {
            column.append(new_notes);
            column.sort();
        }
    }
    fn draw_field(&mut self, ctx: &mut ggez::Context) -> Result<(), ggez::GameError> {
        for &column_position in self.layout.column_positions.iter() {
            graphics::draw(ctx, &self.layout.receptor_sprite, graphics::Point2::new(column_position as f32, self.layout.receptor_height as f32), 0.0)?;
        }
        if self.start_time.is_none() {
            return Ok(());
        }
        let current_time = Instant::now();
        for (column_index, column_data) in self.notes.iter().enumerate() {
            for note in column_data.iter() {
                let distance = to_milliseconds(*note) - to_milliseconds(current_time.duration_since(self.start_time.unwrap()));
                let position = (distance as f32 * self.layout.scroll_speed) as i64 + self.layout.receptor_height;
                //let note_graphic = &graphics::Image::solid(ctx, 32, graphics::Color::from_rgb(128,128,128)).unwrap();
                graphics::draw(ctx, &self.layout.arrow_sprite, graphics::Point2::new(self.layout.column_positions[column_index] as f32, position as f32), 0.0)?;
            }
        }
        Ok(())
    }
}

fn to_milliseconds(dur: Duration) -> i64 {
    dur.as_secs() as i64 * 1000 + dur.subsec_millis() as i64
}

impl GameplayScreen {
    pub fn new(layout: player_config::NoteLayout, notes: [Vec<Duration>; 4], p2layout: player_config::NoteLayout, p2notes: [Vec<Duration>; 4]) -> Self {
        GameplayScreen {
            notefield: Notefield::new(layout, notes),
            p2notefield: Notefield::new(p2layout, p2notes),
        }
    }
    pub fn start(&mut self) {
        self.notefield.start_time = Some(Instant::now());
        self.p2notefield.start_time = Some(Instant::now());
    }
}

impl ggez::event::EventHandler for GameplayScreen {
    fn update(&mut self, _ctx: &mut ggez::Context) -> Result<(), ggez::GameError> {
        Ok(())
    }
    fn draw(&mut self, ctx: &mut ggez::Context) -> Result<(), ggez::GameError> {
        graphics::clear(ctx);
        self.notefield.draw_field(ctx)?;
        self.p2notefield.draw_field(ctx)?;
        graphics::present(ctx);
        Ok(())
    }
}