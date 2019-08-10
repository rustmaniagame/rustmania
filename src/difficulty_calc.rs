use crate::{
    notedata::NoteType,
    timingdata::{CalcInfo, Judgement, TimingData},
};
use std::cmp::Ordering;

pub fn rate_chart(notes: &TimingData<CalcInfo>, target: f64) -> f64 {
    let mut difficulty = notes
        .notes
        .iter()
        .map(|column| {
            column
                .notes
                .iter()
                .enumerate()
                .filter(|(_, CalcInfo(_, x))| *x == NoteType::Tap || *x == NoteType::Hold)
                .map(|(index, CalcInfo(base_time, _))| {
                    column
                        .notes
                        .iter()
                        .take(index)
                        .filter(|x| x.1 == NoteType::Tap || x.1 == NoteType::Hold)
                        .map(|CalcInfo(other_time, _)| {
                            1_000_000.0 / (base_time - other_time).pow(2) as f64
                        })
                        .sum::<f64>()
                })
                .collect::<Vec<_>>()
        })
        .flatten()
        .collect::<Vec<_>>();

    difficulty.sort_by(|a, b| b.partial_cmp(a).unwrap_or(Ordering::Less));

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
