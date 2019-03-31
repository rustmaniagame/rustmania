use crate::screen::{Element, Screen};

pub struct _GameState<'a> {
    _scene_stack: Vec<Screen<'a>>,
    _loose_elements: Vec<Box<dyn Element + 'a>>,
}

impl<'a> _GameState<'a> {
    pub fn _new() -> Self {
        _GameState {
            _scene_stack: Vec::new(),
            _loose_elements: Vec::new(),
        }
    }
}
