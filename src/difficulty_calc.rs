use crate::{
    notedata::NoteType,
    timingdata::{CalcInfo, TimingData},
};

pub fn rate_chart(notes: &TimingData<CalcInfo>) -> f64 {
    let mut difficulty = 0.0;
    for column in notes.notes.iter() {
        for start_note in column
            .notes
            .iter()
            .filter(|note| note.1 == NoteType::Tap || note.1 == NoteType::Hold)
        {
            for end_note in column
                .notes
                .iter()
                .filter(|note| note.1 == NoteType::Tap || note.1 == NoteType::Hold)
            {
                if start_note.0 < end_note.0 {
                    difficulty +=
                        1.0 / ((end_note.0 - start_note.0) * (end_note.0 - start_note.0)) as f64;
                }
            }
        }
    }
    difficulty /= notes
        .notes
        .iter()
        .map(|column| column.notes.len())
        .sum::<usize>() as f64;
    difficulty *= 1_000_000.0;
    difficulty
}
