#[macro_use]
extern crate nom;
extern crate chrono;
#[macro_use]
extern crate clap;
extern crate ggez;
extern crate num_rational;
extern crate rlua;

mod gamestate;
mod lua;
mod notedata;
mod notefield;
mod player_config;
mod screen;
mod timingdata;

use clap::{App, Arg};
use ggez::conf;
use ggez::graphics::{set_background_color, Color, Rect};
use notedata::NoteType;
use num_rational::Rational32;
use player_config::NoteSkin;
use rlua::{Lua, MultiValue};
use std::fs::File;
use std::io::Read;

fn sprite_finder(
    _measure: usize,
    _row_time: f64,
    row_alignment: Rational32,
    _note_type: NoteType,
    _column: usize,
) -> Rect {
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
                .help("The path to your NoteSkin image file.")
                .index(2)
                .required(true),
            Arg::with_name("Theme")
                .help("The path to your theme lua file.")
                .index(3)
                .required(true)
        ])
        .after_help("Licenced under MIT.")
        .get_matches();

    let simfile = File::open(
        matches
            .value_of("SimFile")
            .expect("No path for simfile received."),
    ).expect("Could not open simfile.");

    let noteskin = matches
        .value_of("NoteSkin")
        .expect("No path for NoteSkin specified");

    let c = conf::Conf::from_toml_file(&mut File::open("src/config.toml").unwrap()).unwrap();
    let context = &mut ggez::Context::load_from_conf("rustmania", "ixsetf", c).unwrap();
    set_background_color(context, Color::new(0.0, 0.0, 0.0, 1.0));

    let current_theme = Lua::new();

    let theme_address = matches
        .value_of("Theme")
        .expect("No path for theme received.");
    let mut theme_file = File::open(theme_address).unwrap();
    let mut theme_lines = String::new();
    theme_file
        .read_to_string(&mut theme_lines)
        .expect(&format!("Error Reading: {}", theme_address));
    let theme_lines = theme_lines.lines();
    let mut current_chunk = String::new();

    lua::create_lua_functions(&current_theme).unwrap();

    for theme_line in theme_lines {
        current_chunk += "\n";
        current_chunk += theme_line;
        if let Ok(output) = current_theme.eval::<MultiValue>(&current_chunk, None) {
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
    }

    let default_note_skin = NoteSkin::new(
        ggez::graphics::Image::new(context, noteskin).expect("Could not parse noteskin from path."),
        ggez::graphics::Image::new(context, "/receptor.png").expect("Could not parse receptor."),
        ggez::graphics::Image::new(context, "/Judgments.png").expect("Could not parse judgments."),
    );

    let mut p1_layout = player_config::NoteLayout::new(
        [72, 136, 200, 264],
        &default_note_skin,
        436,
        ggez::graphics::Point2::new(72.0, 183.0),
    );

    let mut p2_layout = player_config::NoteLayout::new(
        [472, 536, 600, 664],
        &default_note_skin,
        100,
        ggez::graphics::Point2::new(472.0, 383.0),
    );

    p1_layout.set_scroll_speed(-0.7);

    p2_layout.set_scroll_speed(1.0);

    let notedata = notedata::NoteData::from_sm(simfile).expect("Failed to parse .sm file.");

    let music = ggez::audio::Source::new(
        context,
        format!(
            "/{}",
            notedata
                .data
                .music_path
                .clone()
                .expect("No valid music path")
        ),
    ).expect("couldnt open audio file");

    let notes = timingdata::TimingData::from_notedata(&notedata, sprite_finder);

    let notefield_p1 = notefield::Notefield::new(&p1_layout, &notes, 600);
    let notefield_p2 = notefield::Notefield::new(&p2_layout, &notes, 600);

    let mut gameplay_screen = screen::Screen::new(vec![
        Box::new(notefield_p1),
        Box::new(notefield_p2),
        Box::new(music),
    ]);

    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let mut path = std::path::PathBuf::from(manifest_dir);
        path.push("resources");
        context.filesystem.mount(&path, true);
    }

    if let Err(e) = gameplay_screen.start() {
        println!("Error starting screen: {}", e)
    }
    if let Err(e) = ggez::event::run(context, &mut gameplay_screen) {
        println!("Error: {}", e);
    } else {
        println!("Exit successful.");
    }
}
