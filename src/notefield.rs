use crate::{
    player_config::NoteLayout,
    screen::{Element, Message, Resource},
    timingdata::{GameplayInfo, Judgement, TimingColumn, TimingData},
    NOTEFIELD_SIZE,
};
use ggez::graphics::{self, spritebatch::SpriteBatch};
use notedata::NoteType;
use std::time::Instant;

#[derive(PartialEq, Debug)]
pub struct Notefield {
    layout: NoteLayout,
    column_info: [ColumnInfo; NOTEFIELD_SIZE],
    batches: Vec<SpriteBatch>,
    draw_distance: i64,
    last_judgement: Option<Judgement>,
}

#[derive(PartialEq, Debug)]
struct ColumnInfo {
    on_screen: (usize, usize),
    next_to_hit: usize,
    active_hold: Option<i64>,
    notes: TimingColumn<GameplayInfo>,
    judgement_list: TimingColumn<Judgement>,
}

impl ColumnInfo {
    fn update_on_screen(&mut self, layout: &NoteLayout, time: i64, draw_distance: i64) -> bool {
        let mut updated = false;
        let (draw_start, draw_end) = &mut self.on_screen;
        while *draw_end != self.notes.notes.len()
            && (layout.delta_to_position(self.notes.notes[*draw_end].0 - time) < draw_distance
                || layout.delta_to_position(self.notes.notes[*draw_end].0 - time) > 0)
        {
            *draw_end += 1;
            updated = true;
        }
        if self.next_to_hit <= *draw_end {
            *draw_start = self.next_to_hit;
        }
        updated
    }
    fn update_misses(&mut self, time: i64) -> bool {
        let mut missed_judge = false;
        let mut offset = match self.notes.notes.get(self.next_to_hit) {
            Some(x) => x.0 - time,
            None => return false,
        };
        while offset < -180 {
            let n = self.notes.notes.get(self.next_to_hit);
            let n = match n {
                Some(n) => n.2,
                None => break,
            };
            match n {
                NoteType::Tap => {
                    missed_judge = true;
                    self.judgement_list.add(Judgement::Miss);
                }
                NoteType::Hold => {
                    missed_judge = true;
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
        missed_judge
    }
    fn handle_hit(&mut self, time: i64) -> Option<Judgement> {
        self.update_misses(time);
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
        self.judgement_list.notes.last().copied()
    }
}

impl From<TimingColumn<GameplayInfo>> for ColumnInfo {
    fn from(notes: TimingColumn<GameplayInfo>) -> Self {
        Self {
            on_screen: (0, 0),
            next_to_hit: 0,
            active_hold: None,
            notes,
            judgement_list: TimingColumn::new(),
        }
    }
}

impl Notefield {
    pub fn new(layout: NoteLayout, notes: &TimingData<GameplayInfo>, draw_distance: i64) -> Self {
        let batches = vec![
            SpriteBatch::new(layout.sprites.hold_end.clone()),
            SpriteBatch::new(layout.sprites.hold_body.clone()),
            SpriteBatch::new(layout.sprites.arrows.clone()),
            SpriteBatch::new(layout.sprites.mine.clone()),
        ];
        Self {
            layout,
            column_info: array_init::array_init(|i| ColumnInfo::from(notes.notes[i].clone())),
            //Using a Vec of SpriteBatch should be temporary, optimally we want to reference these
            // by a NoteType key, but this would require ggez refactoring.
            batches,
            draw_distance,
            last_judgement: None,
        }
    }
    fn redraw_batch(&mut self) {
        self.batches
            .iter_mut()
            .for_each(ggez::graphics::spritebatch::SpriteBatch::clear);
        for column_index in 0..NOTEFIELD_SIZE {
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

impl Element for Notefield {
    fn run(
        &mut self,
        ctx: &mut ggez::Context,
        time: Option<i64>,
    ) -> Result<Message, ggez::GameError> {
        self.layout.draw_receptors(ctx)?;
        let time = match time {
            Some(time) => time,
            None => return Ok(Message::None),
        };
        let mut completed = true;
        for column_index in 0..NOTEFIELD_SIZE {
            if let Some(value) = self.column_info[column_index].active_hold {
                let delta = value - time;
                if delta > 0 {
                    self.layout.add_hold(ctx, column_index, value - time)?;
                }
            }
            if self.column_info[column_index].update_misses(time) {
                self.handle_judgement(Judgement::Miss);
            };
            self.column_info[column_index].update_on_screen(&self.layout, time, self.draw_distance);
            completed &= self.column_info[column_index].next_to_hit
                == self.column_info[column_index].notes.notes.len();
            completed &= self.column_info[column_index].active_hold.is_none();
        }
        self.redraw_batch();
        let target_parameter =
            graphics::DrawParam::new().dest([0.0, -1.0 * (self.layout.delta_to_offset(time))]);

        for batch in &self.batches {
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
        Ok(if completed {
            Message::Finish
        } else {
            Message::None
        })
    }
    fn start(&mut self, _time: Option<Instant>) -> Result<Message, ggez::GameError> {
        Ok(Message::None)
    }
    fn finish(&mut self) -> Option<Resource> {
        Some(Resource::Replay(
            self.column_info
                .iter()
                .map(|x| x.judgement_list.clone())
                .collect(),
        ))
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
            if let Some(value) = self.column_info[index].handle_hit(time) {
                self.handle_judgement(value)
            };
        } else if self.column_info[index].active_hold.is_some() {
            self.column_info[index]
                .judgement_list
                .add(Judgement::Hold(false));
            self.column_info[index].active_hold = None;
        }
    }
}
