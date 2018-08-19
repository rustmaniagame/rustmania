#[macro_use]
extern crate nom;
extern crate chrono;
#[macro_use]
extern crate clap;
extern crate ggez;
extern crate num_rational;

mod gameplay_screen;
mod notedata;
mod player_config;
mod timingdata;

use clap::{App, Arg};
use ggez::conf;
use ggez::graphics::{set_background_color, Color, Rect};
use notedata::NoteType;
use num_rational::Rational32;
use std::fs::File;

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
    let _matches = App::new("Rustmania")
        .author(crate_authors!())
        .version("0.1.0")
        .about("A rhythm game in the vein of Stepmania and Etterna, currently in very early stages of development.")
        .args(&[
            Arg::with_name("SimFile")
                .help("The path to your .sm file.")
                .required(true),
            Arg::with_name("NoteSkin")
                .help("The path to your NoteSkin image file.")
                .required(true)
        ])
        .after_help("Licenced under MIT.")
        .get_matches();

    let c = conf::Conf::from_toml_file(&mut File::open("src/config.toml").unwrap()).unwrap();
    let context = &mut ggez::Context::load_from_conf("rustmania", "ixsetf", c).unwrap();
    set_background_color(context, Color::new(0.0, 0.0, 0.0, 1.0));

    let mut p1_layout = player_config::NoteLayout::new(
        [72, 136, 200, 264],
        ggez::graphics::Image::new(context, "/arrows.png").unwrap(),
        ggez::graphics::Image::new(context, "/receptor.png").unwrap(),
        100,
    );

    let mut p2_layout = player_config::NoteLayout::new(
        [472, 536, 600, 664],
        ggez::graphics::Image::new(context, "/arrows.png").unwrap(),
        ggez::graphics::Image::new(context, "/receptor.png").unwrap(),
        100,
    );

    if let Err(e) = p1_layout.set_scroll_speed(0.5) {
        println!("Couldn't set scroll speed: {}", e);
    }

    if let Err(e) = p2_layout.set_scroll_speed(1.0) {
        println!("Couldn't set scroll speed: {}", e);
    }

    let notedata = notedata::NoteData::from_sm();

    let notes = timingdata::TimingData::from_notedata(notedata, sprite_finder);

    let mut game_screen =
        gameplay_screen::GameplayScreen::new(&p1_layout, &notes, &p2_layout, &notes, 600);

    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let mut path = std::path::PathBuf::from(manifest_dir);
        path.push("resources");
        context.filesystem.mount(&path, true);
    }

    if let Err(e) = game_screen.start() {
        println!("Error starting screen: {}", e)
    }
    if let Err(e) = ggez::event::run(context, &mut game_screen) {
        println!("Error: {}", e);
    } else {
        println!("Exit successful.");
    }
}
