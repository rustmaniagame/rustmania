use graphics::{Context, WinitState};
use winit::{
    Event, WindowEvent,
};

fn main() {
    let mut winit_state = WinitState::default();
    let mut context = Context::build(&winit_state.window, "Rustmania").unwrap();
    let (_frame_width, _frame_height) = winit_state
        .window
        .get_inner_size()
        .map(|logical| logical.into())
        .unwrap_or((0.0, 0.0));
    let mut running = true;
    while running {
        winit_state.events_loop.poll_events(|event| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => running = false,
            _ => (),
        });
        context.clear([1.0, 0.0, 0.0, 1.0]).expect("fail");
    }
}