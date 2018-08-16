mod sm_parse;

use fraction::Fraction;
use std::fs;
use std::slice;

#[derive(Debug)]
pub struct ChartMetadata {
    pub title: Option<String>,
    pub offset: Option<f64>,
    pub bpm: Option<f64>,
}

#[derive(Debug)]
pub struct NoteData {
    notes: Vec<Vec<(Fraction, NoteRow)>>,
    pub data: ChartMetadata,
}

#[derive(Debug)]
pub struct NoteRow {
    row: Vec<(NoteType, usize)>,
}

#[derive(Debug)]
pub enum NoteType {
    Tap,
    Hold,
    Roll,
    Mine,
    Lift,
    Fake,
}

impl ChartMetadata {
    pub fn new() -> Self {
        ChartMetadata {
            title: None,
            offset: None,
            bpm: None,
        }
    }
}

fn split_once(contents: &str, letter: char) -> (&str, &str) {
    let mut split = contents.splitn(2, letter);
    let first = split.next().unwrap_or("");
    let second = split.next().unwrap_or("");
    (first, second)
}

impl NoteData {
    pub fn from_sm() -> Self {
        let mut chart = NoteData {
            notes: Vec::new(),
            data: ChartMetadata::new(),
        };
        let simfile = fs::read_to_string("resources/barebones.sm").unwrap();
        let tags = simfile.split(|x| x == '#').map(|x| split_once(x, ':'));
        for (tag, contents) in tags {
            sm_parse::parse_tag(tag, contents, &mut chart);
        }
        chart
    }
    pub fn columns(&self) -> slice::Iter<Vec<(Fraction, NoteRow)>> {
        self.notes.iter()
    }
}

impl NoteRow {
    pub fn notes(&self) -> slice::Iter<(NoteType, usize)> {
        self.row.iter()
    }
}
