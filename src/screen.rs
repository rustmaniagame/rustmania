use ggez::{
    event::{KeyCode, KeyMods},
    graphics::{self, Color},
    Context, GameError,
};
use std::time::{Duration, Instant};

pub trait Element: Send {
    fn run(&mut self, context: &mut Context, time: Option<i64>) -> Result<Message, GameError>;
    fn start(&mut self, time: Option<Instant>) -> Result<Message, GameError>;
    fn handle_event(&mut self, key: KeyCode, time: Option<i64>, key_down: bool);
}

pub struct Screen {
    start_time: Option<Instant>,
    elements: Vec<Box<dyn Element>>,
    _key_handler: (),
}

pub enum Message {
    Normal,
    Finish,
}

fn to_milliseconds(dur: Duration) -> i64 {
    dur.as_secs() as i64 * 1000 + i64::from(dur.subsec_millis())
}

impl Screen {
    pub fn new(elements: Vec<Box<dyn Element>>) -> Self {
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

impl Screen {
    fn _update(&mut self, _ctx: &mut Context) -> Result<Message, GameError> {
        Ok(Message::Normal)
    }
    pub fn draw(&mut self, ctx: &mut Context) -> Result<Message, GameError> {
        graphics::clear(ctx, Color::new(0.0, 0.0, 0.0, 1.0));
        let time_delta = self.start_time_to_milliseconds();
        for element in &mut self.elements {
            match element.run(ctx, time_delta)? {
                Message::Normal => {}
                Message::Finish => return Ok(Message::Finish),
            }
        }
        graphics::present(ctx)?;
        Ok(Message::Normal)
    }
    pub fn key_down_event(
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
    pub fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods) {
        let time_delta = self.start_time_to_milliseconds();
        for element in &mut self.elements {
            element.handle_event(keycode, time_delta, false);
        }
    }
}
