use std::slice;
use notedata::NoteData;

pub struct TimingData {
    notes: [Vec<i64>; 4],
}

impl TimingData {
    pub fn from_notedata(data: NoteData) -> Self {
        let bpm = data.data.bpm.unwrap_or(6.0);
        let offset= data.data.offset.unwrap_or(0.0) * 1000.0;
        let mut output = [Vec::new(),Vec::new(),Vec::new(),Vec::new()];
        for ( measure_index , measure) in data.columns().enumerate() {
            let measure_time = (measure_index * 240_000) as f64 / bpm + offset;
            for (inner_time, row) in measure.iter() {
                let (num_beats, division) = inner_time.contents();
                let row_time = measure_time + (240_000 * num_beats / division) as f64 / bpm;
                for (note, column_index) in row.row.iter() {
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