use ggez::{graphics, Context, GameError, audio::Source, event::{EventHandler, Keycode, Mod}};
use std::time::{Duration, SystemTime};

pub trait Element: Send {
    fn run(&mut self, &mut Context, Option<i64>) -> Result<(), GameError>;
    fn start(&mut self) -> Result<(), GameError>;
    fn handle_event(&mut self, Keycode, Option<i64>);
}

pub struct Screen<'a> {
    start_time: Option<SystemTime>,
    elements: Vec<Box<dyn Element + 'a>>,
    key_handler: (),
}

fn to_milliseconds(dur: Duration) -> i64 {
    dur.as_secs() as i64 * 1000 + dur.subsec_millis() as i64
}

impl<'a> Screen<'a> {
    pub fn new(elements: Vec<Box<dyn Element + 'a>>) -> Self {
        Screen {
            start_time: Some(SystemTime::now() + Duration::from_secs(3)),
            elements,
            key_handler: (),
        }
    }
    pub fn start(&mut self) -> Result<(), GameError> {
        for element in &mut self.elements {
            element.start()?;
        }
        Ok(())
    }
    fn start_time_to_milliseconds(&self) -> Option<i64> {
        match self.start_time {
            Some(time) => match SystemTime::now().duration_since(time) {
                Ok(time) => Some(to_milliseconds(time)),
                Err(negtime) => Some(-to_milliseconds(negtime.duration())),
            },
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
        for element in &mut self.elements {
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
        for element in &mut self.elements {
            element.handle_event(keycode, time_delta);
        }
    }
}

impl Element for Source {
    fn run(&mut self, _ctx: &mut Context, _time: Option<i64>) -> Result<(), GameError> {
        Ok(())
    }
    fn start(&mut self) -> Result<(), GameError> {
        //self.play()?;
        Ok(())
    }
    fn handle_event(&mut self, _keycode: Keycode, _time: Option<i64>) {}
}
