use ggez::{graphics, GameError, Context};
use crate::screen::Element;
use std::time::Instant;
use ggez::event::KeyCode;
use ggez::graphics::DrawParam;

pub struct TextBox {
    image: graphics::Text,
    _contents: String,
    _position: (i64, i64),
    _size: u32,
}

impl TextBox {
    pub fn new(contents: String, position: (i64,i64), size: u32) -> Self {
        TextBox {
            image: graphics::Text::new(contents.clone()),
            _contents: contents,
            _position: position,
            _size: size,
        }
    }
}

impl Element for TextBox {
    fn run(&mut self, context: &mut Context, _time: Option<i64>) -> Result<(), GameError> {
        graphics::draw(context, &self.image, DrawParam::new())?;
        Ok(())
    }
    fn start(&mut self, _time: Option<Instant>) -> Result<(), GameError> {Ok(())}
    fn handle_event(&mut self, _key: KeyCode, _time: Option<i64>, _key_down: bool) {}
}