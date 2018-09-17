use screen::Screen;

pub struct GameState<'a> {
    scene_stack: Vec<Screen<'a>>
}