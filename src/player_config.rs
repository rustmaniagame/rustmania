extern crate ggez;
use crate::notedata::NoteType;
use crate::notefield::Judgement;
use crate::timingdata::GameplayInfo;
use gfx_core::texture::WrapMode;
use ggez::error::GameResult;
use ggez::graphics::{self, Rect};
use serde_derive::Deserialize;
use std::fs::File;
use std::io::Read;
use toml;

#[derive(PartialEq)]
pub struct NoteLayout {
    pub sprites: NoteSprites,
    pub column_positions: [i64; 4],
    pub column_rotations: [f32; 4],
    pub receptor_height: i64,
    pub judgment_position: [f32; 2],
    pub scroll_speed: f32,
}

#[derive(PartialEq, Clone)]
pub struct NoteSkin {
    pub sprites: NoteSprites,
    pub column_positions: [i64; 4],
    pub column_rotations: [f32; 4],
}

#[derive(PartialEq, Clone)]
pub struct NoteSprites {
    pub arrows: graphics::Image,
    pub receptor: graphics::Image,
    pub judgment: graphics::Image,
    pub hold_body: graphics::Image,
    pub hold_end: graphics::Image,
    pub mine: graphics::Image,
}

#[derive(PartialEq, Copy, Clone)]
pub struct PlayerOptions {
    notefield_position: i64,
    receptor_height: i64,
    scroll_speed: f32,
    is_reverse: bool,
    judgment_position: (f32, f32),
}

impl NoteLayout {
    pub fn new(skin: &NoteSkin, screen_height: i64, player_options: PlayerOptions) -> NoteLayout {
        let NoteSkin {
            sprites,
            mut column_positions,
            mut column_rotations,
        } = skin.clone();
        let PlayerOptions {
            notefield_position,
            mut receptor_height,
            mut scroll_speed,
            is_reverse,
            mut judgment_position,
        } = player_options;
        column_positions
            .iter_mut()
            .for_each(|x| *x += notefield_position);
        column_rotations.iter_mut().for_each(|x| *x *= 6.28 / 360.0);
        judgment_position.0 += notefield_position as f32;
        if is_reverse {
            receptor_height = screen_height - receptor_height;
            judgment_position.1 = screen_height as f32 - judgment_position.1;
            scroll_speed *= -1.0;
        }
        let judgment_position = [judgment_position.0, judgment_position.1];
        NoteLayout {
            sprites,
            column_positions,
            column_rotations,
            receptor_height,
            judgment_position,
            scroll_speed,
        }
    }
    pub fn delta_to_position(&self, delta: i64) -> i64 {
        (delta as f32 * self.scroll_speed) as i64 + self.receptor_height
    }
    pub fn delta_to_offset(&self, delta: i64) -> f32 {
        (delta as f32 * self.scroll_speed)
    }
    pub fn add_note(
        &self,
        column: usize,
        column_data: &[GameplayInfo],
        batches: &mut Vec<graphics::spritebatch::SpriteBatch>,
    ) {
        let GameplayInfo(position, coords, note_type) = match column_data.get(0) {
            Some(val) => *val,
            None => return,
        };
        let position = self.delta_to_position(position);
        let batch_index = match note_type {
            NoteType::Tap => 2,
            NoteType::Hold => {
                if let Some(GameplayInfo(end, _, _)) = column_data.get(1) {
                    batches[1].add(
                        graphics::DrawParam::new()
                            .src(Rect::new(
                                0.0,
                                0.0,
                                1.0,
                                (position - self.delta_to_position(*end)) as f32 / 64.0
                                    + if self.scroll_speed > 0.0 { 0.5 } else { -0.5 },
                            ))
                            .dest([self.column_positions[column] as f32, position as f32])
                            .rotation(if note_type == NoteType::Tap {
                                self.column_rotations[column]
                            } else {
                                0.0
                            })
                            .offset([0.5, 1.0]),
                    );
                };
                2
            }
            NoteType::Roll => 2,
            NoteType::Mine => 3,
            NoteType::Lift => 2,
            NoteType::Fake => 2,
            NoteType::HoldEnd => 0,
        };
        batches[batch_index].add(
            graphics::DrawParam::new()
                .src(coords)
                .dest([self.column_positions[column] as f32, position as f32])
                .rotation(if note_type == NoteType::Tap {
                    self.column_rotations[column]
                } else {
                    0.0
                })
                .offset([0.5, 0.5])
                .scale(if batch_index == 0 && self.scroll_speed > 0.0 {
                    [1.0, -1.0]
                } else {
                    [1.0, 1.0]
                }),
        );
    }
    pub fn add_column_of_notes(
        &self,
        column: &[GameplayInfo],
        column_index: usize,
        batches: &mut Vec<graphics::spritebatch::SpriteBatch>,
    ) {
        for index in 0..column.len() {
            self.add_note(column_index, &column[index..], batches);
        }
    }
    pub fn draw_receptors(&self, ctx: &mut ggez::Context) -> Result<(), ggez::GameError> {
        for (index, &column_position) in self.column_positions.iter().enumerate() {
            graphics::draw(
                ctx,
                &self.sprites.receptor,
                graphics::DrawParam::new()
                    .dest([column_position as f32, self.receptor_height as f32])
                    .rotation(self.column_rotations[index])
                    .offset([0.5, 0.5]),
            )?;
        }
        Ok(())
    }
    //this will likely be the method to draw receptors in the future, but it is not currently in use
    pub fn _add_receptors(
        &self,
        batch: &mut graphics::spritebatch::SpriteBatch,
    ) -> Result<(), ggez::GameError> {
        for &column_position in &self.column_positions {
            batch.add(
                graphics::DrawParam::new()
                    .dest([column_position as f32, self.receptor_height as f32]),
            );
        }
        Ok(())
    }
    fn select_judgment(&self, judge: Judgement) -> graphics::DrawParam {
        let src = match judge {
            Judgement::Hit(0) => graphics::Rect::new(0.0, 0.0, 1.0, 0.1666),
            Judgement::Hit(1) => graphics::Rect::new(0.0, 0.1666, 1.0, 0.1666),
            Judgement::Hit(2) => graphics::Rect::new(0.0, 0.3333, 1.0, 0.1666),
            Judgement::Hit(3) => graphics::Rect::new(0.0, 0.5, 1.0, 0.1666),
            Judgement::Hit(_) => graphics::Rect::new(0.0, 0.6666, 1.0, 0.1666),
            Judgement::Miss => graphics::Rect::new(0.0, 0.8333, 1.0, 1.666),
        };
        graphics::DrawParam::new()
            .src(src)
            .dest(self.judgment_position)
    }
    pub fn draw_judgment(
        &self,
        ctx: &mut ggez::Context,
        judge: Judgement,
    ) -> Result<(), ggez::GameError> {
        graphics::draw(ctx, &self.sprites.judgment, self.select_judgment(judge))?;
        Ok(())
    }
}

