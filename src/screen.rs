use crate::{
    music::Music,
    notefield::Notefield,
    player_config::NoteLayout,
    timingdata::{GameplayInfo, Judgement, TimingColumn, TimingData},
};
use ggez::{
    event::{KeyCode, KeyMods},
    graphics::{self, Color},
    Context, GameError,
};
use serde_derive::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

pub trait Element: Send {
    fn run(&mut self, context: &mut Context, time: Option<i64>) -> Result<Message, GameError>;
    fn start(&mut self, time: Option<Instant>) -> Result<Message, GameError>;
    fn finish(&mut self) -> Option<Resource>;
    fn handle_event(&mut self, key: KeyCode, time: Option<i64>, key_down: bool);
}

#[derive(Deserialize, Serialize)]
pub enum ElementType {
    MUSIC(usize, usize),
    NOTEFIELD(usize, usize),
    TEXT(usize, usize, usize),
}

pub enum Resource {
    _Notes(TimingData<GameplayInfo>),
    _Path(PathBuf),
    _Layout(Box<NoteLayout>),
    _Float(f64),
    _Integer(i64),
    String(String),
    Replay(Vec<TimingColumn<Judgement>>),
    _Multiple(Vec<Resource>),
}

pub type ResourceCallback = fn(Option<Resource>) -> Option<Resource>;

pub struct Resources {
    notes: Vec<TimingData<GameplayInfo>>,
    paths: Vec<PathBuf>,
    layouts: Vec<NoteLayout>,
    floats: Vec<f64>,
    #[allow(clippy::used_underscore_binding)]
    integers: Vec<i64>,
    strings: Vec<String>,
    replays: Vec<Vec<TimingColumn<Judgement>>>,
}

#[derive(Deserialize, Serialize)]
pub struct ScreenBuilder {
    pub elements: Vec<ElementType>,
}

pub struct Screen {
    start_time: Option<Instant>,
    elements: Vec<(Box<dyn Element>, usize)>,
    _key_handler: (),
}

pub enum Message {
    Normal,
    Finish,
}

fn to_milliseconds(dur: Duration) -> i64 {
    dur.as_secs() as i64 * 1000 + i64::from(dur.subsec_millis())
}

impl ElementType {
    pub fn build(&self, resources: &Resources) -> Box<dyn Element> {
        match self {
            Self::MUSIC(rate, name) => Box::new(Music::new(
                resources.floats[*rate],
                resources.paths[*name].clone(),
            )),
            Self::NOTEFIELD(layout, timing_data) => Box::new(Notefield::new(
                resources.layouts[*layout].clone(),
                &resources.notes[*timing_data],
                600,
            )),
            Self::TEXT(contents, x_pos, y_pos) => Box::new(crate::text::TextBox::new(
                resources.strings[*contents].clone(),
                [
                    resources.floats[*x_pos] as f32,
                    resources.floats[*y_pos] as f32,
                ],
                0,
            )),
        }
    }
}

impl Resources {
    pub fn _new() -> Self {
        Self {
            notes: vec![],
            paths: vec![],
            layouts: vec![],
            floats: vec![],
            integers: vec![],
            strings: vec![],
            replays: vec![],
        }
    }
    pub fn from(
        notes: Vec<TimingData<GameplayInfo>>,
        paths: Vec<PathBuf>,
        layouts: Vec<NoteLayout>,
        floats: Vec<f64>,
        integers: Vec<i64>,
        strings: Vec<String>,
        replays: Vec<Vec<TimingColumn<Judgement>>>,
    ) -> Self {
        Self {
            notes,
            paths,
            layouts,
            floats,
            integers,
            strings,
            replays,
        }
    }
    pub fn push(&mut self, resource: Resource) {
        match resource {
            Resource::_Notes(notes) => self.notes.push(notes),
            Resource::_Path(path) => self.paths.push(path),
            Resource::_Layout(layout) => self.layouts.push(*layout),
            Resource::_Float(f) => self.floats.push(f),
            Resource::_Integer(int) => self.integers.push(int),
            Resource::String(string) => self.strings.push(string),
            Resource::Replay(replay) => self.replays.push(replay),
            Resource::_Multiple(list) => list.into_iter().for_each(|resource| self.push(resource)),
        }
    }
}

impl ScreenBuilder {
    pub fn build(&self, resources: &Resources) -> Screen {
        let element_list = self
            .elements
            .iter()
            .map(|element| (element.build(resources), 0))
            .collect();
        Screen::new(element_list)
    }
}

impl Screen {
    pub fn new(elements: Vec<(Box<dyn Element>, usize)>) -> Self {
        Self {
            start_time: Some(Instant::now() + Duration::from_secs(3)),
            elements,
            _key_handler: (),
        }
    }
    pub fn start(&mut self) -> Result<(), GameError> {
        for (element, _callback_index) in &mut self.elements {
            element.start(self.start_time)?;
        }
        Ok(())
    }
    pub fn finish(&mut self, resources: &mut Resources, callbacks: &[ResourceCallback]) {
        for (elem, callback_index) in &mut self.elements {
            if let Some(resource) = callbacks[*callback_index](elem.finish()) {
                resources.push(resource);
            }
        }
    }
    fn start_time_to_milliseconds(&self) -> Option<i64> {
        match self.start_time {
            Some(time) => {
                let now = Instant::now();
                if time > now {
                    Some(-to_milliseconds(time.duration_since(now)))
                } else {
                    Some(to_milliseconds(now.duration_since(time)))
                }
            }
            None => None,
        }
    }
}

impl Screen {
    fn _update(&mut self, _ctx: &mut Context) -> Result<Message, GameError> {
        Ok(Message::Normal)
    }
    pub fn draw(&mut self, ctx: &mut Context) -> Result<Message, GameError> {
        graphics::clear(ctx, Color::new(0.0, 0.0, 0.0, 1.0));
        let time_delta = self.start_time_to_milliseconds();
        for (element, _callback_index) in &mut self.elements {
            match element.run(ctx, time_delta)? {
                Message::Normal => {}
                Message::Finish => return Ok(Message::Finish),
            }
        }
        graphics::present(ctx)?;
        Ok(Message::Normal)
    }
    pub fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymod: KeyMods,
        repeat: bool,
    ) {
        if repeat {
            return;
        }
        let time_delta = self.start_time_to_milliseconds();
        for (element, _callback_index) in &mut self.elements {
            element.handle_event(keycode, time_delta, true);
        }
    }
    pub fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods) {
        let time_delta = self.start_time_to_milliseconds();
        for (element, _callback_index) in &mut self.elements {
            element.handle_event(keycode, time_delta, false);
        }
    }
}
