use cpal;
use ggez::{Context, GameError, event::Keycode};
use screen::Element;
use std;
use std::f64::consts::PI;
use std::thread;
use std::time::{Duration, Instant};

const CORRECTION_DEGREE: f64 = 0.0001;

pub struct Music {}

impl Music {
    pub fn new() -> Self {
        Music {}
    }
}

fn do_stuff(start_time: Instant) {
    let device = cpal::default_output_device().expect("Failed to get default output device");
    let format = device
        .default_output_format()
        .expect("Failed to get default output format");
    let event_loop = cpal::EventLoop::new();
    let stream_id = event_loop.build_output_stream(&device, &format).unwrap();
    event_loop.play_stream(stream_id.clone());

    let sample_rate = format.sample_rate.0 as f64;
    println!("{}", sample_rate);
    let mut sample_clock = 0.0;

    let to_sample_number = |dur: Duration| {
        dur.as_secs() as f64 * sample_rate + dur.subsec_nanos() as f64 * (sample_rate / 1000_000_000.0)
    };

    let mut next_value = |time: Instant| {
        let now = Instant::now();
        if now > time {
            sample_clock =
                (1.0 - CORRECTION_DEGREE) * (sample_clock + 1.0) + CORRECTION_DEGREE * to_sample_number(now.duration_since(time));
            ((sample_clock) as f64 / sample_rate * 440.0 * PI).sin()
        } else {
            0.0
        }
    };

    event_loop.run(move |_, data| match data {
        cpal::StreamData::Output {
            buffer: cpal::UnknownTypeOutputBuffer::U16(mut buffer),
        } => {
            for sample in buffer.chunks_mut(format.channels as usize) {
                let value = ((next_value(start_time) * 0.5 + 0.5) * std::u16::MAX as f64) as u16;
                for out in sample.iter_mut() {
                    *out = value;
                }
            }
        }
        cpal::StreamData::Output {
            buffer: cpal::UnknownTypeOutputBuffer::I16(mut buffer),
        } => {
            for sample in buffer.chunks_mut(format.channels as usize) {
                let value = (next_value(start_time) * std::i16::MAX as f64) as i16;
                for out in sample.iter_mut() {
                    *out = value;
                }
            }
        }
        cpal::StreamData::Output {
            buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer),
        } => {
            for sample in buffer.chunks_mut(format.channels as usize) {
                let value = next_value(start_time);
                for out in sample.iter_mut() {
                    *out = value as f32;
                }
            }
        }
        _ => (),
    });
}

impl Element for Music {
    fn run(&mut self, _ctx: &mut Context, _time: Option<i64>) -> Result<(), GameError> {
        Ok(())
    }
    fn start(&mut self, time: Option<Instant>) -> Result<(), GameError> {
        if let Some(time) = time {
            thread::spawn(move || do_stuff(time));
        }
        Ok(())
    }
    fn handle_event(&mut self, _keycode: Keycode, _time: Option<i64>) {}
}
