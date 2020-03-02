#![allow(clippy::needless_pass_by_value)]
use crate::{
    load_song,
    screen::{Globals, Resource},
    timingdata::TimingColumn,
};
use std::{convert::TryFrom, path::PathBuf};

pub fn map_to_string(resource: Option<Resource>, _globals: &Globals) -> Option<Resource> {
    resource.map(|resource| match resource {
        Resource::Replay(replay) => Resource::String(
            (replay
                .iter()
                .map(|column| column.current_points(1.0))
                .sum::<f64>()
                / replay.iter().map(TimingColumn::max_points).sum::<f64>()
                * 100.0)
                .to_string(),
        ),
        Resource::Float(val) => Resource::String(format!("{}", val)),
        _ => Resource::String("".to_owned()),
    })
}

pub fn song_title(resource: Option<Resource>, globals: &Globals) -> Option<Resource> {
    if let Some(Resource::Integer(index)) = resource {
        let index = match usize::try_from(index) {
            Ok(number) => number,
            Err(_) => return None,
        };
        Some(Resource::String(
            globals.cache.get(index).map_or_else(String::new, |entry| {
                entry.data.title.clone().unwrap_or_else(String::new)
            }),
        ))
    } else {
        None
    }
}

pub fn print_resource(resource: Option<Resource>, _globals: &Globals) -> Option<Resource> {
    println!("{:?}", resource);
    None
}

pub fn add_one(resource: Option<Resource>, _globals: &Globals) -> Option<Resource> {
    resource.map(|x| {
        if let Resource::Integer(x) = x {
            Resource::Integer(x + 1)
        } else {
            x
        }
    })
}

pub fn subtract_one(resource: Option<Resource>, _globals: &Globals) -> Option<Resource> {
    resource.map(|x| {
        if let Resource::Integer(x) = x {
            Resource::Integer(x - 1)
        } else {
            x
        }
    })
}

pub fn song_path(resource: Option<Resource>, globals: &Globals) -> Option<Resource> {
    if let Some(Resource::Integer(index)) = resource {
        globals
            .cache
            .get(usize::try_from(index).ok()?)
            .map(|entry| Resource::_Path(entry.path.clone()))
    } else {
        None
    }
}

pub fn song_from_path(resource: Option<Resource>, globals: &Globals) -> Option<Resource> {
    if let Some(Resource::_Path(path)) = resource {
        Some(Resource::_Notes(
            super::timingdata::TimingData::from_notedata(
                &load_song(&path).ok().map(|(_, data)| data)?,
                super::sprite_finder,
                globals.song_options.rate,
            )
            .get(0)?
            .clone(),
        ))
    } else {
        None
    }
}

pub fn music_path(resource: Option<Resource>, globals: &Globals) -> Option<Resource> {
    if let Some(Resource::Integer(index)) = resource {
        let index = usize::try_from(index).ok()?;
        Some(Resource::_Path(globals.cache.get(index).map_or_else(
            PathBuf::new,
            |entry| {
                entry
                    .data
                    .music_path
                    .clone()
                    .map_or_else(PathBuf::new, |x| {
                        PathBuf::from(format!(
                            "{}/{}",
                            String::from(
                                entry
                                    .path
                                    .parent()
                                    .expect("No parent folder for selected file")
                                    .as_os_str()
                                    .to_str()
                                    .expect("failed to parse path")
                            ),
                            x
                        ))
                    })
            },
        )))
    } else {
        None
    }
}
