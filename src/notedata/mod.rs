mod sm_parser;

use num_rational::Rational32;
use std::io;
use std::slice;

#[derive(Debug, PartialEq)]
pub struct ChartData {
    notes: Vec<Vec<(Rational32, NoteRow)>>,
}

#[derive(Debug, PartialEq)]
pub struct ChartMetadata {
    pub title: Option<String>,
    pub music_path: Option<String>,
    pub offset: Option<f64>,
    pub bpms: Vec<(i32, Rational32, f64)>,
}

#[derive(Debug, PartialEq)]
pub struct NoteData {
    notes: Vec<ChartData>,
    pub data: ChartMetadata,
}

#[derive(Debug, PartialEq)]
pub struct NoteRow {
    row: Vec<(NoteType, usize)>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
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
            music_path: None,
            offset: None,
            bpms: Vec::new(),
        }
    }
}

impl ChartData {
    fn new(notes: Vec<Vec<(Rational32, NoteRow)>>) -> Self {
        ChartData { notes }
    }
    pub fn columns(&self) -> slice::Iter<Vec<(Rational32, NoteRow)>> {
        self.notes.iter()
    }
}

impl NoteData {
    pub fn from_sm<T>(mut simfile: T) -> Result<Self, io::Error>
    where
        T: io::Read,
    {
        let mut chart = NoteData {
            notes: Vec::new(),
            data: ChartMetadata::new(),
        };
        let mut chart_string = String::new();
        simfile.read_to_string(&mut chart_string)?;
        let (_, tags) = sm_parser::break_to_tags(&chart_string).unwrap();
        for (tag, contents) in tags.iter() {
            sm_parser::parse_tag(tag, contents, &mut chart);
        }
        Ok(chart)
    }
    pub fn charts(&self) -> slice::Iter<ChartData> {
        self.notes.iter()
    }
}

impl NoteRow {
    pub fn notes(&self) -> slice::Iter<(NoteType, usize)> {
        self.row.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notedata::NoteType;
    use std::fs::File;

    #[test]
    fn simple_file_parse() {
        assert_eq!(
            NoteData::from_sm(File::open("test_files/notes_test.sm").unwrap()).unwrap(),
            NoteData {
                notes: vec![ChartData::new(vec![
                    vec![(
                        Rational32::new(0, 1),
                        NoteRow {
                            row: vec![(NoteType::Tap, 3)],
                        },
                    ),],
                    vec![],
                    vec![
                        (
                            Rational32::new(0, 1),
                            NoteRow {
                                row: vec![(NoteType::Mine, 1), (NoteType::Hold, 3)],
                            },
                        ),
                        (
                            Rational32::new(1, 2),
                            NoteRow {
                                row: vec![(NoteType::Fake, 0)],
                            },
                        ),
                    ],
                ])],
                data: ChartMetadata::new(),
            }
        );
    }
}
