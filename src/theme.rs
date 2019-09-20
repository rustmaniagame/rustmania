use crate::timingdata::{Judgement, TimingColumn};
use crate::{
    music::Music,
    notefield::Notefield,
    player_config::NoteLayout,
    screen::{Element, Screen},
    timingdata::{GameplayInfo, TimingData},
};
use serde_derive::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Deserialize, Serialize)]
pub struct ScreenBuilder {
    pub elements: Vec<ElementType>,
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

#[derive(Deserialize, Serialize)]
pub enum ElementType {
    MUSIC(usize, usize),
    NOTEFIELD(usize, usize),
    TEXT(usize, usize, usize),
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

pub type ResourceCallback = fn(Option<Resource>) -> Option<Resource>;

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
