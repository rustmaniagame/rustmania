use fraction::value;
use fraction::Fraction;
use ggez::graphics;
use notedata::NoteData;
use notedata::NoteType;
use std::slice;

pub struct TimingData<'a> {
    notes: [Vec<(i64, &'a graphics::Image)>; 4],
}

impl<'a> TimingData<'a> {
    pub fn from_notedata<T>(
        data: NoteData,
        sprite_list: &'a Vec<graphics::Image>,
        sprite_finder: T,
    ) -> Self
    where
        T: Fn(usize, f64, Fraction, NoteType, usize) -> usize,
    {
        let bpm = data.data.bpm.unwrap_or(6.0);
        let offset = data.data.offset.unwrap_or(0.0) * 1000.0;
        let mut output = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        for (measure_index, measure) in data.columns().enumerate() {
            let measure_time = (measure_index * 240_000) as f64 / bpm + offset;
            for (inner_time, row) in measure.iter() {
                let (_, division) = (*inner_time * 4).contents();
                let row_time = measure_time + (240_000.0 * value(*inner_time)) / bpm;
                for (note, column_index) in row.notes() {
                    let sprite = sprite_list
                        .get(sprite_finder(measure_index, measure_time, *inner_time, *note, *column_index) )
                        .unwrap_or(&sprite_list[0]);
                    output[*column_index].push((row_time as i64, sprite));
                }
            }
        }
        TimingData { notes: output }
    }
    pub fn columns(&self) -> slice::Iter<Vec<(i64, &'a graphics::Image)>> {
        self.notes.iter()
    }
}
