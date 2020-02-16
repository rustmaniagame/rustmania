use crate::screen::{
    Globals, Message, ResourceCallback, ResourceMaps, Resources, Screen, ScreenBuilder,
};
use ggez::{
    event::{EventHandler, KeyCode, KeyMods},
    Context, GameError,
};

pub struct GameState {
    scene_stack: Vec<(ScreenBuilder, ResourceMaps)>,
    current_screen: Option<Screen>,
    screen_index: usize,
    resources: Resources,
    callbacks: Vec<ResourceCallback>,
    globals: Globals,
}

impl GameState {
    pub fn _new() -> Self {
        Self {
            scene_stack: Vec::new(),
            current_screen: None,
            screen_index: 0,
            resources: Resources::_new(),
            callbacks: vec![],
            globals: Globals { cache: vec![] },
        }
    }
    pub fn new(
        scene_stack: Vec<(ScreenBuilder, ResourceMaps)>,
        resources: Resources,
        callbacks: Vec<ResourceCallback>,
        globals: Globals,
    ) -> Self {
        Self {
            scene_stack,
            current_screen: None,
            screen_index: 0,
            resources,
            callbacks,
            globals,
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
                Message::None => {}
                Message::Finish => {
                    self.screen_index += 1;
                    screen.finish(&mut self.resources, &self.callbacks, &self.globals);
                    self.current_screen = None;
                }
            };
        } else {
            self.current_screen = self
                .scene_stack
                .get(self.screen_index)
                .map(|screen| screen.0.build(&self.resources, screen.1.clone()));
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
