#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::module_name_repetitions,
    clippy::must_use_candidate
)]

mod difficulty_calc;
mod gamestate;
mod music;
mod notefield;
mod player_config;
mod screen;
mod text;
mod timingdata;

use crate::{
    gamestate::GameState,
    player_config::NoteSkin,
    screen::{
        ElementMap, ElementType, ResourceMap, ResourceType, Resources, ScreenBuilder, ScriptMap,
    },
    timingdata::{CalcInfo, TimingData},
};
use bincode::deserialize;
use ggez::{filesystem::mount, graphics::Rect, ContextBuilder};
use log::{debug, info};
use notedata::{Fraction, NoteData, NoteType};
use parallel_folder_walk::{load_songs_folder, LoadError};
use rand::seq::SliceRandom;
use std::{
    cmp::Ordering,
    ffi::OsStr,
    fs::{File, OpenOptions},
    io::Read,
    path::PathBuf,
    str::from_utf8,
    time::Instant,
};
use structopt::StructOpt;

const NOTEFIELD_SIZE: usize = 4;

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
) -> Rect {
    match note_type {
        NoteType::Tap | NoteType::Hold => {
            let &division = (row_alignment * 4).denom();
            match division {
                1 => Rect::new(0.0, 0.0, 1.0, 0.125),
                2 => Rect::new(0.0, 0.125, 1.0, 0.125),
                3 => Rect::new(0.0, 0.25, 1.0, 0.125),
                4 => Rect::new(0.0, 0.375, 1.0, 0.125),
                6 => Rect::new(0.0, 0.5, 1.0, 0.125),
                8 => Rect::new(0.0, 0.625, 1.0, 0.125),
                12 => Rect::new(0.0, 0.75, 1.0, 0.125),
                _ => Rect::new(0.0, 0.875, 1.0, 0.125),
            }
        }
        _ => Rect::new(0.0, 0.0, 1.0, 1.0),
    }
}

pub fn load_song(sim: &PathBuf) -> Result<(f64, NoteData), LoadError> {
    if let Some(extension) = sim.extension() {
        let mut sim = match File::open(sim.clone()) {
            Ok(file) => file,
            Err(_) => return Err(LoadError::FailedParse),
        };
        match extension.to_str() {
            Some("sm") => notedata::NoteData::from_sm(sim).map_err(|_| LoadError::FailedParse),
            Some("dwi") => notedata::NoteData::from_dwi(sim).map_err(|_| LoadError::FailedParse),
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

mod callbacks {
    use crate::screen::Resource;
    use crate::timingdata::TimingColumn;

    pub fn map_to_string(resource: Option<Resource>) -> Option<Resource> {
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
            _ => Resource::String("".to_owned()),
        })
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "RustMania", author, about)]
struct Opt {
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
    let mut rng = rand::thread_rng();
    let opt = Opt::from_args();

    set_up_logging().expect("Failed to setup logging");

    let (simfile_folder, difficulty, notedata) = {
        let start_time = Instant::now();
        let notedata_list = load_songs_folder(opt.simfile, load_song);
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
        let (simfile_path, (difficulty, notedata)) = notedata_list
            .choose(&mut rng)
            .expect("Failed to select chart from cache")
            .clone();
        let simfile_folder = String::from(
            simfile_path
                .parent()
                .expect("No parent folder for selected file")
                .as_os_str()
                .to_str()
                .expect("failed to parse path"),
        );
        (simfile_folder, difficulty, notedata)
    };
    println!(
        "Selected Song is: {}",
        notedata.meta.title.clone().unwrap_or_default()
    );
    println!("With difficulty: {}", difficulty);

    let (context, events_loop) = &mut ContextBuilder::new("rustmania", "ixsetf")
        .add_resource_path("")
        .window_setup(ggez::conf::WindowSetup {
            title: "Rustmania".to_string(),
            ..ggez::conf::WindowSetup::default()
        })
        .build()
        .expect("Failed to build context");

    let default_note_skin =
        NoteSkin::new(&opt.noteskin, context).expect("Could not open default noteskin");

    let p1_options = player_config::PlayerOptions::new(200, 125, 0.8, true, (-128.0, 383.0));
    let p2_options = player_config::PlayerOptions::new(600, 125, 1.1, false, (-128.0, 383.0));

    let p1_layout = player_config::NoteLayout::new(&default_note_skin, 600, p1_options);
    let p2_layout = player_config::NoteLayout::new(&default_note_skin, 600, p2_options);

    let notes = timingdata::TimingData::from_notedata(&notedata, sprite_finder, opt.rate);

    let resources = Resources::new(
        notes,
        vec![PathBuf::from(format!(
            "{}/{}",
            simfile_folder,
            notedata.meta.music_path.expect("No music path specified")
        ))],
        vec![p1_layout, p2_layout],
        vec![opt.rate, 0.0, 12.0],
        vec![600],
        vec![
            notedata.meta.title.expect("Needs a title"),
            String::from("Results screen placeholder text"),
        ],
        vec![],
        vec![],
    );

    let gameplay_screen = match opt.theme {
        Some(value) => {
            // This currently is not getting the music rate so the theme will have incorrect behavior
            // if the rate specified in the theme is different than the rate passed in through the CLI
            let mut theme = File::open(value).expect("Can not find theme file");
            let mut theme_string = vec![];
            theme
                .read_to_end(&mut theme_string)
                .expect("Could not read theme file completely");
            serde_yaml::from_str(
                from_utf8(&theme_string).expect("Can not parse theme file as string"),
            )
            .expect("Can not parse theme file as YAML")
        }
        None => ScreenBuilder {
            elements: vec![
                ElementType::NOTEFIELD(0, 0, 0),
                ElementType::NOTEFIELD(1, 0, 0),
                ElementType::MUSIC(0, 0),
                ElementType::TEXT(0, 1, 2),
            ],
        },
    };

    let results_screen = ScreenBuilder {
        elements: vec![ElementType::TEXT(2, 1, 2)],
    };

    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let mut path = PathBuf::from(manifest_dir);
        path.push("resources");
        mount(context, &path, true);
    }

    let mut gamestate = GameState::new(
        vec![
            (
                gameplay_screen,
                vec![
                    ResourceMap::Element(ElementMap {
                        element_index: 0,
                        resource_index: 0,
                    }),
                    ResourceMap::Script(ScriptMap {
                        resource_type: ResourceType::Replay,
                        resource_index: 0,
                        script_index: 0,
                    }),
                ],
            ),
            (results_screen, vec![]),
        ],
        resources,
        vec![callbacks::map_to_string],
    );
    if let Err(e) = ggez::event::run(context, events_loop, &mut gamestate) {
        debug!("Error: {}", e);
    } else {
        debug!("Exit successful.");
    }
}
