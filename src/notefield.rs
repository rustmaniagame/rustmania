extern crate ggez;

use crate::notedata::NoteType;
use crate::player_config;
use crate::player_config::NoteLayout;
use crate::screen::Element;
use crate::timingdata::{GameplayInfo, Judgement, TimingColumn, TimingData};
use ggez::graphics;
use ggez::graphics::spritebatch::SpriteBatch;
use rlua::UserData;
use std::time::Instant;

#[derive(PartialEq, Debug)]
pub struct Notefield<'a> {
    layout: &'a player_config::NoteLayout,
    column_info: [ColumnInfo<'a>; 4],
    batches: Vec<SpriteBatch>,
    draw_distance: i64,
    last_judgement: Option<Judgement>,
}

#[derive(PartialEq, Debug)]
struct ColumnInfo<'a> {
    on_screen: (usize, usize),
    next_to_hit: usize,
    active_hold: Option<i64>,
    notes: &'a TimingColumn<GameplayInfo>,
    judgement_list: TimingColumn<Judgement>,
}

impl<'a> ColumnInfo<'a> {
    fn from_column(notes: &'a TimingColumn<GameplayInfo>) -> Self {
        ColumnInfo {
            on_screen: (0, 0),
            next_to_hit: 0,
            active_hold: None,
            notes,
            judgement_list: TimingColumn::new(),
        }
    }
    fn update_on_screen(&mut self, layout: &NoteLayout, time: i64, draw_distance: i64) -> bool {
        let mut updated = false;
        let (mut draw_start, mut draw_end) = self.on_screen;
        while draw_end != self.notes.notes.len() - 1
            && (layout.delta_to_position(self.notes.notes[draw_end].0 - time) < draw_distance
                || layout.delta_to_position(self.notes.notes[draw_end].0 - time) > 0)
        {
            draw_end += 1;
            updated = true;
        }
        if self.next_to_hit < draw_end {
            draw_start = self.next_to_hit;
        }
        self.on_screen = (draw_start, draw_end);
        updated
    }
}

impl<'a> Notefield<'a> {
    pub fn new(
        layout: &'a player_config::NoteLayout,
        notes: &'a TimingData<GameplayInfo>,
        draw_distance: i64,
    ) -> Self {
        Notefield {
            layout,
            column_info: [
                ColumnInfo::from_column(&notes.notes[0]),
                ColumnInfo::from_column(&notes.notes[1]),
                ColumnInfo::from_column(&notes.notes[2]),
                ColumnInfo::from_column(&notes.notes[3]),
            ],
            //Using a Vec of SpriteBatch should be temporary, optimally we want to reference these
            // by a NoteType key, but this would require ggez refactoring.
            batches: vec![
                SpriteBatch::new(layout.sprites.hold_end.clone()),
                SpriteBatch::new(layout.sprites.hold_body.clone()),
                SpriteBatch::new(layout.sprites.arrows.clone()),
                SpriteBatch::new(layout.sprites.mine.clone()),
            ],
            draw_distance,
            last_judgement: None,
        }
    }
    fn redraw_batch(&mut self) {
        self.batches.iter_mut().for_each(|x| x.clear());
        for column_index in 0..4 {
            let (draw_start, draw_end) = self.column_info[column_index].on_screen;
            if draw_start < draw_end {
                self.layout.add_column_of_notes(
                    &self.column_info[column_index].notes.notes[draw_start..],
                    column_index,
                    &mut self.batches,
                );
            }
        }
    }
    fn handle_judgement(&mut self, judge: Judgement, column: usize) {
        match judge {
            Judgement::Hit(_) | Judgement::Miss => self.last_judgement = Some(judge),
            _ => {}
        }
        self.column_info[column].judgement_list.add(judge);
    }
}

