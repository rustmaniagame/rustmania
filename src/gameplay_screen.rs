extern crate ggez;
extern crate chrono;

use std::result::Result;
use ggez::graphics;
use std::time::{Instant, Duration};
use super::player_config;

pub struct GameplayScreen<'a> {
    notefield: Notefield<'a>,
    p2notefield: Notefield<'a>,
}

pub struct Notefield<'a> {
    layout: &'a super::player_config::NoteLayout,
    notes: &'a [Vec<i64>; 4],
    start_time: Option<Instant>,
    on_screen: Vec<(usize,usize)>,
    draw_distance: i64,
}

impl<'a> Notefield<'a> {
    pub fn new(layout: &'a player_config::NoteLayout, notes: &'a [Vec<i64>; 4], draw_distance: i64) -> Self {
        Notefield {
            layout,
            notes,
            start_time: None,
            on_screen: Vec::<_>::new(),
            draw_distance,
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
        let time_delta = to_milliseconds(current_time.duration_since(self.start_time.unwrap()));
        for (column_index, (column_data, (draw_start, draw_end))) in self.notes.iter().zip(&mut self.on_screen).enumerate() {
            for note in column_data[*draw_start..*draw_end].iter() {
                let note_delta = *note - time_delta;
                let position = self.layout.delta_to_position(note_delta);
                graphics::draw(ctx, &self.layout.arrow_sprite, graphics::Point2::new(self.layout.column_positions[column_index] as f32, position as f32), 0.0)?;
            }
            if self.layout.delta_to_position(self.notes[column_index][*draw_end] - time_delta) < self.draw_distance && *draw_end != column_data.len()-1 {
                *draw_end += 1;
            }
        }
        Ok(())
    }
}

fn to_milliseconds(dur: Duration) -> i64 {
    dur.as_secs() as i64 * 1000 + dur.subsec_millis() as i64
}

impl<'a> GameplayScreen<'a> {
    pub fn new(layout: &'a player_config::NoteLayout, notes: &'a [Vec<i64>; 4], p2layout: &'a player_config::NoteLayout, p2notes: &'a [Vec<i64>; 4], draw_distance: i64) -> Self {
        GameplayScreen {
            notefield: Notefield::new(layout, notes, draw_distance),
            p2notefield: Notefield::new(p2layout, p2notes, draw_distance),
        }
    }
    pub fn start(&mut self) {
        self.notefield.start_time = Some(Instant::now());
        self.notefield.on_screen = self.notefield.notes.iter().map(|x| (0, match x.iter().position(|y| *y > self.notefield.draw_distance) {Some(num)=> num, None => x.len()})).collect();

        self.p2notefield.start_time = Some(Instant::now());
        self.p2notefield.on_screen = self.p2notefield.notes.iter().map(|x| (0, match x.iter().position(|y| *y > self.p2notefield.draw_distance) {Some(num)=> num, None => x.len()})).collect();

    }
}

impl<'a> ggez::event::EventHandler for GameplayScreen<'a> {
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