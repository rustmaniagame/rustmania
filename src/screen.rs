use ggez::{
    event::{EventHandler, KeyCode, KeyMods},
    graphics::{self, Color},
    Context, GameError,
};
use std::time::{Duration, Instant};

pub trait Element: Send {
    fn run(&mut self, context: &mut Context, time: Option<i64>) -> Result<(), GameError>;
    fn start(&mut self, time: Option<Instant>) -> Result<(), GameError>;
    fn handle_event(&mut self, key: KeyCode, time: Option<i64>, key_down: bool);
}

pub struct Screen<'a> {
    start_time: Option<Instant>,
    elements: Vec<Box<dyn Element + 'a>>,
    _key_handler: (),
}

fn to_milliseconds(dur: Duration) -> i64 {
    dur.as_secs() as i64 * 1000 + i64::from(dur.subsec_millis())
}

impl<'a> Screen<'a> {
    pub fn new(elements: Vec<Box<dyn Element + 'a>>) -> Self {
        Screen {
            start_time: Some(Instant::now() + Duration::from_secs(3)),
            elements,
            _key_handler: (),
        }
    }
    pub fn start(&mut self) -> Result<(), GameError> {
        for element in &mut self.elements {
            element.start(self.start_time)?;
        }
        Ok(())
    }
    fn start_time_to_milliseconds(&self) -> Option<i64> {
        match self.start_time {
            Some(time) => {
                let now = Instant::now();
                if time > now {
                    Some(-to_milliseconds(time.duration_since(now)))
                } else {
                    Some(to_milliseconds(now.duration_since(time)))
                }
            }
            None => None,
        }
    }
}

impl<'a> EventHandler for Screen<'a> {
    fn update(&mut self, _ctx: &mut Context) -> Result<(), GameError> {
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        graphics::clear(ctx, Color::new(0.0, 0.0, 0.0, 1.0));
        let time_delta = self.start_time_to_milliseconds();
        for element in &mut self.elements {
            element.run(ctx, time_delta)?;
        }
        graphics::present(ctx)?;
        Ok(())
    }
    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymod: KeyMods,
        repeat: bool,
    ) {
        if repeat {
            return;
        }
        let time_delta = self.start_time_to_milliseconds();
        for element in &mut self.elements {
            element.handle_event(keycode, time_delta, true);
        }
    }
    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods) {
        let time_delta = self.start_time_to_milliseconds();
        for element in &mut self.elements {
            element.handle_event(keycode, time_delta, false);
        }
    }
}
