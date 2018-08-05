use std::fs;

pub struct TimingData {
    pub notes: [Vec<i64>; 4],
}

pub struct _Notedata();

impl TimingData {
    pub fn from_sm() -> Self {
        let mut notes = [Vec::new(),Vec::new(),Vec::new(),Vec::new()];
        let bpm = 128;
        let offset = 1000;
        let simfile = fs::read_to_string("resources/barebones.sm").unwrap();
        let lines: Vec<_> = simfile.lines().collect();
        let measures = lines.split(|&x| x == ",");
        for (index, measure) in measures.enumerate() {
            let measure_time = index * 240_000 / bpm;
            let division = measure.len();
            for (subindex, beat) in measure.iter().enumerate() {
                let time = measure_time + (subindex * 240_000 / bpm / division) + offset;
                for (column, notetype) in beat.chars().enumerate() {
                    if notetype != '0' {
                        notes[column].push(time as i64);
                    }
                }
            }
        }
        TimingData {
            notes,
        }
    }
}