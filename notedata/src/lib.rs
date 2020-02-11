#![cfg_attr(not(feature = "std"), no_std)]
#![warn(
    clippy::cast_lossless,
    clippy::checked_conversions,
    clippy::copy_iterator,
    clippy::default_trait_access,
    clippy::doc_markdown,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::expl_impl_clone_on_copy,
    clippy::explicit_into_iter_loop,
    clippy::explicit_iter_loop,
    clippy::filter_map,
    clippy::filter_map_next,
    clippy::find_map,
    clippy::if_not_else,
    clippy::inline_always,
    clippy::items_after_statements,
    clippy::large_digit_groups,
    clippy::large_stack_arrays,
    clippy::map_flatten,
    clippy::match_same_arms,
    clippy::maybe_infinite_iter,
    clippy::mut_mut,
    clippy::needless_continue,
    clippy::needless_pass_by_value,
    clippy::non_ascii_literal,
    clippy::option_map_unwrap_or,
    clippy::option_map_unwrap_or_else,
    clippy::pub_enum_variant_names,
    clippy::redundant_closure_for_method_calls,
    clippy::replace_consts,
    clippy::result_map_unwrap_or_else,
    clippy::same_functions_in_if_condition,
    clippy::shadow_unrelated,
    clippy::similar_names,
    clippy::single_match_else,
    clippy::too_many_lines,
    clippy::type_repetition_in_bounds,
    clippy::unicode_not_nfc,
    clippy::unseparated_literal_suffix,
    clippy::unused_self,
    clippy::used_underscore_binding
)]

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;

mod lib {
    #[cfg(not(feature = "std"))]
    pub use alloc::{
        borrow::ToOwned,
        fmt,
        string::{String, ToString},
        vec::Vec,
    };
    #[cfg(not(feature = "std"))]
    pub trait Error: fmt::Debug + fmt::Display {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            None
        }
    }

    #[cfg(feature = "std")]
    pub use std::{
        borrow::ToOwned,
        error::Error,
        fmt, io,
        string::{String, ToString},
        vec::Vec,
    };
}

#[cfg(feature = "std")]
use lib::io;
use lib::{String, Vec};

pub use num_rational::Rational32 as Fraction;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

mod dwi_parser;
mod error;
mod parser_generic;
mod sm_parser;
mod sm_writer;
pub use error::ParseError;

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

    pub fn from_sm_string(string: &str) -> Result<Self, ParseError> {
        Ok(sm_parser::parse(&string)?)
    }

    #[cfg(feature = "std")]
    pub fn from_sm_reader(mut reader: impl io::Read) -> Result<Self, ParseError> {
        let mut sm_string = String::new();
        reader.read_to_string(&mut sm_string)?;
        Ok(sm_parser::parse(&sm_string)?)
    }

    #[must_use]
    pub fn to_sm_string(&self) -> String {
        sm_writer::write_sm(&self)
    }

    #[cfg(feature = "std")]
    pub fn to_sm_writer(&self, mut writer: impl io::Write) -> io::Result<()> {
        writer.write_all(&self.to_sm_string().into_bytes())
    }

    pub fn from_dwi_string(string: &str) -> Result<Self, ParseError> {
        Ok(dwi_parser::parse(&string)?)
    }

    #[cfg(feature = "std")]
    pub fn from_dwi_reader(mut reader: impl io::Read) -> Result<Self, ParseError> {
        let mut dwi_string = String::new();
        reader.read_to_string(&mut dwi_string)?;
        Ok(dwi_parser::parse(&dwi_string)?)
    }
}
