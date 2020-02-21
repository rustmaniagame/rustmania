use crate::screen::{Element, Message, Resource};
use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};
use ggez::{event::KeyCode, Context, GameError};
use lewton::inside_ogg::OggStreamReader;
use minimp3::Decoder;
use std::{
    self,
    fs::File,
    path::{Path, PathBuf},
    sync::mpsc::{channel, Receiver, Sender},
    thread,
    time::{Duration, Instant},
};

const CORRECTION_DEGREE: f64 = 0.00002;

pub struct Music {
    rate: f64,
    path: PathBuf,
    sender: Option<Sender<bool>>,
}

impl Music {
    pub fn new(rate: f64, path: PathBuf) -> Self {
        Self {
            rate,
            path,
            sender: None,
        }
    }
}

//Known issue: playback only operates correctly on two channel audio
//single channel audio needs to be accounted for
fn play_file<T>(start_time: Instant, rate: f64, path: T, recv: Receiver<bool>)
where
    T: AsRef<Path>,
{
    let host = cpal::default_host();
    let event_loop = host.event_loop();
    let device = host
        .default_output_device()
        .expect("no output device available");
    let mut supported_formats_range = device
        .supported_output_formats()
        .expect("error while querying formats");
    let format = supported_formats_range
        .next()
        .expect("no supported format?!")
        .with_max_sample_rate();
    let stream_id = event_loop.build_output_stream(&device, &format).unwrap();

    let sample_rate = f64::from(format.sample_rate.0);

    let mut ext = PathBuf::new();
    ext.push(&path);
    let (stream_sample_rate, samples) = match ext.extension() {
        Some(ext) => match ext.to_str() {
            Some("ogg") => decode_ogg(path),
            Some("mp3") => decode_mp3(path),
            Some("wav") => decode_wav(path),
            _ => panic!("unrecognized file type"),
        },
        _ => panic!("no file type found"),
    };

    let mut sample_index = 0.0;

    let to_sample_number = |dur: Duration| {
        dur.as_secs() as f64 * sample_rate
            + f64::from(dur.subsec_nanos()) * (sample_rate / 1_000_000_000.0)
    };

    let sample_factor = f64::from(stream_sample_rate) / sample_rate;

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

    event_loop
        .play_stream(stream_id.clone())
        .expect("failed to play_stream");

    event_loop.run(move |_, stream_result| {
        let data = match stream_result {
            Ok(data) => data,
            Err(err) => panic!("an error occurred on stream {:?}: {}", stream_id, err),
        };

        if let cpal::StreamData::Output {
            buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer),
        } = data
        {
            for sample in buffer.chunks_mut(format.channels as usize) {
                let value = next_value(start_time);
                for (i, out) in sample.iter_mut().enumerate() {
                    *out = f32::from(*samples.get(value + i).unwrap_or(&0))
                        / f32::from(i16::max_value());
                }
            }
            if recv.try_recv().is_ok() {
                panic!("Intentional panic");
            }
        }
    });
}

fn decode_ogg<T>(path: T) -> (i32, Vec<i16>)
where
    T: AsRef<Path>,
{
    let mut srr = OggStreamReader::new(File::open(path).unwrap()).unwrap();
    let stream_sample_rate = srr.ident_hdr.audio_sample_rate as i32;

    let mut coolvec = Vec::<i16>::new();
    while let Some(pck_samples) = srr.read_dec_packet_itl().unwrap() {
        match srr.ident_hdr.audio_channels {
            2 => coolvec.append(&mut pck_samples.clone()),
            n => panic!("unsupported number of channels: {}", n),
        };
    }
    (stream_sample_rate, coolvec)
}

//Known issue: the last frame seems to be dropped from the end of the file
fn decode_mp3<T>(path: T) -> (i32, Vec<i16>)
where
    T: AsRef<Path>,
{
    let mut decoder = Decoder::new(File::open(path).unwrap());

    let mut frames = Vec::new();

    let stream_sample_rate = if let Ok(frame) = decoder.next_frame() {
        frames.append(&mut frame.data.clone());
        frame.sample_rate
    } else {
        0
    };

    while let Ok(frame) = decoder.next_frame() {
        frames.append(&mut frame.data.clone());
    }
    (stream_sample_rate, frames)
}

fn decode_wav<T>(path: T) -> (i32, Vec<i16>)
where
    T: AsRef<Path>,
{
    let mut reader = hound::WavReader::open(path).unwrap();
    (
        reader.spec().sample_rate as i32,
        reader
            .samples::<i16>()
            .filter_map(Result::ok)
            .collect::<Vec<_>>(),
    )
}

impl Element for Music {
    fn run(&mut self, _ctx: &mut Context, _time: Option<i64>) -> Result<Message, GameError> {
        Ok(Message::None)
    }
    fn start(&mut self, time: Option<Instant>) -> Result<Message, GameError> {
        if let Some(time) = time {
            let rate = self.rate;
            let path = self.path.clone();
            let (send, recv) = channel();
            self.sender = Some(send);
            thread::spawn(move || play_file(time, rate, path, recv));
        }
        Ok(Message::None)
    }
    fn finish(&mut self) -> Option<Resource> {
        if let Some(sender) = &self.sender {
            sender.send(true).expect("fuck");
        }
        None
    }
    fn handle_event(&mut self, _keycode: KeyCode, _time: Option<i64>, _key_down: bool) {}
}
