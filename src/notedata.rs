use std::fs;
use std::slice;
use fraction::Fraction;

pub struct TimingData {
    notes: [Vec<i64>; 4],
}

#[derive(Debug)]
pub struct NoteData {
    notes: Vec<Vec<(Fraction, NoteRow)>>,
}

#[derive(Debug)]
pub struct NoteRow {
    row: Vec<(NoteType,usize)>,
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

impl TimingData {
    pub fn from_notedata(data: NoteData, bpm: f64, offset: f64) -> Self {
        let mut output = [Vec::new(),Vec::new(),Vec::new(),Vec::new()];
        for ( measure_index , measure) in data.notes.iter().enumerate() {
            let measure_time = (measure_index * 240_000) as f64 / bpm + offset;
            for (inner_time, data) in measure.iter() {
                let (num_beats, division) = inner_time.contents();
                let row_time = measure_time + (240_000 * num_beats / division) as f64 / bpm;
                for (note, column_index) in data.row.iter() {
                    output[*column_index].push(row_time as i64);
                }
            }
        }
        TimingData {
            notes: output,
        }
    }
    pub fn columns(&self) -> slice::Iter<Vec<i64>> {
        self.notes.iter()
    }
}


fn parse_measure(measure: &[&str]) -> Vec<(Fraction,NoteRow)> {
    let mut output = Vec::new();
    let division = measure.len();
    for (subindex, beat) in measure.iter().enumerate() {
        output.push((Fraction::new(subindex as i64,division as u64).unwrap(),parse_line(beat)));
    }
    output
}

fn parse_line(contents: &&str) -> NoteRow {
    let mut row = Vec::new();
    contents.chars().enumerate().for_each(|(index, character)| {
        if let Some(note) = char_to_notetype(character) {
            row.push((note,index));
        }
    });
    NoteRow {
        row,
    }
}

fn char_to_notetype(character: char) -> Option<NoteType> {
    match character {
        '0' => None,
        '1' => Some(NoteType::Tap),
        '2' => Some(NoteType::Hold),
        '4' => Some(NoteType::Roll),
        'M' => Some(NoteType::Mine),
        'L' => Some(NoteType::Lift),
        'F' => Some(NoteType::Fake),
        _ => None
    }
}

impl NoteData {
    pub fn from_sm() -> Self {
        let mut notes = Vec::new();
        let simfile = fs::read_to_string("resources/barebones.sm").unwrap();
        let lines: Vec<_> = simfile.lines().collect();
        let measures = lines.split(|&x| x == ",");
        for measure in measures {
            notes.push(parse_measure(measure));
        }
        NoteData {
            notes,
        }
    }
}