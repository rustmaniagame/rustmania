use screen::{Screen,Element};

pub struct GameState<'a> {
    scene_stack: Vec<Screen<'a>>,
    loose_elements: Vec<Box<dyn Element + 'a>>,
}