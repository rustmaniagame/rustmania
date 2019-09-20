use crate::{
    screen::{Message, Screen},
    theme::{Resource, ResourceCallback, Resources, ScreenBuilder},
};
use ggez::{
    event::{EventHandler, KeyCode, KeyMods},
    Context, GameError,
};

pub struct GameState {
    scene_stack: Vec<ScreenBuilder>,
    current_screen: Option<Screen>,
    screen_index: usize,
    resources: Resources,
    callbacks: Vec<fn(Option<Resource>) -> Option<Resource>>,
}

impl GameState {
    pub fn _new() -> Self {
        Self {
            scene_stack: Vec::new(),
            current_screen: None,
            screen_index: 0,
            resources: Resources::_new(),
            callbacks: vec![],
        }
    }
    pub fn from(
        scene_stack: Vec<ScreenBuilder>,
        resources: Resources,
        callbacks: Vec<ResourceCallback>,
    ) -> Self {
        Self {
            scene_stack,
            current_screen: None,
            screen_index: 0,
            resources,
            callbacks,
        }
    }
}

impl EventHandler for GameState {
    fn update(&mut self, _ctx: &mut Context) -> Result<(), GameError> {
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        if let Some(ref mut screen) = self.current_screen {
            match screen.draw(ctx)? {
                Message::Normal => {}
                Message::Finish => {
                    self.screen_index += 1;
                    screen.finish(&mut self.resources, &self.callbacks);
                    self.current_screen = None;
                }
            };
        } else {
            self.current_screen = self
                .scene_stack
                .get(self.screen_index)
                .map(|screen| screen.build(&self.resources));
            if let Some(ref mut screen) = self.current_screen {
                screen.start()?;
            }
        }
        Ok(())
    }
    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        keymod: KeyMods,
        repeat: bool,
    ) {
        if let Some(ref mut screen) = self.current_screen {
            screen.key_down_event(ctx, keycode, keymod, repeat)
        };
    }
    fn key_up_event(&mut self, ctx: &mut Context, keycode: KeyCode, keymod: KeyMods) {
        if let Some(ref mut screen) = self.current_screen {
            screen.key_up_event(ctx, keycode, keymod)
        };
    }
}
