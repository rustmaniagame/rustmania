use crate::screen::Element;
use ggez::event::KeyCode;
use ggez::graphics::DrawParam;
use ggez::{graphics, Context, GameError};
use std::time::Instant;

pub struct TextBox {
    image: graphics::Text,
    _contents: String,
    _position: [f32; 2],
    _size: u32,
}

impl TextBox {
    pub fn new(contents: String, position: [f32; 2], size: u32) -> Self {
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
        graphics::draw(context, &self.image, DrawParam::new().dest(self._position))?;
        Ok(())
    }
    fn start(&mut self, _time: Option<Instant>) -> Result<(), GameError> {
        Ok(())
    }
    fn handle_event(&mut self, _key: KeyCode, _time: Option<i64>, _key_down: bool) {}
}
