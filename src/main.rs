extern crate ggez;
extern crate chrono;

pub mod gameplay_screen;
pub mod player_config;
use ggez::conf;
use std::fs::File;
use std::time::Duration;

fn main() {
    let c = conf::Conf::from_toml_file(&mut File::open("src/config.toml").unwrap()).unwrap();
    let context = &mut ggez::Context::load_from_conf("rhythm attempt", "ixsetf", c).unwrap();

    let mut default_layout = player_config::NoteLayout::new([272, 336, 400, 464], ggez::graphics::Image::new(context, "/arrow.png").unwrap(),
                                                        ggez::graphics::Image::new(context, "/receptor.png").unwrap(), 50);

    if let Err(e) = default_layout.set_scroll_speed(0.2){
        println!("Couldn't set scroll speed: {}", e);
    }

    let mut game_screen = gameplay_screen::GameplayScreen::new(default_layout);

    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let mut path = std::path::PathBuf::from(manifest_dir);
        path.push("resources");
        context.filesystem.mount(&path, true);
    }

    game_screen.add_notes(&mut [vec![Duration::from_millis(2000),Duration::from_millis(2375),Duration::from_millis(2750),Duration::from_millis(3000)],
        vec![Duration::from_millis(2000),Duration::from_millis(2250),Duration::from_millis(2625),Duration::from_millis(3000)],
    vec![Duration::from_millis(2125),Duration::from_millis(2500),Duration::from_millis(2750),Duration::from_millis(4000)],
    vec![Duration::from_millis(2250),Duration::from_millis(2500),Duration::from_millis(2875),Duration::from_millis(4000)],
                                    ]);
    game_screen.start();
    if let Err(e) = ggez::event::run(context, &mut game_screen) {
        println!("Error: {}", e);
    } else {
        println!("Exit successful.");
    }
}
