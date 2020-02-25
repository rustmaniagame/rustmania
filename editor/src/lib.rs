use notedata::{BeatPair, Fraction, Note, NoteData, NoteType};
use std::collections::BTreeMap;

#[derive(Default)]
pub struct Editor {
    bpms: BTreeMap<(i32, Fraction), (f64, f64)>,
    notes: BTreeMap<(usize, Fraction), BTreeMap<usize, NoteType>>,
}

impl Editor {
    pub fn toggle_note(
        &mut self,
        measure: usize,
        beat: Fraction,
        column: usize,
        note_type: NoteType,
    ) {
        if let Some(noterow) = self.notes.get_mut(&(measure, beat)) {
            if noterow.get(&column) == Some(&note_type) {
                noterow.remove(&column);
            } else {
                noterow.insert(column, note_type);
            }
        } else {
            let mut row = BTreeMap::new();
            row.insert(column, note_type);
            self.notes.insert((measure, beat), row);
        }
    }
    pub fn add_bpm(&mut self, measure: i32, beat: Fraction, bpm: f64) {
        self.bpms.insert((measure, beat), (bpm, 0.0));
    }
    pub fn export(&mut self) -> Result<NoteData, ()> {
        let mut data = NoteData::new();
        data.structure.bpms = self
            .bpms
            .iter()
            .map(|((measure, beat), (bpm, _time))| BeatPair {
                beat: *measure,
                sub_beat: *beat,
                value: *bpm,
            })
            .collect();
        let mut out = vec![];
        for ((measure, beat), map) in self.notes.iter() {
            while out.len() <= *measure {
                out.push(vec![]);
            }
            if let Some(current) = out.last_mut() {
                current.push((
                    map.iter()
                        .map(|(&column, &note_type)| Note { note_type, column })
                        .collect::<Vec<_>>(),
                    *beat,
                ));
            }
        }
        data.charts = vec![out];
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_notedata() {
        let mut edit = Editor::default();
        edit.add_bpm(3, Fraction::new(1, 2), 180.0);
        edit.add_bpm(0, Fraction::new(0, 1), 120.0);
        edit.toggle_note(1, Fraction::new(1, 2), 3, NoteType::Tap);
        edit.toggle_note(3, Fraction::new(2, 3), 2, NoteType::Tap);
        edit.toggle_note(3, Fraction::new(1, 4), 1, NoteType::Tap);
        edit.toggle_note(1, Fraction::new(1, 2), 0, NoteType::Tap);
        edit.toggle_note(0, Fraction::new(1, 2), 3, NoteType::Tap);
        edit.toggle_note(3, Fraction::new(2, 3), 3, NoteType::Tap);
        edit.toggle_note(1, Fraction::new(1, 2), 0, NoteType::Tap);
        edit.toggle_note(1, Fraction::new(1, 2), 0, NoteType::Mine);
        let mut cool = NoteData::new();
        cool.charts = vec![vec![
            vec![(vec![Note::new(NoteType::Tap, 3)], Fraction::new(1, 2))],
            vec![(
                vec![Note::new(NoteType::Mine, 0), Note::new(NoteType::Tap, 3)],
                Fraction::new(1, 2),
            )],
            vec![],
            vec![
                (vec![Note::new(NoteType::Tap, 1)], Fraction::new(1, 4)),
                (
                    vec![Note::new(NoteType::Tap, 2), Note::new(NoteType::Tap, 3)],
                    Fraction::new(2, 3),
                ),
            ],
        ]];
        cool.structure.bpms = vec![
            BeatPair {
                beat: 0,
                sub_beat: Fraction::new(0, 1),
                value: 120.0,
            },
            BeatPair {
                beat: 3,
                sub_beat: Fraction::new(1, 2),
                value: 180.0,
            },
        ];
        assert_eq!(edit.export(), Ok(cool))
    }
}
