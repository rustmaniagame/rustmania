use crate::screen::{Element, Screen};
use rlua::UserData;

pub struct GameState<'a> {
    _scene_stack: Vec<Screen<'a>>,
    _loose_elements: Vec<Box<dyn Element + 'a>>,
}

impl UserData for GameState<'static> {}

impl<'a> GameState<'a> {
    pub fn new() -> Self {
        GameState {
            _scene_stack: Vec::new(),
            _loose_elements: Vec::new(),
        }
    }
}