#[derive(Deserialize)]
struct NoteSkinInfo {
    arrows: String,
    receptor: String,
    judgment: String,
    hold_body: String,
    hold_head: String,
    mine: String,
    column_positions: [i64; 4],
    column_rotations: [f32; 4],
}

impl NoteSkin {
    pub fn from_path(path: &str, context: &mut ggez::Context) -> Option<Self> {
        let mut config_file = match File::open(format!("{}/config.toml", path)) {
            Ok(file) => file,
            Err(_) => return None,
        };
        let mut config_string = String::new();
        match config_file.read_to_string(&mut config_string) {
            Ok(_) => {}
            Err(_) => return None,
        };
        let NoteSkinInfo {
            arrows,
            receptor,
            judgment,
            hold_body,
            hold_head,
            mine,
            column_positions,
            column_rotations,
        } = match toml::from_str(&config_string) {
            Ok(skin) => skin,
            Err(_) => return None,
        };
        if let (
            Ok(arrows),
            Ok(receptor),
            Ok(judgment),
            Ok(mut hold_body),
            Ok(hold_head),
            Ok(mine),
        ) = (
            image_from_subdirectory(context, path, arrows),
            image_from_subdirectory(context, path, receptor),
            image_from_subdirectory(context, path, judgment),
            image_from_subdirectory(context, path, hold_body),
            image_from_subdirectory(context, path, hold_head),
            image_from_subdirectory(context, path, mine),
        ) {
            hold_body.set_wrap(WrapMode::Tile, WrapMode::Tile);
            let sprites = NoteSprites {
                arrows,
                receptor,
                judgment,
                hold_body,
                hold_end: hold_head,
                mine,
            };
            Some(NoteSkin {
                sprites,
                column_positions,
                column_rotations,
            })
        } else {
            None
        }
    }
}

fn image_from_subdirectory(
    context: &mut ggez::Context,
    path: &str,
    extension: String,
) -> GameResult<graphics::Image> {
    graphics::Image::new(context, format!("/{}/{}", path, extension))
}

impl PlayerOptions {
    pub fn new(
        notefield_position: i64,
        receptor_height: i64,
        scroll_speed: f32,
        is_reverse: bool,
        judgment_position: (f32, f32),
    ) -> Self {
        PlayerOptions {
            notefield_position,
            receptor_height,
            scroll_speed,
            is_reverse,
            judgment_position,
        }
    }
}
