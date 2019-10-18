use crate::{
    difficulty_calc,
    notedata::{self, NoteData},
    sprite_finder,
    timingdata::{CalcInfo, TimingData},
};
use bincode::deserialize;
use rayon::{iter::ParallelIterator, join, prelude::*};
use std::{
    fs::{read_dir, File},
    io::Read,
    path::{Path, PathBuf},
    sync::mpsc::{sync_channel, SyncSender},
};

pub fn load_song<T>(sim: T) -> Option<(f64, NoteData)>
where
    T: AsRef<Path> + Clone,
{
    if let Some(extension) = sim.as_ref().extension() {
        let mut sim = match File::open(sim.clone()) {
            Ok(file) => file,
            Err(_) => return None,
        };
        match extension.to_str() {
            Some("sm") => notedata::NoteData::from_sm(sim).ok(),
            Some("rm") => {
                let mut n = vec![];
                sim.read_to_end(&mut n)
                    .expect("Failed to read to end of .rm file");
                deserialize(&n).ok()
            }
            _ => None,
        }
    } else {
        None
    }
    .map(|x| {
        if let Some(timing) = TimingData::<CalcInfo>::from_notedata(&x, sprite_finder, 1.0).get(0) {
            (difficulty_calc::rate_chart(&timing, 1.86), x)
        } else {
            (0.0, x)
        }
    })
}

pub fn load_songs_folder<T>(songs_directory: T) -> Vec<(PathBuf, (f64, NoteData))>
where
    T: AsRef<Path> + Send + Sync,
{
    let (sender, receiver) = sync_channel(2);
    let (_, out) = join(
        || send_songs(songs_directory.as_ref(), sender),
        || receiver.into_iter().collect::<Vec<_>>(),
    );
    out
}

pub fn send_songs(songs_folder: &Path, sender: SyncSender<(PathBuf, (f64, NoteData))>) {
    read_dir(songs_folder)
        .expect("Failed to open folder")
        .collect::<Vec<_>>()
        .into_par_iter()
        .for_each_with(sender, |s, x| {
            if let Ok(entry) = x {
                if entry.path().is_dir() {
                    send_songs(&entry.path(), s.clone())
                } else if let Some(song) = load_song(entry.path()) {
                    let _ = s.send((entry.path(), song));
                }
            }
        })
}
