use crate::{
    screen::{Globals, Message, ResourceCallback, Resources, Screen, ScreenBuilder, ScriptList},
    SongOptions,
};
use ggez::{
    event::{EventHandler, KeyCode, KeyMods},
    Context, GameError,
};
use std::collections::HashMap;

pub struct GameState {
    scene_stack: HashMap<String, ScreenBuilder>,
    current_screen: Option<Screen>,
    screen_index: String,
    resources: Resources,
    callbacks: Vec<ResourceCallback>,
    globals: Globals,
    scripts: ScriptList,
}

impl GameState {
    pub fn _new() -> Self {
        Self {
            scene_stack: HashMap::new(),
            current_screen: None,
            screen_index: "start".to_string(),
            resources: Resources::_new(),
            callbacks: vec![],
            globals: Globals {
                cache: vec![],
                song_options: SongOptions::default(),
            },
            scripts: ScriptList {
                scripts: HashMap::new(),
            },
        }
    }
    pub fn new(
        scene_stack: HashMap<String, ScreenBuilder>,
        resources: Resources,
        callbacks: Vec<ResourceCallback>,
        globals: Globals,
        scripts: ScriptList,
    ) -> Self {
        Self {
            scene_stack,
            current_screen: None,
            screen_index: "start".to_string(),
            resources,
            callbacks,
            globals,
            scripts,
        }
    }
}

impl EventHandler for GameState {
    fn update(&mut self, _ctx: &mut Context) -> Result<(), GameError> {
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        if let Some(ref mut screen) = self.current_screen {
            match &screen.current_message {
                Message::None => {}
                Message::Finish(destination) => {
                    self.screen_index = destination.clone();
                    screen.finish(
                        &mut self.resources,
                        &self.callbacks,
                        &self.globals,
                        &self.scripts,
                    );
                    self.current_screen = None;
                }
            };
        }
        if let Some(ref mut screen) = self.current_screen {
            match screen.draw(ctx)? {
                Message::None => {}
                Message::Finish(destination) => {
                    self.screen_index = destination;
                    screen.finish(
                        &mut self.resources,
                        &self.callbacks,
                        &self.globals,
                        &self.scripts,
                    );
                    self.current_screen = None;
                }
            };
        } else {
            self.current_screen = self
                .scene_stack
                .get(&self.screen_index)
                .map(|screen| screen.build(&self.resources));
            if let Some(ref mut screen) = self.current_screen {
                screen.start()?;
            }
        }
        Ok(())
    }
    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymod: KeyMods,
        repeat: bool,
    ) {
        if let Some(ref mut screen) = self.current_screen {
            if !repeat {
                screen.key_down_event(
                    keycode,
                    &mut self.resources,
                    &self.callbacks,
                    &self.globals,
                    &self.scripts,
                )
            }
        };
    }
    fn key_up_event(&mut self, ctx: &mut Context, keycode: KeyCode, keymod: KeyMods) {
        if let Some(ref mut screen) = self.current_screen {
            screen.key_up_event(ctx, keycode, keymod)
        };
    }
}
