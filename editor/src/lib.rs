use ggez::event::{EventHandler, KeyCode, KeyMods};
use ggez::graphics::spritebatch::SpriteBatch;
use ggez::{graphics, Context, GameError};
use notedata::timingdata::{GameplayInfo, Rectangle, TimingColumn};
use notedata::{BeatPair, Fraction, Note, NoteData, NoteRow, NoteType, NOTEFIELD_SIZE};
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::path::PathBuf;
use utils::notefield::player_config::{NoteLayout, NoteSkin, PlayerOptions};
use utils::notefield::ColumnInfo;

pub struct Editor {
    chart: ChartEditor,
    current_beat: (i32, Fraction),
    pub snap: i32,
    pub layout: NoteLayout,
    pub column_info: [ColumnInfo; NOTEFIELD_SIZE],
    pub batches: Vec<SpriteBatch>,
    pub zoom: Fraction,
}

#[derive(Default)]
pub struct ChartEditor {
    bpms: BTreeMap<(i32, Fraction), (f64, f64)>,
    notes: BTreeMap<(usize, Fraction), BTreeMap<usize, NoteType>>,
}

/*impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}*/

impl Editor {
    pub fn new(
        noteskin: PathBuf,
        screen_height: i64,
        options: PlayerOptions,
        ctx: &mut Context,
    ) -> Self {
        let layout = NoteLayout::new(
            &NoteSkin::new(&noteskin, ctx).unwrap(),
            screen_height,
            options,
        );
        let batches = vec![
            SpriteBatch::new(layout.sprites.hold_end.clone()),
            SpriteBatch::new(layout.sprites.hold_body.clone()),
            SpriteBatch::new(layout.sprites.arrows.clone()),
            SpriteBatch::new(layout.sprites.mine.clone()),
        ];
        Self {
            chart: ChartEditor::default(),
            current_beat: (0, Fraction::new(0, 1)),
            snap: 4,
            layout,
            column_info: [
                ColumnInfo::from(TimingColumn::new()),
                ColumnInfo::from(TimingColumn::new()),
                ColumnInfo::from(TimingColumn::new()),
                ColumnInfo::from(TimingColumn::new()),
            ],
            batches,
            zoom: Fraction::new(1, 1),
        }
    }
    pub fn redraw_batch(&mut self) {
        self.batches.iter_mut().for_each(SpriteBatch::clear);
        for column_index in 0..NOTEFIELD_SIZE {
            let (draw_start, draw_end) = self.column_info[column_index].on_screen;
            self.layout.add_column_of_notes(
                &self.column_info[column_index].notes.notes[draw_start..draw_end],
                column_index,
                &mut self.batches,
            );
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
    pub fn get_bpm(&self) -> Option<f64> {
        self.chart.get_bpm(self.current_beat.0, self.current_beat.1)
    }
    pub fn get_noterow(&self) -> Vec<Note> {
        self.chart
            .get_noterow(self.current_beat.0 as usize, self.current_beat.1)
    }
    pub fn export(&self) -> Result<NoteData, ()> {
        self.chart.export()
    }
}

fn handle_keypress(editor: &mut Editor, code: KeyCode) {
    match code {
        KeyCode::Left => {
            if editor.snap > 1 {
                editor.snap -= 1;
                println!("snap changed to: {}", editor.snap);
            }
        }
        KeyCode::Right => {
            editor.snap += 1;
            println!("snap changed to: {}", editor.snap);
        }
        KeyCode::Up => {
            if editor.layout.scroll_speed > 0.0 {
                editor.previous_snap();
            } else {
                editor.next_snap();
            }
            let (measure, beat) = editor.get_beat();
            println!(
                "measure: {} beat: {} row: {:?}",
                measure,
                beat,
                editor.get_noterow()
            );
        }
        KeyCode::Down => {
            if editor.layout.scroll_speed > 0.0 {
                editor.next_snap();
            } else {
                editor.previous_snap();
            }
            let (measure, beat) = editor.get_beat();
            println!(
                "measure: {} beat: {} row: {:?}",
                measure,
                beat,
                editor.get_noterow()
            );
        }
        KeyCode::Key1 => {
            editor.toggle_note(0, NoteType::Tap);
            let (measure, beat) = editor.get_beat();
            println!(
                "measure: {} beat: {} row: {:?}",
                measure,
                beat,
                editor.get_noterow()
            );
        }
        KeyCode::Key2 => {
            editor.toggle_note(1, NoteType::Tap);
            let (measure, beat) = editor.get_beat();
            println!(
                "measure: {} beat: {} row: {:?}",
                measure,
                beat,
                editor.get_noterow()
            );
        }
        KeyCode::Key3 => {
            editor.toggle_note(2, NoteType::Tap);
            let (measure, beat) = editor.get_beat();
            println!(
                "measure: {} beat: {} row: {:?}",
                measure,
                beat,
                editor.get_noterow()
            );
        }
        KeyCode::Key4 => {
            editor.toggle_note(3, NoteType::Tap);
            let (measure, beat) = editor.get_beat();
            println!(
                "measure: {} beat: {} row: {:?}",
                measure,
                beat,
                editor.get_noterow()
            );
        }
        KeyCode::N => {
            let bpm = editor.get_bpm().unwrap_or(120.0) + 10.0;
            editor.chart.bpms.insert(editor.current_beat, (bpm, 0.0));
        }
        KeyCode::M => {
            let bpm = editor.get_bpm().unwrap_or(120.0) - 10.0;
            editor.chart.bpms.insert(editor.current_beat, (bpm, 0.0));
        }
        KeyCode::Return => {
            let simfile_string = editor
                .export()
                .map(|data| data.to_sm_string())
                .unwrap_or_else(|_| String::from("Failed"));
            println!("{}", simfile_string);
        }
        KeyCode::Add => editor.zoom *= 2,
        KeyCode::Subtract => editor.zoom /= 2,
        _ => {}
    }
}

impl EventHandler for Editor {
    fn update(&mut self, _ctx: &mut Context) -> Result<(), GameError> {
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        graphics::clear(ctx, graphics::BLACK);
        let scroll_const = 1000.0 * self.layout.scroll_speed * *self.zoom.numer() as f32
            / *self.zoom.denom() as f32;
        let time = ((self.current_beat.0 as f32
            + (*self.current_beat.1.numer() as f32 / *self.current_beat.1.denom() as f32))
            * scroll_const) as i64;
        for column_index in 0..4 {
            self.column_info[column_index].update_on_screen(&self.layout, time, 600);
        }
        let target_parameter =
            graphics::DrawParam::new().dest([0.0, (self.layout.delta_to_offset(time))]);
        self.redraw_batch();

        self.column_info = [
            ColumnInfo::from(TimingColumn::new()),
            ColumnInfo::from(TimingColumn::new()),
            ColumnInfo::from(TimingColumn::new()),
            ColumnInfo::from(TimingColumn::new()),
        ];
        for ((measure, beat), contents) in self.chart.notes.iter() {
            for (&index, &note) in contents {
                self.column_info[index].notes.notes.push(GameplayInfo(
                    (-1.0
                        * (*measure as f32 + (*beat.numer() as f32 / *beat.denom() as f32))
                        * scroll_const) as i64,
                    Rectangle {
                        x: 0.0,
                        y: 0.0,
                        w: 1.0,
                        h: 0.125,
                    },
                    note,
                ))
            }
        }
        self.layout.draw_receptors(ctx)?;
        for batch in &self.batches {
            graphics::draw(ctx, batch, target_parameter)?;
        }
        graphics::present(ctx)
    }
    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymod: KeyMods,
        _repeat: bool,
    ) {
        handle_keypress(self, keycode);
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
    pub fn get_bpm(&self, measure: i32, beat: Fraction) -> Option<f64> {
        self.bpms
            .range(..=(measure, beat))
            .rev()
            .next()
            .map(|(_, (x, _))| *x)
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
