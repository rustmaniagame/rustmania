use crate::notedata;
use crate::notedata::NoteData;
use std::ffi::OsStr;
use std::fs::{read_dir, File};
use std::path::Path;

pub fn load_song<T>(simfile_folder: T) -> Option<NoteData>
where
    T: AsRef<Path>,
{
    read_dir(simfile_folder)
        .expect("Couldn't open folder")
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension() == Some(OsStr::new("sm")))
        .filter_map(|sim| File::open(sim.path()).ok())
        .find_map(|sim| notedata::NoteData::from_sm(sim).ok())
}
