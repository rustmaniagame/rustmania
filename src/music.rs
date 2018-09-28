use cpal;
use ggez::{Context, GameError, event::Keycode};
use lewton::inside_ogg::OggStreamReader;
use screen::Element;
use std;
use std::fs::File;
use std::thread;
use std::time::{Duration, Instant};

const CORRECTION_DEGREE: f64 = 0.00002;

pub struct Music {
    rate: f64,
    path: String,
}

impl Music {
    pub fn new(rate: f64, path: String) -> Self {
        Music { rate, path }
    }
}

fn play_ogg(start_time: Instant, rate: f64, path: String) {
    let device = cpal::default_output_device().expect("Failed to get default output device");
    let format = device
        .default_output_format()
        .expect("Failed to get default output format");
    let event_loop = cpal::EventLoop::new();
    let stream_id = event_loop.build_output_stream(&device, &format).unwrap();

    let f = File::open(path).unwrap();
    let sample_rate = format.sample_rate.0 as f64;

    let mut srr = OggStreamReader::new(f).unwrap();
    let stream_sample_rate = srr.ident_hdr.audio_sample_rate as i32;

    let mut coolvec = Vec::<i16>::new();
    while let Some(pck_samples) = srr.read_dec_packet_itl().unwrap() {
        match srr.ident_hdr.audio_channels {
            2 => coolvec.append(&mut pck_samples.clone()),
            n => panic!("unsupported number of channels: {}", n),
        };
    }

    let mut sample_index = 0.0;

    let to_sample_number = |dur: Duration| {
        dur.as_secs() as f64 * sample_rate
            + dur.subsec_nanos() as f64 * (sample_rate / 1000_000_000.0)
    };

    let sample_factor = stream_sample_rate as f64 / sample_rate;

    let mut next_value = |time: Instant| {
        let now = Instant::now();
        if now > time {
            sample_index = (1.0 - CORRECTION_DEGREE) * (sample_index + 1.0)
                + CORRECTION_DEGREE * to_sample_number(now.duration_since(time));
            (sample_index * sample_factor * rate) as usize * 2
        } else {
            0
        }
    };

    event_loop.play_stream(stream_id.clone());

    event_loop.run(move |_, data| {
        if let cpal::StreamData::Output {
            buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer),
        } = data
        {
            for sample in buffer.chunks_mut(format.channels as usize) {
                let value = next_value(start_time);
                for (i, out) in sample.iter_mut().enumerate() {
                    *out = *coolvec.get(value + i).unwrap_or(&0) as f32 / std::i16::MAX as f32;
                }
            }
        }
    });
}

impl Element for Music {
    fn run(&mut self, _ctx: &mut Context, _time: Option<i64>) -> Result<(), GameError> {
        Ok(())
    }
    fn start(&mut self, time: Option<Instant>) -> Result<(), GameError> {
        if let Some(time) = time {
            let rate = self.rate;
            let path = self.path.clone();
            thread::spawn(move || play_ogg(time, rate, path));
        }
        Ok(())
    }
    fn handle_event(&mut self, _keycode: Keycode, _time: Option<i64>) {}
}
