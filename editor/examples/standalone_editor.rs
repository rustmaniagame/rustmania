use editor::Editor;
use notedata::NoteType;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn handle_keypress(editor: &mut Editor, code: VirtualKeyCode) {
    match code {
        VirtualKeyCode::Left => {
            if editor.snap > 1 {
                editor.snap -= 1;
                println!("snap changed to: {}", editor.snap);
            }
        }
        VirtualKeyCode::Right => {
            editor.snap += 1;
            println!("snap changed to: {}", editor.snap);
        }
        VirtualKeyCode::Up => {
            editor.previous_snap();
            let (measure, beat) = editor.get_beat();
            println!(
                "measure: {} beat: {} row: {:?}",
                measure,
                beat,
                editor.get_noterow()
            );
        }
        VirtualKeyCode::Down => {
            editor.next_snap();
            let (measure, beat) = editor.get_beat();
            println!(
                "measure: {} beat: {} row: {:?}",
                measure,
                beat,
                editor.get_noterow()
            );
        }
        VirtualKeyCode::Key1 => {
            editor.toggle_note(0, NoteType::Tap);
            let (measure, beat) = editor.get_beat();
            println!(
                "measure: {} beat: {} row: {:?}",
                measure,
                beat,
                editor.get_noterow()
            );
        }
        VirtualKeyCode::Key2 => {
            editor.toggle_note(1, NoteType::Tap);
            let (measure, beat) = editor.get_beat();
            println!(
                "measure: {} beat: {} row: {:?}",
                measure,
                beat,
                editor.get_noterow()
            );
        }
        VirtualKeyCode::Key3 => {
            editor.toggle_note(2, NoteType::Tap);
            let (measure, beat) = editor.get_beat();
            println!(
                "measure: {} beat: {} row: {:?}",
                measure,
                beat,
                editor.get_noterow()
            );
        }
        VirtualKeyCode::Key4 => {
            editor.toggle_note(3, NoteType::Tap);
            let (measure, beat) = editor.get_beat();
            println!(
                "measure: {} beat: {} row: {:?}",
                measure,
                beat,
                editor.get_noterow()
            );
        }
        VirtualKeyCode::B => editor.add_bpm(120.0),
        VirtualKeyCode::Return => {
            let the_thing = editor
                .export()
                .map(|data| data.to_sm_string())
                .unwrap_or(String::from("Failed"));
            println!("{}", the_thing);
        }
        _ => {}
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut editor = Editor::new();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(keycode),
                                ..
                            },
                        ..
                    },
                ..
            } => {
                handle_keypress(&mut editor, keycode);
            }
            _ => (),
        }
    })
}