impl<'a> Element for Notefield<'a> {
    fn run(&mut self, ctx: &mut ggez::Context, time: Option<i64>) -> Result<(), ggez::GameError> {
        self.layout.draw_receptors(ctx)?;
        let time = match time {
            Some(time) => time,
            None => return Ok(()),
        };
        let mut clear_batch = false;
        for column_index in 0..4 {
            let notes = self.column_info[column_index].notes;
            let mut next_to_hit = self.column_info[column_index].next_to_hit;
            if let Some(value) = self.column_info[column_index].active_hold {
                let delta = value - time;
                if delta > 0 {
                    self.layout.add_hold(ctx, column_index, value - time)?;
                }
            }
            while next_to_hit != notes.notes.len() && notes.notes[next_to_hit].0 - time < -180 {
                if notes.notes[next_to_hit].2 == NoteType::Mine {
                    self.handle_judgement(Judgement::Mine(false), column_index);
                } else {
                    self.handle_judgement(Judgement::Miss, column_index);
                }
                next_to_hit += 1;
                clear_batch = true;
            }
            self.column_info[column_index].next_to_hit = next_to_hit;
            if self.column_info[column_index].update_on_screen(
                self.layout,
                time,
                self.draw_distance,
            ) {
                clear_batch = true
            };
        }
        if clear_batch {
            self.redraw_batch();
        }
        let target_parameter =
            graphics::DrawParam::new().dest([0.0, -1.0 * (self.layout.delta_to_offset(time))]);

        for batch in self.batches.iter() {
            graphics::draw(ctx, batch, target_parameter)?;
        }
        if let Some(judgment) = self.last_judgement {
            self.layout.draw_judgment(ctx, judgment)?;
        }
        println!("FPS: {:.2}", ggez::timer::fps(ctx));
        println!(
            "Score: {:.2}%",
            (self
                .column_info
                .iter()
                .map(|x| x.judgement_list.current_points(1.0))
                .sum::<f64>())
                / (self
                    .column_info
                    .iter()
                    .map(|x| x.judgement_list.max_points())
                    .sum::<f64>())
                * 100.0
        );
        Ok(())
    }
    fn start(&mut self, _time: Option<Instant>) -> Result<(), ggez::GameError> {
        Ok(())
    }
    fn handle_event(&mut self, keycode: ggez::event::KeyCode, time: Option<i64>, key_down: bool) {
        let index = match keycode {
            ggez::event::KeyCode::Z => 0,
            ggez::event::KeyCode::X => 1,
            ggez::event::KeyCode::Comma => 2,
            ggez::event::KeyCode::Period => 3,
            _ => return,
        };
        if let Some(hold_end) = self.column_info[index].active_hold {
            if let Some(time) = time {
                if time > hold_end {
                    self.column_info[index]
                        .judgement_list
                        .add(Judgement::Hold(true));
                    self.column_info[index].active_hold = None;
                }
            }
        }
        if key_down {
            loop {
                let delta = self.column_info[index]
                    .notes
                    .notes
                    .get(self.column_info[index].next_to_hit);
                if let (Some(time), Some(GameplayInfo(delta, _, note_type))) = (time, delta) {
                    let offset = delta - time;
                    if offset < 180 {
                        if self.column_info[index].on_screen.0 < self.column_info[index].on_screen.1
                        {
                            if *note_type == NoteType::Hold {
                                self.column_info[index].active_hold = Some(
                                    self.column_info[index].notes.notes
                                        [self.column_info[index].next_to_hit + 1]
                                        .0,
                                );
                                self.column_info[index].next_to_hit += 2;
                                self.column_info[index].on_screen.0 += 2;
                            } else {
                                self.column_info[index].next_to_hit += 1;
                                self.column_info[index].on_screen.0 += 1;
                            }
                        }
                        if offset < -180 {
                            continue;
                        }
                        match *note_type {
                            NoteType::Tap | NoteType::Hold => {
                                self.handle_judgement(Judgement::Hit(offset), index)
                            }
                            NoteType::Mine => self.handle_judgement(Judgement::Mine(true), index),
                            _ => {}
                        }
                        self.handle_judgement(Judgement::Hit(offset), index);
                        self.redraw_batch();
                    }
                }
                break;
            }
        } else {
            if self.column_info[index].active_hold.is_some() {
                self.column_info[index]
                    .judgement_list
                    .add(Judgement::Hold(false));
                self.column_info[index].active_hold = None;
            }
        }
    }
}

impl UserData for Notefield<'static> {}
