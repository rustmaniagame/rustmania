use std::fs;
use std::slice;
use fraction::Fraction;

pub struct TimingData {
    notes: [Vec<i64>; 4],
}

#[derive(Debug)]
pub struct ChartMetadata {
    title: Option<String>,
    bpm: Option<f64>,
}

#[derive(Debug)]
pub struct NoteData {
    notes: Vec<Vec<(Fraction, NoteRow)>>,
    data: ChartMetadata,
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

impl ChartMetadata {
    pub fn new() -> Self {
        ChartMetadata {
            title: None,
            bpm: None,
        }
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

fn parse_main_block(contents: String) -> Vec<Vec<(Fraction, NoteRow)>> {
    let mut notes = Vec::new();
    let lines: Vec<_> = contents.lines().skip(5).collect();
    let measures = lines.split(|&x| x == ",");
    for measure in measures {
        notes.push(parse_measure(measure));
    }
    notes
}

fn split_once(contents: &str, letter: char) -> (&str,&str) {
    let mut split = contents.splitn(2, letter);
    let first = split.next().unwrap_or("");
    let second = split.next().unwrap_or("");
    (first,second)
}

fn parse_tag(tag: &str, contents: &str, data: &mut NoteData) {
    match tag {
        "NOTES" => data.notes = parse_main_block(contents.to_string()),
        _ => {},
    }
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
            parse_tag(tag, contents, &mut chart);
        }
        chart
    }
}