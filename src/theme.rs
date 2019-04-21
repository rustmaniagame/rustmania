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
    pub fn build<'a>(&self, resources: &'a Resources) -> Screen<'a> {
        let element_list = self
            .elements
            .iter()
            .map(|element| element.build(resources))
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
    pub fn build<'a>(&self, resources: &'a Resources) -> Box<dyn Element + 'a> {
        match self {
            ElementType::MUSIC(rate, name) => Box::new(Music::new(
                resources.floats[*rate],
                resources.paths[*name].clone(),
            )),
            ElementType::NOTEFIELD(layout, timing_data) => Box::new(Notefield::new(
                &resources.layouts[*layout],
                &resources.notes[*timing_data],
                600,
            )),
            ElementType::TEXT(contents, x_pos, y_pos) => Box::new(crate::text::TextBox::new(
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

pub struct Resources {
    notes: Vec<TimingData<GameplayInfo>>,
    paths: Vec<PathBuf>,
    layouts: Vec<NoteLayout>,
    floats: Vec<f64>,
    _integers: Vec<i64>,
    strings: Vec<String>,
}

impl Resources {
    pub fn from(
        notes: Vec<TimingData<GameplayInfo>>,
        paths: Vec<PathBuf>,
        layouts: Vec<NoteLayout>,
        floats: Vec<f64>,
        _integers: Vec<i64>,
        strings: Vec<String>,
    ) -> Self {
        Resources {
            notes,
            paths,
            layouts,
            floats,
            _integers,
            strings,
        }
    }
}
