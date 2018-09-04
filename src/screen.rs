use ggez::{
    audio::Source, event::{EventHandler, Keycode, Mod}, graphics, Context, GameError,
};
use std::time::{Duration, Instant};

pub trait Element {
    fn run(&mut self, &mut Context, Option<i64>) -> Result<(), GameError>;
    fn start(&mut self) -> Result<(), GameError>;
    fn handle_event(&mut self, Keycode, Option<i64>) -> Result<(), GameError>;
}

pub struct Screen<'a> {
    start_time: Option<Instant>,
    elements: Vec<Box<dyn Element + 'a>>,
    key_handler: (),
}

fn to_milliseconds(dur: Duration) -> i64 {
    dur.as_secs() as i64 * 1000 + dur.subsec_millis() as i64
}

impl<'a> Screen<'a> {
    pub fn new(elements: Vec<Box<dyn Element + 'a>>) -> Self {
        Screen {
            start_time: None,
            elements,
            key_handler: (),
        }
    }
    pub fn start(&mut self) -> Result<(), GameError> {
        self.start_time = Some(Instant::now());
        for element in self.elements.iter_mut() {
            element.start()?;
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

impl<'a> EventHandler for Screen<'a> {
    fn update(&mut self, _ctx: &mut Context) -> Result<(), GameError> {
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        graphics::clear(ctx);
        let time_delta = self.start_time_to_milliseconds();
        for element in self.elements.iter_mut() {
            element.run(ctx, time_delta)?;
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

impl Element for Source {
    fn run(&mut self, ctx: &mut Context, time: Option<i64>) -> Result<(), GameError> {
        Ok(())
    }
    fn start(&mut self) -> Result<(), GameError> {
        self.play()?;
        Ok(())
    }
    fn handle_event(&mut self, keycode: Keycode, time: Option<i64>) -> Result<(),GameError>{
        Ok(())
    }
}
