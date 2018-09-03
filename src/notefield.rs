extern crate ggez;

use ggez::graphics;
use ggez::graphics::spritebatch::SpriteBatch;
use player_config;
use timingdata::{GameplayInfo, OffsetInfo, TimingData};

pub struct Notefield<'a> {
    layout: &'a super::player_config::NoteLayout,
    notes: &'a TimingData<GameplayInfo>,
    on_screen: Vec<(usize, usize)>,
    batch: SpriteBatch,
    draw_distance: i64,
    last_judgement: Option<Judgement>,
    judgment_list: TimingData<OffsetInfo>,
}

#[derive(Copy, Clone)]
pub enum Judgement {
    Hit(usize),
    Miss,
}

impl<'a> Notefield<'a> {
    pub fn new(
        layout: &'a player_config::NoteLayout,
        notes: &'a TimingData<GameplayInfo>,
        batch: SpriteBatch,
        draw_distance: i64,
    ) -> Self {
        Notefield {
            layout,
            notes,
            on_screen: Vec::<_>::new(),
            batch,
            draw_distance,
            last_judgement: None,
            judgment_list: TimingData::<_>::new(),
        }
    }
    pub fn start(&mut self) -> Result<(), ggez::GameError> {
        //self.layout.add_receptors(&mut self.batch)?;
        self.on_screen = self.notes
            .columns()
            .map(|x| {
                (
                    0,
                    match x.iter()
                        .position(|GameplayInfo(y, _)| *y > self.draw_distance)
                    {
                        Some(num) => num,
                        None => x.len(),
                    },
                )
            })
            .collect();
        Ok(())
    }
    fn redraw_batch(&mut self) {
        self.batch.clear();
        for ((column_index, column_data), (draw_start, draw_end)) in
            self.notes.columns().enumerate().zip(&mut self.on_screen)
        {
            if *draw_start < *draw_end {
                self.layout.add_column_of_notes(
                    column_data[*draw_start..*draw_end].iter().map(|x| *x),
                    column_index,
                    &mut self.batch,
                );
            }
        }
    }
    pub fn draw_field(
        &mut self,
        ctx: &mut ggez::Context,
        time: Option<i64>,
    ) -> Result<(), ggez::GameError> {
        self.layout.draw_receptors(ctx)?;
        let time = match time {
            Some(time) => time,
            None => return Ok(()),
        };
        let mut clear_batch = false;
        for ((column_index, column_data), (draw_start, draw_end)) in
            self.notes.columns().enumerate().zip(&mut self.on_screen)
        {
            while *draw_end != column_data.len()
                && self.layout
                    .delta_to_position(column_data[*draw_end].0 - time)
                    < self.draw_distance
            {
                if *draw_start <= *draw_end {
                    self.layout.add_note(
                        column_index,
                        self.layout.delta_to_position(column_data[*draw_end].0),
                        column_data[*draw_end].1,
                        &mut self.batch,
                    );
                }
                *draw_end += 1;
            }
            while *draw_start != column_data.len() && column_data[*draw_start].0 - time < -180 {
                *draw_start += 1;
                clear_batch = true;
            }
        }
        if clear_batch {
            self.redraw_batch();
            self.last_judgement = Some(Judgement::Miss);
        }
        let target_parameter = graphics::DrawParam {
            dest: graphics::Point2::new(0.0, -1.0 * (self.layout.delta_to_offset(time))),
            ..Default::default()
        };
        graphics::draw_ex(ctx, &self.batch, target_parameter)?;
        if let Some(judgment) = self.last_judgement {
            self.layout.draw_judgment(ctx, judgment)?;
        }
        Ok(())
    }
    pub fn handle_event(&mut self, keycode: ggez::event::Keycode, time: Option<i64>) {
        let index = match keycode {
            ggez::event::Keycode::Z => 0,
            ggez::event::Keycode::X => 1,
            ggez::event::Keycode::Comma => 2,
            ggez::event::Keycode::Period => 3,
            _ => return,
        };
        let delta = self.notes.columns().collect::<Vec<_>>()[index].get(self.on_screen[index].0);
        if let (Some(time), Some(GameplayInfo(delta, _))) = (time, delta) {
            let offset = delta - time;
            if offset < 180 {
                self.on_screen[index].0 += 1;
                self.handle_judgement(offset, index);
                self.redraw_batch();
            }
        }
    }
    //noinspection RsUnresolvedReference
    fn handle_judgement(&mut self, offset: i64, column: usize) {
        let abs_offset = offset.abs();
        match abs_offset {
            0...22 => self.last_judgement = Some(Judgement::Hit(0)),
            23...45 => self.last_judgement = Some(Judgement::Hit(1)),
            46...90 => self.last_judgement = Some(Judgement::Hit(2)),
            91...135 => self.last_judgement = Some(Judgement::Hit(3)),
            136...180 => self.last_judgement = Some(Judgement::Hit(4)),
            _ => {}
        }
        self.judgment_list.add(OffsetInfo(offset), column);
    }
}
