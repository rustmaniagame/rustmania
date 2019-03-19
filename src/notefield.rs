extern crate ggez;

use crate::notedata::NoteType;
use crate::player_config::NoteLayout;
use crate::screen::Element;
use crate::timingdata::{GameplayInfo, Judgement, TimingColumn, TimingData};
use ggez::graphics;
use ggez::graphics::spritebatch::SpriteBatch;
use rlua::UserData;
use std::time::Instant;

#[derive(PartialEq, Debug)]
pub struct Notefield<'a> {
    layout: &'a NoteLayout,
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
        let (draw_start, draw_end) = &mut self.on_screen;
        while *draw_end != self.notes.notes.len() - 1
            && (layout.delta_to_position(self.notes.notes[*draw_end].0 - time) < draw_distance
                || layout.delta_to_position(self.notes.notes[*draw_end].0 - time) > 0)
        {
            *draw_end += 1;
            updated = true;
        }
        if self.next_to_hit < *draw_end {
            *draw_start = self.next_to_hit;
        }
        updated
    }
    fn update_for_misses(&mut self, time: i64) -> bool {
        let before = self.next_to_hit;
        let mut offset = match self.notes.notes.get(self.next_to_hit) {
            Some(x) => x.0 - time,
            None => return false,
        };;
        while offset < -180 {
            let n = self.notes.notes.get(self.next_to_hit);
            let n = match n {
                Some(n) => n.2,
                None => break,
            };
            match n {
                NoteType::Tap => {
                    self.judgement_list.add(Judgement::Miss);
                }
                NoteType::Hold => {
                    self.judgement_list.add(Judgement::Miss);
                    self.judgement_list.add(Judgement::Hold(false));
                }
                NoteType::Mine => {
                    self.judgement_list.add(Judgement::Mine(false));
                }
                _ => {}
            };
            self.next_to_hit += 1;
            offset = match self.notes.notes.get(self.next_to_hit) {
                Some(x) => x.0 - time,
                None => break,
            };
        }
        while self.notes.notes.get(self.next_to_hit).map(|x| x.2) == Some(NoteType::HoldEnd) {
            self.next_to_hit += 1;
        }
        before != self.next_to_hit
    }
    fn handle_hit(&mut self, time: i64) -> Option<Judgement> {
        self.update_for_misses(time);
        let offset = self.notes.notes.get(self.next_to_hit).map(|x| x.0 - time)?;
        if offset < 180 {
            match self.notes.notes[self.next_to_hit].2 {
                NoteType::Tap => self.judgement_list.add(Judgement::Hit(offset)),
                NoteType::Hold => {
                    self.judgement_list.add(Judgement::Hit(offset));
                    self.active_hold = self.notes.notes.get(self.next_to_hit + 1).map(|x| x.0);
                }
                NoteType::Mine => self.judgement_list.add(Judgement::Mine(true)),
                _ => {}
            }
            self.next_to_hit += 1;
            while self.notes.notes.get(self.next_to_hit).map(|x| x.2) == Some(NoteType::HoldEnd) {
                self.next_to_hit += 1;
            }
        };
        self.judgement_list.notes.last().map(|x| *x)
    }
}

impl<'a> Notefield<'a> {
    pub fn new(
        layout: &'a NoteLayout,
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
            self.layout.add_column_of_notes(
                &self.column_info[column_index].notes.notes[draw_start..draw_end],
                column_index,
                &mut self.batches,
            );
        }
    }
    fn handle_judgement(&mut self, judge: Judgement) {
        if let Judgement::Hit(_) | Judgement::Miss = judge {
            self.last_judgement = Some(judge);
        }
    }
}

impl<'a> Element for Notefield<'a> {
    fn run(&mut self, ctx: &mut ggez::Context, time: Option<i64>) -> Result<(), ggez::GameError> {
        self.layout.draw_receptors(ctx)?;
        let time = match time {
            Some(time) => time,
            None => return Ok(()),
        };
        for column_index in 0..4 {
            if let Some(value) = self.column_info[column_index].active_hold {
                let delta = value - time;
                if delta > 0 {
                    self.layout.add_hold(ctx, column_index, value - time)?;
                }
            }
            if self.column_info[column_index].update_for_misses(time) {
                self.handle_judgement(Judgement::Miss);
            };
            self.column_info[column_index].update_on_screen(self.layout, time, self.draw_distance);
        }
        self.redraw_batch();
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
        let time = match time {
            Some(time) => time,
            None => return,
        };
        if let Some(hold_end) = self.column_info[index].active_hold {
            if time > hold_end {
                self.column_info[index]
                    .judgement_list
                    .add(Judgement::Hold(true));
                self.column_info[index].active_hold = None;
            }
        }
        if key_down {
            match self.column_info[index].handle_hit(time) {
                Some(value) => self.handle_judgement(value),
                None => {}
            };
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
