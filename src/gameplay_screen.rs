extern crate ggez;
extern crate chrono;

use std::result::Result;
use ggez::graphics;
use std::time::{Instant, Duration};

pub struct GameplayScreen {
    layout: super::player_config::NoteLayout,
    notes: Vec<Duration>,
    start_time: Option<Instant>,
}

impl GameplayScreen {
    pub fn new() -> Self {
        GameplayScreen {
            layout: super::player_config::NoteLayout::new(),
            notes: Vec::new(),
            start_time: None,
        }
    }
    pub fn add_notes(&mut self, new_notes: &mut Vec<Duration>) {
        self.notes.append(new_notes);
        self.notes.sort();
    }
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }
}

fn to_millis(dur: Duration) -> i64 {
    dur.as_secs() as i64 * 1000 + dur.subsec_millis() as i64
}

impl ggez::event::EventHandler for GameplayScreen {
    fn update(&mut self, ctx: &mut ggez::Context) -> Result<(), ggez::GameError> {
        Ok(())
    }
    fn draw(&mut self, ctx: &mut ggez::Context) -> Result<(), ggez::GameError> {
        graphics::clear(ctx);
        if self.start_time.is_none() {
            return Ok(());
        }
        let mut distance = 0;
        for note in self.notes.iter() {
            distance = to_millis(*note) - to_millis(Instant::now().duration_since(self.start_time.unwrap()));
            let catdog = &graphics::Image::solid(ctx, 32, graphics::Color::from_rgb(128,128,128)).unwrap();
            graphics::draw(ctx, catdog, graphics::Point2::new(self.layout.column_positions[0] as f32, distance as f32), 0.0)?;
        }
        graphics::present(ctx);
        Ok(())
    }
}