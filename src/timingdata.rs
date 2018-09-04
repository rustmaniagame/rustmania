use ggez::graphics;
use notedata::NoteData;
use notedata::NoteType;
use num_rational::Rational32;
use std::slice;

fn value(fraction: Rational32) -> f64 {
    *fraction.numer() as f64 / *fraction.denom() as f64
}

#[derive(Debug, PartialEq)]
pub struct TimingData<T>
where
    T: TimingInfo,
{
    notes: [Vec<T>; 4],
}

pub trait TimingInfo {}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GameplayInfo(pub i64, pub graphics::Rect);

impl TimingInfo for GameplayInfo {}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct OffsetInfo(pub i64);

impl TimingInfo for OffsetInfo {}

impl OffsetInfo {
    fn wife(&self, ts: f64) -> f64 {
        let maxms = self.0 as f64;
        let avedeviation = 95.0 * ts;
        let mut y = 1.0 - 2.0_f64.powf(-1.0 * maxms * maxms / (avedeviation * avedeviation));
        y *= y;
        (10.0) * (1.0 - y) - 8.0
    }
}

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
    pub fn add(&mut self, offset: T, column: usize) {
        self.notes[column].push(offset);
    }
    pub fn columns(&self) -> slice::Iter<Vec<T>> {
        self.notes.iter()
    }
    pub fn new() -> Self {
        TimingData {
            notes: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
        }
    }
}
impl TimingData<OffsetInfo> {
    pub fn calculate_score(&self) -> f64 {
        let max_points =
            (self.notes[0].len() + self.notes[1].len() + self.notes[2].len() + self.notes[3].len())
                as f64;
        let mut current_points = 0.0;
        for column in self.columns() {
            for offset in column {
                current_points += offset.wife(1.0);
            }
        }
        current_points / max_points
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wife_symmetry() {
        for offset in 0..180 {
            let early = OffsetInfo(-offset);
            let late = OffsetInfo(offset);
            assert_eq!(early.wife(1.0), late.wife(1.0));
        }
    }
    #[test]
    fn wife_peak() {
        assert_eq!(OffsetInfo(0).wife(1.0), 2.0);
        assert_eq!(OffsetInfo(0).wife(0.5), 2.0);
        assert_eq!(OffsetInfo(0).wife(2.0), 2.0);
    }
    #[test]
    fn wife_decreasing() {
        for offset in 0..179 {
            assert!(OffsetInfo(offset).wife(1.0) > OffsetInfo(offset + 1).wife(1.0));
            assert!(OffsetInfo(offset).wife(0.5) > OffsetInfo(offset + 1).wife(0.5));
            assert!(OffsetInfo(offset).wife(2.0) > OffsetInfo(offset + 1).wife(2.0));
        }
    }
}
