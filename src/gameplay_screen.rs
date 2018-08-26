extern crate chrono;
extern crate ggez;

use super::player_config;
use ggez::audio;
use ggez::graphics;
use ggez::graphics::spritebatch::SpriteBatch;
use std::result::Result;
use std::time::{Duration, Instant};
use timingdata;

pub struct GameplayScreen<'a> {
    notefield: Notefield<'a>,
    p2notefield: Notefield<'a>,
    music: audio::Source,
    start_time: Option<Instant>,
}

pub struct Notefield<'a> {
    layout: &'a super::player_config::NoteLayout,
    notes: &'a timingdata::TimingData,
    on_screen: Vec<(usize, usize)>,
    batch: SpriteBatch,
    draw_distance: i64,
}

impl<'a> Notefield<'a> {
    pub fn new(
        layout: &'a player_config::NoteLayout,
        notes: &'a timingdata::TimingData,
        batch: SpriteBatch,
        draw_distance: i64,
    ) -> Self {
        Notefield {
            layout,
            notes,
            on_screen: Vec::<_>::new(),
            batch,
            draw_distance,
        }
    }
    fn start(&mut self) -> Result<(), ggez::GameError> {
        //self.layout.add_receptors(&mut self.batch)?;
        self.on_screen = self.notes
            .columns()
            .map(|x| {
                (
                    0,
                    match x.iter().position(|(y, _)| *y > self.draw_distance) {
                        Some(num) => num,
                        None => x.len(),
                    },
                )
            })
            .collect();
        Ok(())
    }
    fn draw_field(
        &mut self,
        ctx: &mut ggez::Context,
        time: Option<i64>,
    ) -> Result<(), ggez::GameError> {
        self.layout.draw_receptors(ctx)?;
        let time = match time {
            Some(time) => time,
            None => return Ok(()),
        };
        let mut clear_batch = false;
        for ((column_index, column_data), (draw_start, draw_end)) in
            self.notes.columns().enumerate().zip(&mut self.on_screen)
        {
            while *draw_end != column_data.len()
                && self.layout
                    .delta_to_position(column_data[*draw_end].0 - time)
                    < self.draw_distance
            {
                self.layout.add_note(
                    column_index,
                    self.layout.delta_to_position(column_data[*draw_end].0),
                    column_data[*draw_end].1,
                    &mut self.batch,
                )?;
                *draw_end += 1;
            }
            while *draw_start != column_data.len() && column_data[*draw_start].0 - time < -180 {
                *draw_start += 1;
                clear_batch = true;
            }
        }
        if clear_batch {
            self.batch.clear();
            for ((column_index, column_data), (draw_start, draw_end)) in
                self.notes.columns().enumerate().zip(&mut self.on_screen)
            {
                self.layout.add_column_of_notes(
                    column_data[*draw_start..*draw_end].iter().map(|x| *x),
                    column_index,
                    &mut self.batch,
                )?;
            }
        }
        let coolparam = graphics::DrawParam {
            dest: graphics::Point2::new(0.0, -1.0 * (self.layout.delta_to_offset(time))),
            ..Default::default()
        };
        graphics::draw_ex(ctx, &self.batch, coolparam)?;
        Ok(())
    }
}

fn to_milliseconds(dur: Duration) -> i64 {
    dur.as_secs() as i64 * 1000 + dur.subsec_millis() as i64
}

impl<'a> GameplayScreen<'a> {
    pub fn new(
        layout: &'a player_config::NoteLayout,
        notes: &'a timingdata::TimingData,
        p2layout: &'a player_config::NoteLayout,
        p2notes: &'a timingdata::TimingData,
        music: audio::Source,
        draw_distance: i64,
    ) -> Self {
        GameplayScreen {
            notefield: Notefield::new(
                layout,
                notes,
                SpriteBatch::new(layout.arrows_sprite.clone()),
                draw_distance,
            ),
            p2notefield: Notefield::new(
                p2layout,
                p2notes,
                SpriteBatch::new(p2layout.arrows_sprite.clone()),
                draw_distance,
            ),
            music,
            start_time: None,
        }
    }
    pub fn start(&mut self) -> Result<(), ggez::GameError> {
        self.start_time = Some(Instant::now());
        self.notefield.start()?;
        self.p2notefield.start()?;
        self.music.play()?;
        Ok(())
    }
    fn start_time_to_milliseconds(&self) -> Option<i64> {
        match self.start_time {
            Some(time) => Some(to_milliseconds(Instant::now().duration_since(time))),
            None => None,
        }
    }
}

impl<'a> ggez::event::EventHandler for GameplayScreen<'a> {
    fn update(&mut self, _ctx: &mut ggez::Context) -> Result<(), ggez::GameError> {
        Ok(())
    }
    fn draw(&mut self, ctx: &mut ggez::Context) -> Result<(), ggez::GameError> {
        graphics::clear(ctx);
        let time_delta = self.start_time_to_milliseconds();
        self.notefield.draw_field(ctx, time_delta)?;
        self.p2notefield.draw_field(ctx, time_delta)?;
        graphics::present(ctx);
        Ok(())
    }
}
