use rayon::{iter::ParallelIterator, prelude::*};
use std::{
    fs::read_dir,
    path::{Path, PathBuf},
    sync::mpsc::{sync_channel, SyncSender},
    thread,
};

pub enum LoadError {
    WrongExtension,
    FailedParse,
}

pub fn load_songs_folder<T, U: 'static>(
    songs_directory: T,
    load_song: impl Fn(&PathBuf) -> Result<U, LoadError> + Clone + Send + Sync,
) -> Vec<Option<(PathBuf, U)>>
where
    T: AsRef<Path> + Send + Sync,
    U: Send + Sync,
{
    let (sender, receiver) = sync_channel(2);
    let out = thread::spawn(|| receiver.into_iter().collect::<Vec<_>>());
    send_songs(songs_directory.as_ref(), sender, load_song);
    out.join().expect("Failed to collect songs")
}

pub fn send_songs<U>(
    songs_folder: &Path,
    sender: SyncSender<Option<(PathBuf, U)>>,
    load_song: impl Fn(&PathBuf) -> Result<U, LoadError> + Clone + Send + Sync,
) where
    U: Send + Sync,
{
    read_dir(songs_folder)
        .expect("Failed to open folder")
        .collect::<Vec<_>>()
        .into_par_iter()
        .for_each_with(sender, |s, x| {
            if let Ok(entry) = x {
                if entry.path().is_dir() {
                    send_songs(&entry.path(), s.clone(), load_song.clone())
                } else {
                    match load_song(&entry.path()) {
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
