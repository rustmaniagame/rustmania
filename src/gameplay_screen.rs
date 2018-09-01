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
    last_judgement: Option<Judgement>,
}

#[derive(Copy, Clone)]
pub enum Judgement {
    Marvelous,
    Perfect,
    Great,
    Good,
    Bad,
    Miss,
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
            last_judgement: None,
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
    fn redraw_batch(&mut self) {
        self.batch.clear();
        for ((column_index, column_data), (draw_start, draw_end)) in
            self.notes.columns().enumerate().zip(&mut self.on_screen)
        {
            if *draw_start < *draw_end {
                self.layout.add_column_of_notes(
                    column_data[*draw_start..*draw_end].iter().map(|x| *x),
                    column_index,
                    &mut self.batch,
                );
            }
        }
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
                if *draw_start <= *draw_end {
                    self.layout.add_note(
                        column_index,
                        self.layout.delta_to_position(column_data[*draw_end].0),
                        column_data[*draw_end].1,
                        &mut self.batch,
                    );
                }
                *draw_end += 1;
            }
            while *draw_start != column_data.len() && column_data[*draw_start].0 - time < -180 {
                *draw_start += 1;
                clear_batch = true;
            }
        }
        if clear_batch {
            self.redraw_batch();
            self.last_judgement = Some(Judgement::Miss);
        }
        let target_parameter = graphics::DrawParam {
            dest: graphics::Point2::new(0.0, -1.0 * (self.layout.delta_to_offset(time))),
            ..Default::default()
        };
        graphics::draw_ex(ctx, &self.batch, target_parameter)?;
        if let Some(judgment) = self.last_judgement {
            self.layout.draw_judgment(ctx, judgment)?;
        }
        Ok(())
    }
    fn handle_event(&mut self, keycode: ggez::event::Keycode, time: Option<i64>) {
        let index = match keycode {
            ggez::event::Keycode::Z => 0,
            ggez::event::Keycode::X => 1,
            ggez::event::Keycode::Comma => 2,
            ggez::event::Keycode::Period => 3,
            _ => return,
        };
        let delta = self.notes.columns().collect::<Vec<_>>()[index].get(self.on_screen[index].0);
        if let (Some(time), Some((delta, _))) = (time, delta) {
            let offset = delta - time;
            if offset < 180 {
                self.on_screen[index].0 += 1;
                self.handle_judgement(offset);
            }
        }
        self.redraw_batch();
    }
    //noinspection RsUnresolvedReference
    fn handle_judgement(&mut self, offset: i64) {
        let abs_offset = offset.abs();
        match abs_offset {
            0...22 => self.last_judgement = Some(Judgement::Marvelous),
            23...45 => self.last_judgement = Some(Judgement::Perfect),
            46...90 => self.last_judgement = Some(Judgement::Great),
            91...135 => self.last_judgement = Some(Judgement::Good),
            136...180 => self.last_judgement = Some(Judgement::Bad),
            _ => {}
        }
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
    fn key_down_event(
        &mut self,
        _ctx: &mut ggez::Context,
        keycode: ggez::event::Keycode,
        _keymod: ggez::event::Mod,
        _repeat: bool,
    ) {
        if _repeat {
            return;
        }
        let time_delta = self.start_time_to_milliseconds();
        self.notefield.handle_event(keycode, time_delta);
        self.p2notefield.handle_event(keycode, time_delta);
    }
}
