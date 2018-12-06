use rlua::UserData;
use crate::screen::{Element, Screen};

pub struct GameState<'a> {
    scene_stack: Vec<Screen<'a>>,
    loose_elements: Vec<Box<dyn Element + 'a>>,
}

impl UserData for GameState<'static> {}

impl<'a> GameState<'a> {
    pub fn new() -> Self {
        GameState {
            scene_stack: Vec::new(),
            loose_elements: Vec::new(),
        }
    }
}
