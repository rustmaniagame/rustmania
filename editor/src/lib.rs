use notedata::{BeatPair, Fraction, Note, NoteData, NoteRow, NoteType};
use std::collections::BTreeMap;
use std::convert::TryFrom;

pub struct Editor {
    chart: ChartEditor,
    current_beat: (i32, Fraction),
    pub snap: i32,
}

#[derive(Default)]
pub struct ChartEditor {
    bpms: BTreeMap<(i32, Fraction), (f64, f64)>,
    notes: BTreeMap<(usize, Fraction), BTreeMap<usize, NoteType>>,
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

impl Editor {
    pub fn new() -> Self {
        Self {
            chart: ChartEditor::default(),
            current_beat: (0, Fraction::new(0, 1)),
            snap: 4,
        }
    }
    pub fn toggle_note(&mut self, column: usize, note_type: NoteType) {
        self.chart.toggle_note(
            usize::try_from(self.current_beat.0).unwrap_or(0),
            self.current_beat.1,
            column,
            note_type,
        );
    }
    pub fn add_bpm(&mut self, bpm: f64) {
        self.chart
            .add_bpm(self.current_beat.0, self.current_beat.1, bpm);
    }
    pub fn remove_bpm(&mut self) {
        self.chart
            .remove_bpm(self.current_beat.0, self.current_beat.1);
    }
    fn normalize_measure(&mut self) {
        self.current_beat = (
            self.current_beat.0 + *self.current_beat.1.floor().numer(),
            if self.current_beat.1 >= Fraction::new(0, 1) {
                self.current_beat.1.fract()
            } else {
                Fraction::new(1, 1) + self.current_beat.1.fract()
            },
        );
        if self.current_beat.0 < 0 {
            self.current_beat = (0, Fraction::new(0, 1));
        }
    }
    pub fn next_snap(&mut self) {
        self.current_beat.1 = Fraction::new(
            (self.current_beat.1 * Fraction::new(self.snap, 1))
                .floor()
                .numer()
                + 1,
            self.snap,
        );
        self.normalize_measure();
    }
    pub fn previous_snap(&mut self) {
        self.current_beat.1 = Fraction::new(
            (self.current_beat.1 * Fraction::new(self.snap, 1))
                .ceil()
                .numer()
                - 1,
            self.snap,
        );
        self.normalize_measure();
    }
    pub fn set_beat(&mut self, measure: i32, beat: Fraction) {
        self.current_beat = (measure, beat);
        self.normalize_measure();
    }
    pub fn get_beat(&self) -> (i32, Fraction) {
        self.current_beat
    }
    pub fn get_noterow(&self) -> Vec<Note> {
        self.chart
            .get_noterow(self.current_beat.0 as usize, self.current_beat.1)
    }
    pub fn export(&self) -> Result<NoteData, ()> {
        self.chart.export()
    }
}

impl ChartEditor {
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
    pub fn remove_bpm(&mut self, measure: i32, beat: Fraction) -> Option<(f64, f64)> {
        self.bpms.remove(&(measure, beat))
    }
    pub fn export(&self) -> Result<NoteData, ()> {
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
    pub fn get_noterow(&self, measure: usize, beat: Fraction) -> NoteRow {
        self.notes
            .get(&(measure, beat))
            .map(|row| row.iter().map(|(m, b)| Note::new(*b, *m)).collect())
            .unwrap_or_else(|| vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_notedata() {
        let mut edit = ChartEditor::default();
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
