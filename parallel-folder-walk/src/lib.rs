#![warn(
    clippy::cast_lossless,
    clippy::checked_conversions,
    clippy::copy_iterator,
    clippy::default_trait_access,
    clippy::doc_markdown,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::expl_impl_clone_on_copy,
    clippy::explicit_into_iter_loop,
    clippy::explicit_iter_loop,
    clippy::filter_map,
    clippy::filter_map_next,
    clippy::find_map,
    clippy::if_not_else,
    clippy::inline_always,
    clippy::items_after_statements,
    clippy::large_digit_groups,
    clippy::large_stack_arrays,
    clippy::map_flatten,
    clippy::match_same_arms,
    clippy::maybe_infinite_iter,
    clippy::mut_mut,
    clippy::needless_continue,
    clippy::needless_pass_by_value,
    clippy::non_ascii_literal,
    clippy::option_map_unwrap_or,
    clippy::option_map_unwrap_or_else,
    clippy::pub_enum_variant_names,
    clippy::redundant_closure_for_method_calls,
    clippy::replace_consts,
    clippy::result_map_unwrap_or_else,
    clippy::same_functions_in_if_condition,
    clippy::shadow_unrelated,
    clippy::similar_names,
    clippy::single_match_else,
    clippy::too_many_lines,
    clippy::type_repetition_in_bounds,
    clippy::unicode_not_nfc,
    clippy::unseparated_literal_suffix,
    clippy::unused_self,
    clippy::used_underscore_binding
)]

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
        .par_bridge()
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
