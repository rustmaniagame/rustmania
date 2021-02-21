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
    clippy::pub_enum_variant_names,
    clippy::redundant_closure_for_method_calls,
    clippy::map_unwrap_or,
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

mod callbacks;
mod difficulty_calc;
mod gamestate;
mod screen;
mod text;

use crate::screen::Theme;
use crate::{
    gamestate::GameState,
    screen::{CacheEntry, Globals, Resources},
};
use bincode::deserialize;
use ggez::{filesystem::mount, ContextBuilder};
use log::{debug, info};
use notedata::{
    timingdata::{CalcInfo, Rectangle, TimingData},
    Fraction, NoteData, NoteType,
};
use std::{
    cmp::Ordering,
    ffi::OsStr,
    fs::{File, OpenOptions},
    io::Read,
    path::PathBuf,
    time::Instant,
};
use structopt::StructOpt;
use utils::notefield::player_config::{NoteLayout, NoteSkin, PlayerOptions};
use utils::parallel_folder_walk::{load_songs_folder, LoadError};

fn parse_noteskin_path(arg: &OsStr) -> PathBuf {
    PathBuf::from("Noteskins").join(arg)
}

fn parse_simfile_path(arg: &OsStr) -> PathBuf {
    PathBuf::from("Songs").join(arg)
}

fn parse_theme_path(arg: &OsStr) -> PathBuf {
    PathBuf::from("Themes").join(arg)
}

fn set_up_logging() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, _record| out.finish(format_args!("{}", message)))
        .chain(
            OpenOptions::new()
                .write(true)
                .create(true)
                .open("log.txt")?,
        )
        .apply()?;
    Ok(())
}

fn sprite_finder(
    _measure: usize,
    _row_time: f64,
    row_alignment: Fraction,
    note_type: NoteType,
    _column: usize,
) -> Rectangle {
    match note_type {
        NoteType::Tap | NoteType::Hold => {
            let &division = (row_alignment * 4).denom();
            match division {
                1 => Rectangle::new(0.0, 0.0, 1.0, 0.125),
                2 => Rectangle::new(0.0, 0.125, 1.0, 0.125),
                3 => Rectangle::new(0.0, 0.25, 1.0, 0.125),
                4 => Rectangle::new(0.0, 0.375, 1.0, 0.125),
                6 => Rectangle::new(0.0, 0.5, 1.0, 0.125),
                8 => Rectangle::new(0.0, 0.625, 1.0, 0.125),
                12 => Rectangle::new(0.0, 0.75, 1.0, 0.125),
                _ => Rectangle::new(0.0, 0.875, 1.0, 0.125),
            }
        }
        _ => Rectangle::new(0.0, 0.0, 1.0, 1.0),
    }
}

pub fn load_song(sim: &PathBuf) -> Result<(f64, NoteData), LoadError> {
    if let Some(extension) = sim.extension() {
        let mut sim = match File::open(sim.clone()) {
            Ok(file) => file,
            Err(_) => return Err(LoadError::FailedParse),
        };
        match extension.to_str() {
            Some("sm") => {
                notedata::NoteData::from_sm_reader(sim).map_err(|_| LoadError::FailedParse)
            }
            Some("dwi") => {
                notedata::NoteData::from_dwi_reader(sim).map_err(|_| LoadError::FailedParse)
            }
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

#[derive(Clone, Debug, Default, StructOpt)]
#[structopt(name = "RustMania", author, about)]
pub struct SongOptions {
    /// The path to your .sm file
    #[structopt(parse(from_os_str = parse_simfile_path), short, long, default_value(""))]
    simfile: PathBuf,

    /// The rate of the music
    #[structopt(default_value("1.0"), short, long)]
    rate: f64,

    /// The name of your NoteSkin folder
    #[structopt(parse(from_os_str = parse_noteskin_path), short, long, default_value("Default"))]
    noteskin: PathBuf,

    /// The path to your lua theme file
    #[structopt(parse(from_os_str = parse_theme_path), short, long)]
    theme: Option<PathBuf>,
}

#[allow(clippy::too_many_lines)]
fn main() {
    let song_options = SongOptions::from_args();
    set_up_logging().expect("Failed to setup logging");

    let theme: Theme = serde_yaml::from_reader(
        File::open(
            song_options
                .theme
                .clone()
                .unwrap_or_else(|| PathBuf::from("Themes/Default/theme.yml")),
        )
        .expect("Could not open theme file"),
    )
    .expect("Could not parse theme file as YAML");

    let notedata_list = {
        let start_time = Instant::now();
        let notedata_list = load_songs_folder(song_options.simfile.clone(), load_song);
        let duration = Instant::now() - start_time;
        info!("Found {} total songs", notedata_list.len());
        let mut notedata_list = notedata_list
            .into_iter()
            .filter_map(|x| x)
            .collect::<Vec<_>>();
        info!("Of which, {} loaded", notedata_list.len());
        info!(
            "This took {}.{} seconds",
            duration.as_secs(),
            duration.subsec_millis()
        );
        notedata_list.sort_by(|a, b| (a.1).0.partial_cmp(&(b.1).0).unwrap_or(Ordering::Less));
        notedata_list
            .iter()
            .for_each(|x| info!("{:?}, {}", (x.1).1.meta.title, (x.1).0));
        notedata_list
            .into_iter()
            .map(|(path, (difficulty, data))| CacheEntry {
                path,
                difficulty,
                data: data.meta,
            })
            .collect::<Vec<_>>()
    };

    let (context, events_loop) = &mut ContextBuilder::new("rustmania", "ixsetf")
        .add_resource_path("")
        .window_setup(ggez::conf::WindowSetup {
            title: "Rustmania".to_string(),
            ..ggez::conf::WindowSetup::default()
        })
        .build()
        .expect("Failed to build context");

    let default_note_skin =
        NoteSkin::new(&song_options.noteskin, context).expect("Could not open default noteskin");

    let p1_options = PlayerOptions::new(200, 125, 0.8, true, (-128.0, 383.0));
    let p2_options = PlayerOptions::new(600, 125, 1.1, false, (-128.0, 383.0));

    let p1_layout = NoteLayout::new(&default_note_skin, 600, p1_options);
    let p2_layout = NoteLayout::new(&default_note_skin, 600, p2_options);

    let resources = Resources::new(
        vec![TimingData::new()],
        vec![PathBuf::new(); 2],
        vec![p1_layout, p2_layout],
        vec![song_options.rate, 0.0, 12.0, 36.0, 0.0],
        vec![600, 0, 0],
        vec![
            String::new(),
            String::from("Editor placeholder text"),
            String::new(),
        ],
        vec![],
        vec![],
    );

    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let mut path = PathBuf::from(manifest_dir);
        path.push("resources");
        mount(context, &path, true);
    }

    let mut gamestate = GameState::new(
        theme.scene_stack,
        resources,
        vec![
            callbacks::map_to_string,
            callbacks::song_title,
            callbacks::print_resource,
            callbacks::add_one,
            callbacks::subtract_one,
            callbacks::song_path,
            callbacks::song_from_path,
            callbacks::music_path,
        ],
        Globals {
            cache: notedata_list,
            song_options,
        },
        theme.scripts,
    );
    if let Err(e) = ggez::event::run(context, events_loop, &mut gamestate) {
        debug!("Error: {}", e);
    } else {
        debug!("Exit successful.");
    }
}
