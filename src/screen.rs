use crate::SongOptions;
use ggez::{
    event::{KeyCode, KeyMods},
    graphics::{self, Color},
    Context, GameError,
};
use notedata::{
    timingdata::{GameplayInfo, Judgement, TimingColumn, TimingData},
    ChartMetadata, NOTEFIELD_SIZE,
};
use serde_derive::{Deserialize, Serialize};
use std::sync::mpsc::channel;
use std::{
    collections::HashMap,
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};
use utils::music::{play_file, Music};
use utils::notefield::{player_config::NoteLayout, Notefield};

pub trait Element: Send {
    fn run(&mut self, context: &mut Context, time: Option<i64>) -> Result<Message, GameError>;
    fn start(&mut self, time: Option<Instant>) -> Result<Message, GameError>;
    fn finish(&mut self) -> Option<Resource>;
    fn handle_event(&mut self, key: KeyCode, time: Option<i64>, key_down: bool);
    fn methods(&mut self, _resource: Option<Resource>, _index: usize) -> Option<Resource> {
        None
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
pub enum ElementType {
    MUSIC {
        rate: String,
        path: String,
    },
    NOTEFIELD {
        layout: String,
        notes: String,
        draw_distance: String,
    },
    TEXT {
        contents: String,
        x_pos: String,
        y_pos: String,
    },
}

#[derive(Clone, Debug)]
pub enum Resource {
    _Notes(TimingData<GameplayInfo>),
    _Path(PathBuf),
    _Layout(Box<NoteLayout>),
    Float(f64),
    Integer(i64),
    String(String),
    Replay(Vec<TimingColumn<Judgement>>),
    _Multiple(Vec<Resource>),
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum ResourceType {
    Notes,
    Path,
    _Layout,
    Float,
    Integer,
    String,
    Replay,
    _Multiple,
}

pub type ResourceCallback = fn(Option<Resource>, &Globals) -> Option<Resource>;

#[derive(Clone)]
pub struct Resources {
    notes: HashMap<String, TimingData<GameplayInfo>>,
    paths: HashMap<String, PathBuf>,
    layouts: HashMap<String, NoteLayout>,
    floats: HashMap<String, f64>,
    integers: HashMap<String, i64>,
    strings: HashMap<String, String>,
    replays: HashMap<String, Vec<TimingColumn<Judgement>>>,
    multiples: HashMap<String, Vec<Resource>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ScriptMap {
    pub resource_type: ResourceType,
    pub resource_index: String,
    pub script_index: usize,
    pub destination_type: ResourceType,
    pub destination_index: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ElementMap {
    pub resource_index: String,
    pub element_index: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Message {
    None,
    Finish(String),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MethodMap {
    pub element: String,
    pub method: usize,
    pub resource: String,
    pub resource_type: ResourceType,
    pub ret_index: String,
    pub ret_type: ResourceType,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ResourceMap {
    Script(ScriptMap),
    Element(ElementMap),
    Message(Message),
    Method(MethodMap),
}

pub type ResourceMaps = Vec<ResourceMap>;

#[derive(Clone, Deserialize, Serialize)]
pub struct ScriptList {
    pub scripts: HashMap<String, ResourceMaps>,
}

#[derive(Deserialize, Serialize)]
pub struct ScreenBuilder {
    pub elements: HashMap<String, ElementType>,
    pub on_finish: String,
    pub on_keypress: HashMap<String, String>,
}

pub struct Screen {
    start_time: Option<Instant>,
    elements: HashMap<String, Box<dyn Element>>,
    on_finish: String,
    on_keypress: HashMap<String, String>,
    pub current_message: Message,
}

pub struct CacheEntry {
    pub path: PathBuf,
    pub difficulty: f64,
    pub data: ChartMetadata,
}

pub struct Globals {
    pub cache: Vec<CacheEntry>,
    pub song_options: SongOptions,
}

#[derive(Deserialize, Serialize)]
pub struct Theme {
    pub start_screen: String,
    pub scene_stack: HashMap<String, ScreenBuilder>,
    pub scripts: ScriptList,
}

fn to_milliseconds(dur: Duration) -> i64 {
    dur.as_secs() as i64 * 1000 + i64::from(dur.subsec_millis())
}

impl ElementType {
    pub fn build(&self, resources: &Resources) -> Box<dyn Element> {
        match self {
            Self::MUSIC { rate, path } => Box::new(Music::new(
                resources.floats[rate],
                resources.paths[path].clone(),
            )),
            Self::NOTEFIELD {
                layout,
                notes,
                draw_distance,
            } => Box::new(Notefield::new(
                resources.layouts[layout].clone(),
                &resources.notes[notes],
                resources.integers[draw_distance],
            )),
            Self::TEXT {
                contents,
                x_pos,
                y_pos,
            } => Box::new(crate::text::TextBox::new(
                resources.strings[contents].clone(),
                [
                    resources.floats[x_pos] as f32,
                    resources.floats[y_pos] as f32,
                ],
                0,
            )),
        }
    }
}

impl Resources {
    pub fn _new() -> Self {
        Self {
            notes: HashMap::new(),
            paths: HashMap::new(),
            layouts: HashMap::new(),
            floats: HashMap::new(),
            integers: HashMap::new(),
            strings: HashMap::new(),
            replays: HashMap::new(),
            multiples: HashMap::new(),
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        notes: HashMap<String, TimingData<GameplayInfo>>,
        paths: HashMap<String, PathBuf>,
        layouts: HashMap<String, NoteLayout>,
        floats: HashMap<String, f64>,
        integers: HashMap<String, i64>,
        strings: HashMap<String, String>,
        replays: HashMap<String, Vec<TimingColumn<Judgement>>>,
        multiples: HashMap<String, Vec<Resource>>,
    ) -> Self {
        Self {
            notes,
            paths,
            layouts,
            floats,
            integers,
            strings,
            replays,
            multiples,
        }
    }
    pub fn push(&mut self, key: String, resource: Resource) {
        match resource {
            Resource::_Notes(notes) => {
                self.notes.insert(key, notes);
            }
            Resource::_Path(path) => {
                self.paths.insert(key, path);
            }
            Resource::_Layout(layout) => {
                self.layouts.insert(key, *layout);
            }
            Resource::Float(f) => {
                self.floats.insert(key, f);
            }
            Resource::Integer(int) => {
                self.integers.insert(key, int);
            }
            Resource::String(string) => {
                self.strings.insert(key, string);
            }
            Resource::Replay(replay) => {
                self.replays.insert(key, replay);
            }
            Resource::_Multiple(list) => {
                list.into_iter()
                    .for_each(|resource| self.push(key.clone(), resource));
            }
        };
    }
    pub fn get(&self, index: String, resource_type: ResourceType) -> Resource {
        match resource_type {
            ResourceType::Notes => Resource::_Notes(self.notes[&index].clone()),
            ResourceType::Path => Resource::_Path(self.paths[&index].clone()),
            ResourceType::_Layout => Resource::_Layout(Box::new(self.layouts[&index].clone())),
            ResourceType::Float => Resource::Float(self.floats[&index]),
            ResourceType::Integer => Resource::Integer(self.integers[&index]),
            ResourceType::String => Resource::String(self.strings[&index].clone()),
            ResourceType::Replay => Resource::Replay(self.replays[&index].clone()),
            ResourceType::_Multiple => Resource::_Multiple(self.multiples[&index].clone()),
        }
    }
    pub fn set(&mut self, index: String, value: Resource) -> Option<()> {
        match value {
            Resource::_Notes(val) => {
                *self.notes.get_mut(&index)? = val;
                Some(())
            }
            Resource::_Path(val) => {
                *self.paths.get_mut(&index)? = val;
                Some(())
            }
            Resource::_Layout(val) => {
                *self.layouts.get_mut(&index)? = *val;
                Some(())
            }
            Resource::Float(val) => {
                *self.floats.get_mut(&index)? = val;
                Some(())
            }
            Resource::Integer(val) => {
                *self.integers.get_mut(&index)? = val;
                Some(())
            }
            Resource::String(val) => {
                *self.strings.get_mut(&index)? = val;
                Some(())
            }
            Resource::Replay(val) => {
                *self.replays.get_mut(&index)? = val;
                Some(())
            }
            Resource::_Multiple(val) => {
                *self.multiples.get_mut(&index)? = val;
                Some(())
            }
        }
    }
}

impl ScreenBuilder {
    pub fn build(&self, resources: &Resources) -> Screen {
        let element_list = self
            .elements
            .iter()
            .map(|(name, element)| (name.clone(), element.build(resources)))
            .collect();
        Screen::new(
            element_list,
            self.on_finish.clone(),
            self.on_keypress.clone(),
        )
    }
}

// Temporary workaround for ggez's reexported KeyCode not implementing serde traits
fn keycode_number(code: KeyCode) -> String {
    match code {
        KeyCode::Return => "enter",
        KeyCode::Left => "left",
        KeyCode::Right => "right",
        KeyCode::Escape => "esc",
        KeyCode::Grave => "grave",
        KeyCode::Z => "z",
        KeyCode::X => "x",
        KeyCode::Comma => "comma",
        KeyCode::Period => "period",
        _ => "",
    }
    .to_string()
}

impl Screen {
    pub fn new(
        elements: HashMap<String, Box<dyn Element>>,
        on_finish: String,
        on_keypress: HashMap<String, String>,
    ) -> Self {
        Self {
            start_time: Some(Instant::now() + Duration::from_secs(3)),
            elements,
            on_finish,
            on_keypress,
            current_message: Message::None,
        }
    }
    fn run_script(
        &mut self,
        resources: &mut Resources,
        callbacks: &[ResourceCallback],
        globals: &Globals,
        script: &[ResourceMap],
    ) {
        for map in script {
            match map {
                ResourceMap::Element(ElementMap {
                    resource_index,
                    element_index,
                }) => {
                    if let Some(resource) = self
                        .elements
                        .get_mut(element_index)
                        .and_then(|e| e.finish())
                    {
                        if resources
                            .set(resource_index.clone(), resource.clone())
                            .is_none()
                        {
                            resources.push(resource_index.clone(), resource)
                        }
                    }
                }
                ResourceMap::Script(ScriptMap {
                    resource_type,
                    resource_index,
                    script_index,
                    destination_type: _destination_type,
                    destination_index,
                }) => {
                    if let Some(resource) = callbacks[*script_index](
                        Some(resources.get(resource_index.clone(), *resource_type)),
                        globals,
                    ) {
                        resources.set(destination_index.clone(), resource);
                    }
                }
                ResourceMap::Message(message) => {
                    self.current_message = message.clone();
                }
                ResourceMap::Method(map) => {
                    if let Some(elem) = self.elements.get_mut(&map.element) {
                        if let Some(result) = elem.methods(
                            Some(resources.get(map.resource.clone(), map.resource_type)),
                            map.method,
                        ) {
                            resources.set(map.ret_index.clone(), result);
                        }
                    }
                }
            }
        }
    }
    pub fn start(&mut self) -> Result<(), GameError> {
        for element in self.elements.values_mut() {
            element.start(self.start_time)?;
        }
        Ok(())
    }
    pub fn finish(
        &mut self,
        resources: &mut Resources,
        callbacks: &[ResourceCallback],
        globals: &Globals,
        scripts: &ScriptList,
    ) {
        if let Some(script) = scripts.scripts.get(&self.on_finish) {
            self.run_script(resources, callbacks, globals, script);
        }
        for element in self.elements.values_mut() {
            element.finish();
        }
    }
    fn start_time_to_milliseconds(&self) -> Option<i64> {
        match self.start_time {
            Some(time) => {
                let now = Instant::now();
                if time > now {
                    Some(-to_milliseconds(time.duration_since(now)))
                } else {
                    Some(to_milliseconds(now.duration_since(time)))
                }
            }
            None => None,
        }
    }
}

impl Screen {
    pub fn draw(&mut self, ctx: &mut Context) -> Result<Message, GameError> {
        graphics::clear(ctx, Color::new(0.0, 0.0, 0.0, 1.0));
        let time_delta = self.start_time_to_milliseconds();
        for element in self.elements.values_mut() {
            match element.run(ctx, time_delta)? {
                Message::None => {}
                Message::Finish(val) => return Ok(Message::Finish(val)),
            }
        }
        graphics::present(ctx)?;
        Ok(Message::None)
    }
    pub fn key_down_event(
        &mut self,
        keycode: KeyCode,
        resources: &mut Resources,
        callbacks: &[ResourceCallback],
        globals: &Globals,
        scripts: &ScriptList,
    ) {
        if let Some(cool) = self.on_keypress.get(&keycode_number(keycode)) {
            if let Some(script) = scripts.scripts.get(cool) {
                self.run_script(resources, callbacks, globals, script);
            }
        }
        let time_delta = self.start_time_to_milliseconds();
        for element in self.elements.values_mut() {
            element.handle_event(keycode, time_delta, true);
        }
    }
    pub fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods) {
        let time_delta = self.start_time_to_milliseconds();
        for element in self.elements.values_mut() {
            element.handle_event(keycode, time_delta, false);
        }
    }
}

impl Element for Music {
    fn run(&mut self, _ctx: &mut Context, _time: Option<i64>) -> Result<Message, GameError> {
        Ok(Message::None)
    }
    fn start(&mut self, time: Option<Instant>) -> Result<Message, GameError> {
        if let Some(time) = time {
            let rate = self.rate;
            let path = self.path.clone();
            let (send, recv) = channel();
            self.sender = Some(send);
            thread::spawn(move || play_file(time, rate, path, recv));
        }
        Ok(Message::None)
    }
    fn finish(&mut self) -> Option<Resource> {
        if let Some(sender) = &self.sender {
            sender.send(true).expect("fuck");
        }
        None
    }
    fn handle_event(&mut self, _keycode: KeyCode, _time: Option<i64>, _key_down: bool) {}
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
        Ok(if completed {
            Message::Finish("results".to_string())
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
    fn methods(&mut self, _resource: Option<Resource>, index: usize) -> Option<Resource> {
        match index {
            0 => Some(Resource::Float(
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
                    * 100.0,
            )),
            _ => None,
        }
    }
}
