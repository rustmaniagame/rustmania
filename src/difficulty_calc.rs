use crate::{
    notedata::NoteType,
    timingdata::{CalcInfo, Judgement, TimingData},
};
use std::cmp::Ordering;

pub fn rate_chart(notes: &TimingData<CalcInfo>, target: f64) -> f64 {
    let mut difficulty = vec![];
    for column in notes.notes.iter() {
        for (index, CalcInfo(base_time, base_type)) in column.notes.iter().enumerate() {
            if *base_type == NoteType::Tap || *base_type == NoteType::Hold {
                let mut note_difficulty = 0.0;
                for CalcInfo(other_time, other_type) in column.notes.iter().take(index) {
                    if *other_type == NoteType::Tap || *other_type == NoteType::Hold {
                        note_difficulty += 1_000_000.0
                            / ((base_time - other_time) * (base_time - other_time)) as f64;
                    }
                }
                difficulty.push(note_difficulty);
            }
        }
    }

    difficulty[..].sort_by(|a, b| b.partial_cmp(a).unwrap_or(Ordering::Less));

    let mut lower = 0.0;
    let mut upper = 100.0;
    while upper - lower > 0.001 {
        let mid = (lower + upper) / 2.0;
        if difficulty
            .iter()
            .map(|x| Judgement::Hit((40.0 * x / (mid * mid)) as i64).wife(1.0))
            .sum::<f64>()
            / difficulty.len() as f64
            > target
        {
            upper = mid;
        } else {
            lower = mid;
        }
    }
    //Scale to approximate mina's ratings
    lower * 3.0
}
