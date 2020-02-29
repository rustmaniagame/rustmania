use crate::timingdata::{CalcInfo, Judgement, TimingData};
use notedata::NoteType;
use std::cmp::Ordering;

pub fn rate_chart(notes: &TimingData<CalcInfo>, target: f64) -> f64 {
    let mut difficulty = notes
        .notes
        .iter()
        .flat_map(|column| {
            column
                .notes
                .iter()
                .enumerate()
                .filter_map(|(index, CalcInfo(base_time, note_type))| match *note_type {
                    NoteType::Tap | NoteType::Hold => Some(
                        column
                            .notes
                            .iter()
                            .take(index)
                            .filter_map(|CalcInfo(other_time, note_type)| match *note_type {
                                NoteType::Tap | NoteType::Hold => {
                                    Some(1_000_000.0 / (base_time - other_time).pow(2) as f64)
                                }
                                _ => None,
                            })
                            .sum::<f64>(),
                    ),
                    _ => None,
                })
                .collect::<Vec<_>>()
        })
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
    lower * 3.15 * jack_scaler(notes)
}

pub fn jack_scaler(notes: &TimingData<CalcInfo>) -> f64 {
    let mut out = 0.0;
    for column in &notes.notes {
        let mut start_iter = column.notes.iter();
        while let Some(start_note) = start_iter.next() {
            let mut total_intermediate = 0;
            let mut has_missed = false;
            for end_note in start_iter.clone() {
                let mut middle_iter = start_iter.clone();
                //check if notes between start and end can be hit as a jack
                for i in 0..total_intermediate {
                    if let Some(middle_note) = middle_iter.next() {
                        //check if the middle note would be a CB
                        if start_note.0
                            + (i + 1) * (end_note.0 - start_note.0) / (total_intermediate + 1)
                            - middle_note.0.abs()
                            > 90
                        {
                            has_missed = true;
                            break;
                        };
                    }
                }
                if has_missed {
                    break;
                }
                total_intermediate += 1;
            }
            out += 0.98_f64.powf(total_intermediate as f64);
        }
    }
    out / (notes
        .notes
        .iter()
        .map(|column| column.notes.len())
        .sum::<usize>()) as f64
}
