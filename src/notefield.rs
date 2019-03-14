extern crate ggez;

use crate::notedata::NoteType;
use crate::player_config;
use crate::screen::Element;
use crate::timingdata::{GameplayInfo, Judgement, TimingData};
use ggez::graphics;
use ggez::graphics::spritebatch::SpriteBatch;
use rlua::UserData;
use std::time::Instant;

#[derive(PartialEq, Debug)]
pub struct Notefield<'a> {
    layout: &'a player_config::NoteLayout,
    notes: &'a TimingData<GameplayInfo>,
    column_info: [ColumnInfo; 4],
    batches: Vec<SpriteBatch>,
    draw_distance: i64,
    last_judgement: Option<Judgement>,
    judgment_list: TimingData<Judgement>,
}

#[derive(Copy, Clone, PartialEq, Debug)]
struct ColumnInfo {
    on_screen: (usize, usize),
    next_to_hit: usize,
    active_hold: Option<i64>,
}

impl ColumnInfo {
    fn new() -> Self {
        ColumnInfo {
            on_screen: (0, 0),
            next_to_hit: 0,
            active_hold: None,
        }
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
            notes,
            column_info: [ColumnInfo::new(); 4],
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
            judgment_list: TimingData::<_>::new(),
        }
    }
    fn redraw_batch(&mut self) {
        self.batches.iter_mut().for_each(|x| x.clear());
        for (column_index, column_data) in self.notes.columns().enumerate() {
            let (draw_start, draw_end) = self.column_info[column_index].on_screen;
            if draw_start < draw_end {
                self.layout.add_column_of_notes(
                    &column_data[draw_start..],
                    column_index,
                    &mut self.batches,
                );
            }
        }
    }
    fn handle_judgement(&mut self, offset: Judgement, column: usize, note_type: NoteType) {
        match note_type {
            NoteType::Tap | NoteType::Hold => self.last_judgement = Some(offset),
            _ => {}
        }
        self.judgment_list.add(offset, column);
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
        for (column_index, column_data) in self.notes.columns().enumerate() {
            let (mut draw_start, mut draw_end) = self.column_info[column_index].on_screen;
            if let Some(value) = self.column_info[column_index].active_hold {
                let delta = value - time;
                if delta > 0 {
                    self.layout.add_hold(ctx, column_index, value - time)?;
                }
            }
            while draw_end != column_data.len() - 1
                && (self
                    .layout
                    .delta_to_position(column_data[draw_end].0 - time)
                    < self.draw_distance
                    || self
                        .layout
                        .delta_to_position(column_data[draw_end].0 - time)
                        > 0)
            {
                draw_end += 1;
                clear_batch = true;
            }
            let mut next_note = self.column_info[column_index].next_to_hit;
            while next_note != column_data.len() && column_data[next_note].0 - time < -180 {
                self.handle_judgement(Judgement::Miss, column_index, column_data[next_note].2);
                next_note += 1;
                clear_batch = true;
            }
            self.column_info[column_index].next_to_hit = next_note;
            if next_note < draw_end {
                draw_start = next_note;
            }
            self.column_info[column_index].on_screen = (draw_start, draw_end);
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
            self.judgment_list.calculate_score() * 100.0
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
                    self.judgment_list.add(Judgement::Hold(true), index);
                    self.column_info[index].active_hold = None;
                }
            }
        }
        if key_down {
            loop {
                let delta = self.notes.notes[index].get(self.column_info[index].next_to_hit);
                if let (Some(time), Some(GameplayInfo(delta, _, note_type))) = (time, delta) {
                    let offset = delta - time;
                    if offset < 180 {
                        if self.column_info[index].on_screen.0 < self.column_info[index].on_screen.1
                        {
                            if *note_type == NoteType::Hold {
                                self.column_info[index].active_hold = Some(
                                    self.notes.notes[index]
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
                        self.handle_judgement(Judgement::Hit(offset), index, *note_type);
                        self.redraw_batch();
                    }
                }
                break;
            }
        } else {
            if self.column_info[index].active_hold.is_some() {
                self.judgment_list.add(Judgement::Hold(false), index);
                self.column_info[index].active_hold = None;
            }
        }
    }
}

impl UserData for Notefield<'static> {}
