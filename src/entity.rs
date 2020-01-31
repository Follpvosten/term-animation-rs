use crossterm::style::StyledContent;

use crate::Animation;

pub type ShouldRerender = bool;
#[derive(Default)]
pub struct CallbackResult {
    pub new_x: Option<i16>,
    pub new_y: Option<i16>,
    pub new_z: Option<i16>,
    pub new_frame: Option<usize>,
}
pub type Callback = Box<dyn Fn(&mut Entity, &mut Animation) -> CallbackResult>;
pub type CollHandler = Box<dyn Fn(&mut Entity, &mut Animation, &mut Entity) -> ShouldRerender>;

pub struct StyledLine(pub Vec<StyledContent<char>>);
pub struct StyledSprite {
    pub lines: Vec<StyledLine>,
}

pub struct Entity {
    pub name: String,
    // appearance
    pub transparent: Option<char>,
    pub frames: Vec<StyledSprite>,
    pub width: i16,
    pub height: i16,
    pub pos: Position,
    // collision detection
    pub physical: bool,
    pub depth: u16,
    pub coll_handler: Option<CollHandler>,
    // behavior
    pub wrap: bool,
    pub callback: Option<Callback>,
    pub follow_entity: Option<String>,
    pub follow_offset: Option<u16>,
    // state
    pub current_frame: usize,
    // entity death
    pub die_offscreen: bool,
    pub die_time: Option<std::time::SystemTime>,
    pub die_frame: Option<i32>,
    pub death_callback: Option<Callback>,
    pub die_entity: Option<String>,
}

impl Entity {
    pub fn set_x(&mut self, new_x: i16, anim_width: i16) {
        self.pos.x = if self.wrap {
            if new_x >= anim_width || new_x < 0 {
                new_x % anim_width
            } else {
                new_x
            }
        } else {
            new_x
        };
    }
    pub fn set_y(&mut self, new_y: i16, anim_height: i16) {
        self.pos.y = if self.wrap {
            if new_y >= anim_height || new_y < 0 {
                new_y % anim_height
            } else {
                new_y
            }
        } else {
            new_y
        };
    }
    pub fn set_z(&mut self, new_z: i16) {
        self.pos.z = new_z
    }

    pub fn set_frame(&mut self, new_frame: usize) {
        if new_frame < self.frames.len() {
            self.current_frame = new_frame;
        } else {
            todo!("Handle errors: Bad frame assigned to {}", self.name)
        }
    }
}

pub struct Position {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}
