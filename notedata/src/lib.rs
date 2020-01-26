#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::module_name_repetitions,
    clippy::must_use_candidate
)]

mod dwi_parser;
mod sm_parser;

mod parser_generic;

mod sm_writer;

pub use num_rational::Rational32 as Fraction;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::io;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct BeatPair<T> {
    pub beat: i32,
    pub sub_beat: Fraction,
    pub value: T,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum NoteType {
    Tap,
    Hold,
    Roll,
    Mine,
    Lift,
    Fake,
    HoldEnd,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DisplayBpm {
    Range(f64, f64),
    Static(f64),
    Random,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Note {
    pub note_type: NoteType,
    pub column: usize,
}

type NoteRow = Vec<Note>;
pub type Measure = Vec<(NoteRow, Fraction)>;
pub type Chart = Vec<Measure>;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct ChartMetadata {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub artist: Option<String>,
    pub title_translit: Option<String>,
    pub subtitle_translit: Option<String>,
    pub artist_translit: Option<String>,
    pub genre: Option<String>,
    pub credit: Option<String>,
    pub banner_path: Option<String>,
    pub background_path: Option<String>,
    pub lyrics_path: Option<String>,
    pub cd_title: Option<String>,
    pub music_path: Option<String>,
    pub offset: Option<f64>,
    pub bpms: Vec<BeatPair<f64>>,
    pub stops: Option<Vec<BeatPair<f64>>>,
    pub sample_start: Option<f64>,
    pub sample_length: Option<f64>,
    pub display_bpm: Option<DisplayBpm>,
    pub selectable: Option<String>,
    //it is unclear how this is used in practice, may be better as Option<bool>
    pub background_changes: Option<Vec<BeatPair<String>>>,
    pub foreground_changes: Option<Vec<BeatPair<String>>>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct NoteData {
    pub charts: Vec<Chart>,
    pub meta: ChartMetadata,
}

impl<T> BeatPair<T> {
    #[must_use]
    fn at_start(value: T) -> Self {
        Self {
            beat: 0,
            sub_beat: Fraction::new(0, 1),
            value,
        }
    }

    #[must_use]
    fn from_pair(beat: f64, value: T) -> Option<Self> {
        let ratio = Fraction::approximate_float(beat)?;
        Some(Self {
            beat: ratio.to_integer(),
            sub_beat: ratio.fract(),
            value,
        })
    }
}

impl Note {
    #[must_use]
    pub fn new(note_type: NoteType, column: usize) -> Self {
        Self { note_type, column }
    }
}

impl ChartMetadata {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl NoteData {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn charts(&mut self, charts: Vec<Chart>) -> &mut Self {
        self.charts = charts;
        self
    }

    pub fn meta(&mut self, meta: ChartMetadata) -> &mut Self {
        self.meta = meta;
        self
    }

    #[must_use]
    pub fn to_sm_string(&self) -> String {
        sm_writer::write_sm(&self)
    }

    pub fn from_sm_reader(mut reader: impl io::Read) -> io::Result<Self> {
        let mut sm_string = String::new();
        reader.read_to_string(&mut sm_string)?;

        sm_parser::parse(&sm_string).map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))
    }

    pub fn to_sm_writer(&self, mut writer: impl io::Write) -> io::Result<()> {
        writer.write_all(&self.to_sm_string().into_bytes())
    }

    pub fn from_dwi_reader(mut reader: impl io::Read) -> io::Result<Self> {
        let mut dwi_string = String::new();
        reader.read_to_string(&mut dwi_string)?;

        dwi_parser::parse(&dwi_string).map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))
    }
}
