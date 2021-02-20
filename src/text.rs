use crate::screen::{Element, Message, Resource};
use ggez::{
    event::KeyCode,
    graphics::{self, DrawParam},
    Context, GameError,
};
use std::time::Instant;

pub struct TextBox {
    image: graphics::Text,
    contents: String,
    position: [f32; 2],
    _size: u32,
}

impl TextBox {
    pub fn new(contents: String, position: [f32; 2], size: u32) -> Self {
        Self {
            image: graphics::Text::new(contents.clone()),
            contents,
            position,
            _size: size,
        }
    }
}

impl Element for TextBox {
    fn run(&mut self, context: &mut Context, _time: Option<i64>) -> Result<Message, GameError> {
        println!("{}", self.contents);
        Ok(Message::None)
    }
    fn start(&mut self, _time: Option<Instant>) -> Result<Message, GameError> {
        Ok(Message::None)
    }
    fn finish(&mut self) -> Option<Resource> {
        None
    }
    fn handle_event(&mut self, _key: KeyCode, _time: Option<i64>, _key_down: bool) {}
    fn methods(&mut self, resource: Option<Resource>, index: usize) -> Option<Resource> {
        match index {
            0 => {
                if let Some(Resource::String(contents)) = resource {
                    self.contents = contents;
                    self.image = graphics::Text::new(self.contents.clone());
                }
                None
            }
            1 => {
                if let Some(Resource::Integer(x)) = resource {
                    self.position[0] = x as f32;
                } else if let Some(Resource::Float(x)) = resource {
                    self.position[0] = x as f32;
                }
                None
            }
            2 => {
                if let Some(Resource::Integer(y)) = resource {
                    self.position[1] = y as f32;
                } else if let Some(Resource::Float(y)) = resource {
                    self.position[1] = y as f32;
                }
                None
            }
            _ => None,
        }
    }
}
