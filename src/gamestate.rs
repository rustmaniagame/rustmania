use crate::screen::{Message, Screen};
use ggez::{
    event::{EventHandler, KeyCode, KeyMods},
    Context, GameError,
};

pub struct GameState {
    scene_stack: Vec<Screen>,
    current_screen: usize,
}

impl GameState {
    pub fn _new() -> Self {
        GameState {
            scene_stack: Vec::new(),
            current_screen: 0,
        }
    }
    pub fn from(scene_stack: Vec<Screen>) -> Self {
        Self {
            scene_stack,
            current_screen: 0,
        }
    }
}

impl EventHandler for GameState {
    fn update(&mut self, _ctx: &mut Context) -> Result<(), GameError> {
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        match self.scene_stack[self.current_screen].draw(ctx)? {
            Message::Normal => {}
            Message::Finish => {
                self.current_screen += 1;
            }
        };
        Ok(())
    }
    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        keymod: KeyMods,
        repeat: bool,
    ) {
        self.scene_stack[self.current_screen].key_down_event(ctx, keycode, keymod, repeat);
    }
    fn key_up_event(&mut self, ctx: &mut Context, keycode: KeyCode, keymod: KeyMods) {
        self.scene_stack[self.current_screen].key_up_event(ctx, keycode, keymod);
    }
}
