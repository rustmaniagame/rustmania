use ggez::{
    event::{EventHandler, Keycode, Mod}, graphics, Context, GameError,
};
use std::time::{Duration, Instant};

pub trait Element {
    fn run(&self, Option<i64>);
    fn start(&self);
}

pub struct Screen {
    start_time: Option<Instant>,
    elements: Vec<Box<dyn Element>>,
    key_handler: (),
}

fn to_milliseconds(dur: Duration) -> i64 {
    dur.as_secs() as i64 * 1000 + dur.subsec_millis() as i64
}

impl Screen {
    pub fn new(elements: Vec<Box<dyn Element>>) -> Self {
        Screen {
            start_time: None,
            elements,
            key_handler: (),
        }
    }
    pub fn start(&mut self) -> Result<(), GameError> {
        self.start_time = Some(Instant::now());
        for element in self.elements.iter() {
            element.start();
        }
        Ok(())
    }
    fn start_time_to_milliseconds(&self) -> Option<i64> {
        match self.start_time {
            Some(time) => Some(to_milliseconds(Instant::now().duration_since(time))),
            None => None,
        }
    }
}

impl<'a> EventHandler for Screen {
    fn update(&mut self, _ctx: &mut Context) -> Result<(), GameError> {
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        graphics::clear(ctx);
        let time_delta = self.start_time_to_milliseconds();
        for element in self.elements.iter() {
            element.run(time_delta);
        }
        graphics::present(ctx);
        Ok(())
    }
    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: Keycode,
        _keymod: Mod,
        _repeat: bool,
    ) {
        if _repeat {
            return;
        }
        let time_delta = self.start_time_to_milliseconds();
    }
}
