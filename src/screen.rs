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
use notedata::NoteData;
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
    NOTEFIELD(usize, usize, usize),
    TEXT(usize, usize, usize),
}

#[derive(Clone)]
pub enum Resource {
    _Notes(TimingData<GameplayInfo>),
    _Path(PathBuf),
    _Layout(Box<NoteLayout>),
    _Float(f64),
    Integer(i64),
    String(String),
    Replay(Vec<TimingColumn<Judgement>>),
    _Multiple(Vec<Resource>),
}

#[derive(Copy, Clone)]
pub enum ResourceType {
    _Notes,
    _Path,
    _Layout,
    _Float,
    _Integer,
    String,
    Replay,
    _Multiple,
}

pub type ResourceCallback = fn(Option<Resource>, &Globals) -> Option<Resource>;

#[derive(Clone)]
pub struct Resources {
    notes: Vec<TimingData<GameplayInfo>>,
    paths: Vec<PathBuf>,
    layouts: Vec<NoteLayout>,
    floats: Vec<f64>,
    integers: Vec<i64>,
    strings: Vec<String>,
    replays: Vec<Vec<TimingColumn<Judgement>>>,
    multiples: Vec<Vec<Resource>>,
}

#[derive(Copy, Clone)]
pub struct ScriptMap {
    pub resource_type: ResourceType,
    pub resource_index: usize,
    pub script_index: usize,
    pub destination_type: ResourceType,
    pub destination_index: usize,
}

#[derive(Copy, Clone)]
pub struct ElementMap {
    pub resource_index: usize,
    pub element_index: usize,
}

#[derive(Copy, Clone)]
pub enum ResourceMap {
    Script(ScriptMap),
    Element(ElementMap),
}

pub type ResourceMaps = Vec<ResourceMap>;

#[derive(Deserialize, Serialize)]
pub struct ScreenBuilder {
    pub elements: Vec<ElementType>,
}

pub struct Screen {
    start_time: Option<Instant>,
    elements: Vec<Box<dyn Element>>,
    resource_maps: ResourceMaps,
}

pub enum Message {
    None,
    Finish,
}

pub struct Globals {
    pub cache: Vec<(PathBuf, (f64, NoteData))>,
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
            Self::NOTEFIELD(layout, timing_data, draw_distance) => Box::new(Notefield::new(
                resources.layouts[*layout].clone(),
                &resources.notes[*timing_data],
                resources.integers[*draw_distance],
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
            multiples: vec![],
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        notes: Vec<TimingData<GameplayInfo>>,
        paths: Vec<PathBuf>,
        layouts: Vec<NoteLayout>,
        floats: Vec<f64>,
        integers: Vec<i64>,
        strings: Vec<String>,
        replays: Vec<Vec<TimingColumn<Judgement>>>,
        multiples: Vec<Vec<Resource>>,
    ) -> Self {
        Self {
            notes,
            paths,
            layouts,
            floats,
            integers,
            strings,
            replays,
            multiples,
        }
    }
    pub fn push(&mut self, resource: Resource) {
        match resource {
            Resource::_Notes(notes) => self.notes.push(notes),
            Resource::_Path(path) => self.paths.push(path),
            Resource::_Layout(layout) => self.layouts.push(*layout),
            Resource::_Float(f) => self.floats.push(f),
            Resource::Integer(int) => self.integers.push(int),
            Resource::String(string) => self.strings.push(string),
            Resource::Replay(replay) => self.replays.push(replay),
            Resource::_Multiple(list) => list.into_iter().for_each(|resource| self.push(resource)),
        }
    }
    pub fn get(&self, index: usize, resource_type: ResourceType) -> Resource {
        match resource_type {
            ResourceType::_Notes => Resource::_Notes(self.notes[index].clone()),
            ResourceType::_Path => Resource::_Path(self.paths[index].clone()),
            ResourceType::_Layout => Resource::_Layout(Box::new(self.layouts[index].clone())),
            ResourceType::_Float => Resource::_Float(self.floats[index]),
            ResourceType::_Integer => Resource::Integer(self.integers[index]),
            ResourceType::String => Resource::String(self.strings[index].clone()),
            ResourceType::Replay => Resource::Replay(self.replays[index].clone()),
            ResourceType::_Multiple => Resource::_Multiple(self.multiples[index].clone()),
        }
    }
    pub fn set(&mut self, index: usize, value: Resource) -> Option<()> {
        match value {
            Resource::_Notes(val) => {
                *self.notes.get_mut(index)? = val;
                Some(())
            }
            Resource::_Path(val) => {
                *self.paths.get_mut(index)? = val;
                Some(())
            }
            Resource::_Layout(val) => {
                *self.layouts.get_mut(index)? = *val;
                Some(())
            }
            Resource::_Float(val) => {
                *self.floats.get_mut(index)? = val;
                Some(())
            }
            Resource::Integer(val) => {
                *self.integers.get_mut(index)? = val;
                Some(())
            }
            Resource::String(val) => {
                *self.strings.get_mut(index)? = val;
                Some(())
            }
            Resource::Replay(val) => {
                *self.replays.get_mut(index)? = val;
                Some(())
            }
            Resource::_Multiple(val) => {
                *self.multiples.get_mut(index)? = val;
                Some(())
            }
        }
    }
}

impl ScreenBuilder {
    pub fn build(&self, resources: &Resources, scripts: ResourceMaps) -> Screen {
        let element_list = self
            .elements
            .iter()
            .map(|element| element.build(resources))
            .collect();
        Screen::new(element_list, scripts)
    }
}

impl Screen {
    pub fn new(elements: Vec<Box<dyn Element>>, scripts: ResourceMaps) -> Self {
        Self {
            start_time: Some(Instant::now() + Duration::from_secs(3)),
            elements,
            resource_maps: scripts,
        }
    }
    pub fn start(&mut self) -> Result<(), GameError> {
        for element in &mut self.elements {
            element.start(self.start_time)?;
        }
        Ok(())
    }
    pub fn finish(
        &mut self,
        resources: &mut Resources,
        callbacks: &[ResourceCallback],
        globals: &Globals,
    ) {
        for map in &self.resource_maps {
            match map {
                ResourceMap::Element(ElementMap {
                    resource_index,
                    element_index,
                }) => {
                    if let Some(resource) = self.elements[*element_index].finish() {
                        if resources.set(*resource_index, resource.clone()).is_none() {
                            resources.push(resource)
                        }
                    }
                }
                ResourceMap::Script(ScriptMap {
                    resource_type,
                    resource_index,
                    script_index,
                    destination_type: _destination_type,
                    destination_index,
                }) => {
                    if let Some(resource) = callbacks[*script_index](
                        Some(resources.get(*resource_index, *resource_type)),
                        globals,
                    ) {
                        resources.set(*destination_index, resource);
                    }
                }
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
    pub fn draw(&mut self, ctx: &mut Context) -> Result<Message, GameError> {
        graphics::clear(ctx, Color::new(0.0, 0.0, 0.0, 1.0));
        let time_delta = self.start_time_to_milliseconds();
        for element in &mut self.elements {
            match element.run(ctx, time_delta)? {
                Message::None => {}
                Message::Finish => return Ok(Message::Finish),
            }
        }
        graphics::present(ctx)?;
        Ok(Message::None)
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
        for element in &mut self.elements {
            element.handle_event(keycode, time_delta, true);
        }
    }
    pub fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods) {
        let time_delta = self.start_time_to_milliseconds();
        for element in &mut self.elements {
            element.handle_event(keycode, time_delta, false);
        }
    }
}
