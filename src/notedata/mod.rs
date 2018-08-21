mod sm_parser;

use num_rational::Rational32;
use std::io;
use std::slice;

#[derive(Debug, PartialEq)]
pub struct ChartMetadata {
    pub title: Option<String>,
    pub offset: Option<f64>,
    pub bpm: Option<f64>,
}

#[derive(Debug, PartialEq)]
pub struct NoteData {
    notes: Vec<Vec<(Rational32, NoteRow)>>,
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
        let tags = chart_string.split(|x| x == '#').map(|x| split_once(x, ':'));
        for (tag, contents) in tags {
            sm_parser::parse_tag(tag, contents, &mut chart);
        }
        Ok(chart)
    }
    pub fn columns(&self) -> slice::Iter<Vec<(Rational32, NoteRow)>> {
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
    use notedata::NoteType;
    use num_rational::Ratio;
    use std::fs::File;

    #[test]
    fn split_once_correctly() {
        assert_eq!(split_once("left$right", '$'), (("left", "right")))
    }
    #[test]
    fn simple_file_parse() {
        assert_eq!(
            NoteData::from_sm(File::open("test_files/notes_test.sm").unwrap()).unwrap(),
            NoteData {
                notes: vec![
                    vec![
                        (
                            Rational32::new(0, 1),
                            NoteRow {
                                row: vec![(NoteType::Tap, 3)],
                            },
                        ),
                    ],
                    vec![],
                    vec![
                        (
                            Rational32::new(0, 1),
                            NoteRow {
                                row: vec![(NoteType::Mine, 1),(NoteType::Hold, 3)],
                            },
                        ),
                        (
                            Rational32::new(1, 2),
                            NoteRow {
                                row: vec![(NoteType::Fake, 0)],
                            },
                        ),
                    ],
                ],
                data: ChartMetadata::new(),
            }
        );
    }
}
