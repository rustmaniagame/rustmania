extern crate ggez;
extern crate chrono;

use std::result::Result;
use ggez::graphics;
use std::time::{Instant, Duration};
use super::player_config;
use notedata;

pub struct GameplayScreen<'a> {
    notefield: Notefield<'a>,
    p2notefield: Notefield<'a>,
    start_time: Option<Instant>,
}

pub struct Notefield<'a> {
    layout: &'a super::player_config::NoteLayout,
    notes: &'a notedata::TimingData,
    on_screen: Vec<(usize,usize)>,
    draw_distance: i64,
}

impl<'a> Notefield<'a> {
    pub fn new(layout: &'a player_config::NoteLayout, notes: &'a notedata::TimingData, draw_distance: i64) -> Self {
        Notefield {
            layout,
            notes,
            on_screen: Vec::<_>::new(),
            draw_distance,
        }
    }
    fn draw_field(&mut self, ctx: &mut ggez::Context, time: Option<i64>) -> Result<(), ggez::GameError> {
        self.layout.draw_receptors(ctx);
        if time.is_none() {
            return Ok(());
        }
        let time = time.unwrap();
        for ((column_index, column_data), (draw_start, draw_end)) in self.notes.notes.iter().enumerate().zip(&mut self.on_screen) {
            if *draw_end != column_data.len() && self.layout.delta_to_position(column_data[*draw_end] - time) < self.draw_distance {
                *draw_end += 1;
            }
            if *draw_start != column_data.len() && column_data[*draw_start] - time < -180 {
                *draw_start += 1;
            }
            for note in column_data[*draw_start..*draw_end].iter() {
                let note_delta = *note - time;
                let position = self.layout.delta_to_position(note_delta);
                self.layout.draw_note_at_position(ctx, column_index, position)?;
            }
        }
        Ok(())
    }
}

fn to_milliseconds(dur: Duration) -> i64 {
    dur.as_secs() as i64 * 1000 + dur.subsec_millis() as i64
}

impl<'a> GameplayScreen<'a> {
    pub fn new(layout: &'a player_config::NoteLayout, notes: &'a notedata::TimingData, p2layout: &'a player_config::NoteLayout, p2notes: &'a notedata::TimingData, draw_distance: i64) -> Self {
        GameplayScreen {
            notefield: Notefield::new(layout, notes, draw_distance),
            p2notefield: Notefield::new(p2layout, p2notes, draw_distance),
            start_time: None,
        }
    }
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
        self.notefield.on_screen = self.notefield.notes.notes.iter().map(|x| (0, match x.iter().position(|y| *y > self.notefield.draw_distance) {Some(num)=> num, None => x.len()})).collect();

        self.p2notefield.on_screen = self.p2notefield.notes.notes.iter().map(|x| (0, match x.iter().position(|y| *y > self.p2notefield.draw_distance) {Some(num)=> num, None => x.len()})).collect();

    }
}

impl<'a> ggez::event::EventHandler for GameplayScreen<'a> {
    fn update(&mut self, _ctx: &mut ggez::Context) -> Result<(), ggez::GameError> {
        Ok(())
    }
    fn draw(&mut self, ctx: &mut ggez::Context) -> Result<(), ggez::GameError> {
        graphics::clear(ctx);
        let time_delta = match self.start_time {
            Some(time) => Some(to_milliseconds(Instant::now().duration_since(time))),
            None => None
        };
        self.notefield.draw_field(ctx, time_delta)?;
        self.p2notefield.draw_field(ctx, time_delta)?;
        graphics::present(ctx);
        Ok(())
    }
}