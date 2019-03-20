mod gamestate;
mod lua;
mod music;
mod notedata;
mod notefield;
mod player_config;
mod screen;
mod timingdata;

use crate::notedata::NoteType;
use crate::player_config::NoteSkin;
use clap::{crate_authors, App, Arg};
use ggez::filesystem::mount;
use ggez::graphics::Rect;
use ggez::ContextBuilder;
use num_rational::Rational32;
use rlua::{Error, Lua, MultiValue};
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;

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
    let matches = App::new("Rustmania")
        .author(crate_authors!())
        .version("0.1.0")
        .about("A rhythm game in the vein of Stepmania and Etterna, currently in very early stages of development.")
        .args(&[
            Arg::with_name("SimFile")
                .help("The path to your .sm file.")
                .index(1)
                .required(true),
            Arg::with_name("NoteSkin")
                .help("The name of your NoteSkin folder.")
                .index(2)
                .required(true),
            Arg::with_name("Theme")
                .help("The path to your lua theme file.")
                .index(3)
                .required(true),
            Arg::with_name("Rate")
                .help("The rate of the music.")
                .index(4)
                .required(false)
        ])
        .after_help("Licenced under MIT.")
        .get_matches();

    let simfile_folder = format!(
        "Songs/{}",
        matches
            .value_of("SimFile")
            .expect("No path for simfile received.")
    );

    let simfile_list = walkdir::WalkDir::new(simfile_folder.clone())
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension() == Some(OsStr::new("sm")));

    let noteskin = matches
        .value_of("NoteSkin")
        .expect("No path for NoteSkin specified");

    let theme_address = matches
        .value_of("Theme")
        .expect("No path for theme received.");

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
    //set_background_color(context, Color::new(0.0, 0.0, 0.0, 1.0));

    let current_theme = Lua::new();

    let mut theme_file = File::open(theme_address).expect("Couldn't open theme file");
    let mut theme_lines = String::new();
    theme_file
        .read_to_string(&mut theme_lines)
        .expect(&format!("Error Reading: {}", theme_address));
    let theme_lines = theme_lines.lines();
    let mut current_chunk = String::new();

    lua::create_lua_functions(&current_theme).expect("Couldn't create lua functions");

    for theme_line in theme_lines {
        current_chunk += "\n";
        current_chunk += theme_line;
        match current_theme.eval::<_, MultiValue>(&current_chunk, None) {
            Ok(output) => {
                println!("{}", current_chunk);
                println!(
                    "{}",
                    output
                        .iter()
                        .map(|value| format!("{:?}", value))
                        .collect::<Vec<_>>()
                        .join("\t")
                );
                current_chunk.clear();
            }
            Err(Error::SyntaxError {
                incomplete_input: true,
                ..
            }) => {}
            _ => break,
        }
    }

    let default_note_skin = NoteSkin::from_path(&format!("Noteskins\\{}", noteskin), context)
        .expect("Could not open default noteskin");

    let p1_options = player_config::PlayerOptions::new(200, 125, 0.8, true, (-128.0, 383.0));
    let p2_options = player_config::PlayerOptions::new(600, 125, 1.1, false, (-128.0, 383.0));

    let p1_layout = player_config::NoteLayout::new(&default_note_skin, 600, p1_options);

    let p2_layout = player_config::NoteLayout::new(&default_note_skin, 600, p2_options);

    let notedata = simfile_list
        .filter_map(|sim| File::open(sim.path()).ok())
        .find_map(|sim| notedata::NoteData::from_sm(sim).ok())
        .expect("Could not find simfile or simfile failed to open");
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
