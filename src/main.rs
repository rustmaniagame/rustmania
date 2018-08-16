#[macro_use]
extern crate nom;
extern crate chrono;
extern crate ggez;

mod fraction;
mod gameplay_screen;
mod notedata;
mod player_config;
mod timingdata;

use fraction::Fraction;
use ggez::conf;
use notedata::NoteType;
use std::fs::File;

fn sprite_finder(
    _measure: usize,
    _row_time: f64,
    row_alignment: Fraction,
    _note_type: NoteType,
    _column: usize,
) -> usize {
    let (_, division) = (row_alignment * 4).contents();
    match division {
        1 => 0,
        2 => 1,
        3 => 2,
        4 => 3,
        6 => 4,
        8 => 5,
        12 => 6,
        16 => 7,
        24 => 8,
        _ => 9,
    }
}

fn main() {
    let c = conf::Conf::from_toml_file(&mut File::open("src/config.toml").unwrap()).unwrap();
    let context = &mut ggez::Context::load_from_conf("rustmania", "ixsetf", c).unwrap();

    let mut p1_layout = player_config::NoteLayout::new(
        [72, 136, 200, 264],
        ggez::graphics::Image::new(context, "/arrows.png").unwrap(),
        ggez::graphics::Image::new(context, "/receptor.png").unwrap(),
        250,
    );

    let mut p2_layout = player_config::NoteLayout::new(
        [472, 536, 600, 664],
        ggez::graphics::Image::new(context, "/arrows.png").unwrap(),
        ggez::graphics::Image::new(context, "/receptor.png").unwrap(),
        250,
    );

    if let Err(e) = p1_layout.set_scroll_speed(0.7) {
        println!("Couldn't set scroll speed: {}", e);
    }

    if let Err(e) = p2_layout.set_scroll_speed(1.4) {
        println!("Couldn't set scroll speed: {}", e);
    }

    let notedata = notedata::NoteData::from_sm();

    let notes =
        timingdata::TimingData::from_notedata(notedata, &p1_layout.arrows_sprite, sprite_finder);

    let mut game_screen =
        gameplay_screen::GameplayScreen::new(&p1_layout, &notes, &p2_layout, &notes, 600);

    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let mut path = std::path::PathBuf::from(manifest_dir);
        path.push("resources");
        context.filesystem.mount(&path, true);
    }

    game_screen.start();
    if let Err(e) = ggez::event::run(context, &mut game_screen) {
        println!("Error: {}", e);
    } else {
        println!("Exit successful.");
    }
}
