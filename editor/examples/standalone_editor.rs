use editor::Editor;
use ggez::ContextBuilder;
use notefield::player_config::PlayerOptions;
use std::env::set_current_dir;
use std::path::PathBuf;

fn main() {
    set_current_dir("..").expect("We need to change current dir because of ggez weirdness");
    let (context, events_loop) = &mut ContextBuilder::new("rustmania editor", "ixsetf")
        .add_resource_path("")
        .window_setup(ggez::conf::WindowSetup {
            title: "Rustmania Editor".to_string(),
            ..ggez::conf::WindowSetup::default()
        })
        .build()
        .expect("Failed to build context");
    let options = PlayerOptions::new(200, 250, 0.8, true, (-128.0, 383.0));
    let mut editor = Editor::new(PathBuf::from("Noteskins\\Default"), 600, options, context);
    if let Err(e) = ggez::event::run(context, events_loop, &mut editor) {
        println!("Error: {}", e);
    } else {
        println!("Exit successful.");
    }
}
