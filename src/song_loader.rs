use crate::{
    difficulty_calc, sprite_finder,
    timingdata::{CalcInfo, TimingData},
};
use bincode::deserialize;
use notedata::{self, NoteData};
use rayon::{iter::ParallelIterator, prelude::*};
use std::{
    fs::{read_dir, File},
    io::Read,
    path::{Path, PathBuf},
    sync::mpsc::{sync_channel, SyncSender},
    thread,
};

pub enum LoadError {
    WrongExtension,
    FailedParse,
}

pub fn load_song<T>(sim: T) -> Result<(f64, NoteData), LoadError>
where
    T: AsRef<Path> + Clone,
{
    if let Some(extension) = sim.as_ref().extension() {
        let mut sim = match File::open(sim.clone()) {
            Ok(file) => file,
            Err(_) => return Err(LoadError::FailedParse),
        };
        match extension.to_str() {
            Some("sm") => notedata::NoteData::from_sm(sim).map_err(|_| LoadError::FailedParse),
            Some("rm") => {
                let mut n = vec![];
                sim.read_to_end(&mut n)
                    .expect("Failed to read to end of .rm file");
                deserialize(&n).map_err(|_| LoadError::FailedParse)
            }
            _ => Err(LoadError::WrongExtension),
        }
    } else {
        Err(LoadError::WrongExtension)
    }
    .map(|x| {
        if let Some(timing) = TimingData::<CalcInfo>::from_notedata(&x, sprite_finder, 1.0).get(0) {
            (difficulty_calc::rate_chart(&timing, 1.86), x)
        } else {
            (0.0, x)
        }
    })
}

pub fn load_songs_folder<T>(songs_directory: T) -> Vec<Option<(PathBuf, (f64, NoteData))>>
where
    T: AsRef<Path> + Send + Sync,
{
    let (sender, receiver) = sync_channel(2);
    let out = thread::spawn(|| receiver.into_iter().collect::<Vec<_>>());
    send_songs(songs_directory.as_ref(), sender);
    out.join().expect("Failed to collect songs")
}

pub fn send_songs(songs_folder: &Path, sender: SyncSender<Option<(PathBuf, (f64, NoteData))>>) {
    read_dir(songs_folder)
        .expect("Failed to open folder")
        .collect::<Vec<_>>()
        .into_par_iter()
        .for_each_with(sender, |s, x| {
            if let Ok(entry) = x {
                if entry.path().is_dir() {
                    send_songs(&entry.path(), s.clone())
                } else {
                    match load_song(entry.path()) {
                        Ok(song) => {
                            let _ = s.send(Some((entry.path(), song)));
                        }
                        Err(err) => match err {
                            LoadError::WrongExtension => {}
                            LoadError::FailedParse => {
                                s.send(None).expect("Failed to send song along channel")
                            }
                        },
                    }
                }
            }
        })
}
