use ggez::graphics;
use notedata::NoteData;
use notedata::NoteType;
use num_rational::Rational32;
use std::slice;

fn value(fraction: Rational32) -> f64 {
    *fraction.numer() as f64 / *fraction.denom() as f64
}

#[derive(Debug)]
pub struct TimingData<T>
where
    T: TimingInfo,
{
    notes: [Vec<T>; 4],
}

pub trait TimingInfo {}

#[derive(Copy, Clone)]
pub struct GameplayInfo(pub i64, pub graphics::Rect);

impl TimingInfo for GameplayInfo {}

#[derive(Copy, Clone)]
pub struct OffsetInfo(i64);

impl TimingInfo for OffsetInfo {}

impl TimingData<GameplayInfo> {
    pub fn from_notedata<U>(data: NoteData, sprite_finder: U) -> Self
    where
        U: Fn(usize, f64, Rational32, NoteType, usize) -> graphics::Rect,
    {
        let bpm = data.data.bpm.unwrap_or(6.0);
        let offset = data.data.offset.unwrap_or(0.0) * 1000.0;
        let mut output = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        for (measure_index, measure) in data.columns().enumerate() {
            let measure_time = (measure_index * 240_000) as f64 / bpm + offset;
            for (inner_time, row) in measure.iter() {
                let row_time = measure_time + (240_000.0 * value(*inner_time)) / bpm;
                for (note, column_index) in row.notes() {
                    let sprite = sprite_finder(
                        measure_index,
                        measure_time,
                        *inner_time,
                        *note,
                        *column_index,
                    );
                    output[*column_index].push(GameplayInfo(row_time as i64, sprite));
                }
            }
        }
        TimingData { notes: output }
    }
}
impl<T> TimingData<T>
where
    T: TimingInfo,
{
    pub fn columns(&self) -> slice::Iter<Vec<T>> {
        self.notes.iter()
    }
}
