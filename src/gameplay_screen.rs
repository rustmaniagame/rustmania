extern crate chrono;
extern crate ggez;

use player_config;
use ggez::audio;
use ggez::graphics;
use ggez::graphics::spritebatch::SpriteBatch;
use std::result::Result;
use std::time::{Duration, Instant};
use timingdata;
use notefield::Notefield;

pub struct GameplayScreen<'a> {
    notefield: Notefield<'a>,
    p2notefield: Notefield<'a>,
    music: audio::Source,
    start_time: Option<Instant>,
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
