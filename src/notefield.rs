extern crate ggez;

use crate::notedata::NoteType;
use crate::player_config;
use crate::screen::Element;
use crate::timingdata::{GameplayInfo, OffsetInfo, TimingData};
use ggez::graphics;
use ggez::graphics::spritebatch::SpriteBatch;
use rlua::UserData;
use std::time::Instant;

#[derive(PartialEq)]
pub struct Notefield<'a> {
    layout: &'a super::player_config::NoteLayout,
    notes: &'a TimingData<GameplayInfo>,
    on_screen: Vec<(usize, usize)>,
    batches: Vec<SpriteBatch>,
    draw_distance: i64,
    last_judgement: Option<Judgement>,
    judgment_list: TimingData<OffsetInfo>,
}

#[derive(Copy, Clone, PartialEq)]
pub enum Judgement {
    Hit(usize),
    Miss,
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
            on_screen: Vec::<_>::new(),
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
        for ((column_index, column_data), (draw_start, draw_end)) in
            self.notes.columns().enumerate().zip(&mut self.on_screen)
        {
            if *draw_start < *draw_end {
                self.layout.add_column_of_notes(
                    &column_data[*draw_start..*draw_end],
                    column_index,
                    &mut self.batches,
                );
            }
        }
    }
    fn handle_judgement(&mut self, offset: Option<i64>, column: usize, note_type: NoteType) {
        match note_type {
            NoteType::Tap | NoteType::Hold => match offset {
                Some(off) => match off.abs() {
                    0...22 => self.last_judgement = Some(Judgement::Hit(0)),
                    23...45 => self.last_judgement = Some(Judgement::Hit(1)),
                    46...90 => self.last_judgement = Some(Judgement::Hit(2)),
                    91...135 => self.last_judgement = Some(Judgement::Hit(3)),
                    136...180 => self.last_judgement = Some(Judgement::Hit(4)),
                    //Attempting to register a hit outside the acceptable window would mean
                    // the validity of the score is compromised, therefore it is preferable to panic
                    // rather than attempt recovery.  Alternatives should be evaluated prior to a
                    // public release.
                    _ => panic!("Should not be able to hit note outside registration window"),
                },
                None => self.last_judgement = Some(Judgement::Miss),
            },
            _ => {}
        }

        self.judgment_list
            .add(OffsetInfo(offset, note_type), column);
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
        for ((column_index, column_data), (mut draw_start, mut draw_end)) in
            self.notes.columns().enumerate().zip(self.on_screen.clone())
        {
            while draw_end != column_data.len()
                && self
                    .layout
                    .delta_to_position(column_data[draw_end].0 - time)
                    < self.draw_distance
            {
                if draw_start <= draw_end {
                    self.layout.add_note(
                        column_index,
                        &column_data[draw_end..],
                        &mut self.batches,
                    );
                }
                draw_end += 1;
            }
            while draw_start != column_data.len() && column_data[draw_start].0 - time < -180 {
                self.handle_judgement(None, column_index, column_data[draw_start].2); //this is extremely temporary
                draw_start += 1;
                clear_batch = true;
            }
            self.on_screen[column_index] = (draw_start, draw_end);
        }
        if clear_batch {
            self.redraw_batch();
        }
        let target_parameter = graphics::DrawParam {
            dest: graphics::Point2::new(0.0, -1.0 * (self.layout.delta_to_offset(time))),
            ..Default::default()
        };

        for batch in self.batches.iter() {
            graphics::draw_ex(ctx, batch, target_parameter)?;
        }
        if let Some(judgment) = self.last_judgement {
            self.layout.draw_judgment(ctx, judgment);
        }
        println!("FPS: {:.2}", ggez::timer::get_fps(ctx));
        println!(
            "Score: {:.2}%",
            self.judgment_list.calculate_score() * 100.0
        );
        Ok(())
    }
    fn start(&mut self, _time: Option<Instant>) -> Result<(), ggez::GameError> {
        //self.layout.add_receptors(&mut self.batch)?;
        self.on_screen = self.notes.columns().map(|_| (0, 0)).collect();
        Ok(())
    }
    fn handle_event(&mut self, keycode: ggez::event::Keycode, time: Option<i64>) {
        let index = match keycode {
            ggez::event::Keycode::Z => 0,
            ggez::event::Keycode::X => 1,
            ggez::event::Keycode::Comma => 2,
            ggez::event::Keycode::Period => 3,
            _ => return,
        };
        loop {
            let delta =
                self.notes.columns().collect::<Vec<_>>()[index].get(self.on_screen[index].0);
            if let (Some(time), Some(GameplayInfo(delta, _, note_type))) = (time, delta) {
                let offset = delta - time;
                if offset < 180 {
                    self.on_screen[index].0 += 1;
                    if offset < -180 {
                        continue;
                    }
                    self.handle_judgement(Some(offset), index, *note_type);
                    self.redraw_batch();
                }
            }
            break;
        }
    }
}

impl UserData for Notefield<'static> {}
