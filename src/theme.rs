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
    MUSIC(f64, usize),
    NOTEFIELD(usize, usize),
}

impl ElementType {
    pub fn build<'a>(&self, resources: &'a Resources) -> Box<dyn Element + 'a> {
        match self {
            ElementType::MUSIC(rate, name) => Box::new(Music::new(
                *rate,
                resources.paths[*name]
                    .as_os_str()
                    .to_string_lossy()
                    .to_string()
                    .clone(),
            )),
            ElementType::NOTEFIELD(layout, timing_data) => Box::new(Notefield::new(
                &resources.layouts[*layout],
                &resources.notes[*timing_data],
                600,
            )),
        }
    }
}

pub struct Resources {
    notes: Vec<TimingData<GameplayInfo>>,
    paths: Vec<PathBuf>,
    layouts: Vec<NoteLayout>,
}

impl Resources {
    pub fn from(
        notes: Vec<TimingData<GameplayInfo>>,
        paths: Vec<PathBuf>,
        layouts: Vec<NoteLayout>,
    ) -> Self {
        Resources {
            notes,
            paths,
            layouts,
        }
    }
}
