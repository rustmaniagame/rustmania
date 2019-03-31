mod gamestate;
mod music;
mod notedata;
mod notefield;
mod player_config;
mod screen;
mod song_loader;
mod timingdata;

use crate::{notedata::NoteType, player_config::NoteSkin};
use clap::{crate_authors, crate_version, App, Arg};
use ggez::{filesystem::mount, graphics::Rect, ContextBuilder};
use num_rational::Rational32;
use rand::seq::SliceRandom;
use std::time::Instant;

fn sprite_finder(
    _measure: usize,
    _row_time: f64,
    row_alignment: Rational32,
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
                16 => Rect::new(0.0, 0.875, 1.0, 0.125),
                24 => Rect::new(0.0, 0.875, 1.0, 0.125),
                _ => Rect::new(0.0, 0.875, 1.0, 0.125),
            }
        }
        _ => Rect::new(0.0, 0.0, 1.0, 1.0),
    }
}

fn main() {
    let mut rng = rand::thread_rng();
    let matches = App::new("Rustmania")
        .author(crate_authors!())
        .version(crate_version!())
        .about("A rhythm game in the vein of Stepmania and Etterna, currently in very early stages of development.")
        .args(&[
            Arg::with_name("SimFile")
                .help("The path to your .sm file.")
                .index(1)
                .required(false),
            Arg::with_name("Rate")
                .help("The rate of the music.")
                .index(2)
                .required(false),
            Arg::with_name("NoteSkin")
                .help("The name of your NoteSkin folder.")
                .index(3)
                .required(false),
            Arg::with_name("Theme")
                .help("The path to your lua theme file.")
                .index(4)
                .required(false),
        ])
        .after_help("Licenced under MIT.")
        .get_matches();

    let (simfile_folder, notedata) = match matches.value_of("SimFile") {
        Some(value) => (
            format!("Songs/{}", value),
            song_loader::load_song(format!("Songs/{}", value))
                .expect("Failed to load song from path specified"),
        ),
        None => {
            let start_time = Instant::now();
            let notedata_list = song_loader::load_songs_folder("Songs");
            let duration = Instant::now() - start_time;
            println!("Found {} total songs", notedata_list.len());
            let notedata_list = notedata_list
                .into_iter()
                .filter(|x| x.1.is_some())
                .map(|(p, x)| (p, x.unwrap()))
                .collect::<Vec<_>>();
            println!("Of which, {} loaded", notedata_list.len());
            println!(
                "This took {}.{} seconds",
                duration.as_secs(),
                duration.subsec_millis()
            );
            let (simfile_folder, notedata) = notedata_list.choose(&mut rng).unwrap().clone();
            let simfile_folder = simfile_folder
                .into_os_string()
                .into_string()
                .expect("failed to parse path");
            (simfile_folder, notedata)
        }
    };

    let noteskin = matches.value_of("NoteSkin").unwrap_or("Default");

    let music_rate = matches
        .value_of("Rate")
        .unwrap_or("1.0")
        .parse()
        .unwrap_or(1.0);

    let (context, events_loop) = &mut ContextBuilder::new("rustmania", "ixsetf")
        .add_resource_path("")
        .window_setup(ggez::conf::WindowSetup {
            title: "Rustmania".to_string(),
            ..Default::default()
        })
        .build()
        .expect("Failed to build context");

    let default_note_skin = NoteSkin::from_path(&format!("Noteskins\\{}", noteskin), context)
        .expect("Could not open default noteskin");

    let p1_options = player_config::PlayerOptions::new(200, 125, 0.8, true, (-128.0, 383.0));
    let p2_options = player_config::PlayerOptions::new(600, 125, 1.1, false, (-128.0, 383.0));

    let p1_layout = player_config::NoteLayout::new(&default_note_skin, 600, p1_options);

    let p2_layout = player_config::NoteLayout::new(&default_note_skin, 600, p2_options);

    let notes = timingdata::TimingData::from_notedata(&notedata, sprite_finder, music_rate);
    let notefield_p1 = notefield::Notefield::new(&p1_layout, &notes[0], 600);
    let notefield_p2 = notefield::Notefield::new(&p2_layout, &notes[0], 600);
    let music = music::Music::new(
        music_rate,
        format!(
            "{}/{}",
            simfile_folder,
            notedata.data.music_path.expect("No music path specified")
        ),
    );

    let mut gameplay_screen = screen::Screen::new(vec![
        Box::new(notefield_p1),
        Box::new(notefield_p2),
        Box::new(music),
    ]);

    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let mut path = std::path::PathBuf::from(manifest_dir);
        path.push("resources");
        mount(context, &path, true);
    }

    if let Err(e) = gameplay_screen.start() {
        println!("Error starting screen: {}", e)
    }
    if let Err(e) = ggez::event::run(context, events_loop, &mut gameplay_screen) {
        println!("Error: {}", e);
    } else {
        println!("Exit successful.");
    }
}
