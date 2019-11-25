mod sm_parser;

pub use num_rational::Rational32 as Fraction;
use serde_derive::{Deserialize, Serialize};
use std::io;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BeatPair<T> {
    pub beat: i32,
    pub sub_beat: Fraction,
    pub value: T,
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum NoteType {
    Tap,
    Hold,
    Roll,
    Mine,
    Lift,
    Fake,
    HoldEnd,
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum DisplayBpm {
    Range(f64, f64),
    Static(f64),
    Random,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Note {
    pub note_type: NoteType,
    pub column: usize,
}

type NoteRow = Vec<Note>;
pub type Measure = Vec<(NoteRow, Fraction)>;
pub type Chart = Vec<Measure>;

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct NoteData {
    pub charts: Vec<Chart>,
    pub meta: ChartMetadata,
}

impl<T> BeatPair<T> {
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
    fn new(note_type: NoteType, column: usize) -> Self {
        Self { note_type, column }
    }
}

impl NoteData {
    fn new() -> Self {
        Self {
            charts: vec![],
            meta: ChartMetadata::default(),
        }
    }

    pub fn from_sm<T>(mut simfile: T) -> Result<Self, io::Error>
        where
            T: io::Read,
    {
        let mut chart_string = String::new();
        simfile.read_to_string(&mut chart_string)?;

        sm_parser::parse(&chart_string).map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))
    }
}
