use std::convert::TryFrom;

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
pub type CollHandler = Box<dyn Fn(&mut Entity, &mut Animation, &Entity)>;

pub struct StyledLine(pub Vec<StyledContent<char>>);
pub struct StyledSprite {
    pub lines: Vec<StyledLine>,
}

impl StyledSprite {
    pub fn from_str_simple(source: &str) -> Self {
        let lines = source
            .split('\n')
            .map(|line| StyledLine(line.chars().map(crossterm::style::style).collect()))
            .collect();
        Self { lines }
    }
}

#[derive(Default)]
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
    pub depth: i16,
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
    pub fn from_sprite_simple(sprite: &str) -> Self {
        let mut new_entity = Entity {
            frames: vec![StyledSprite::from_str_simple(sprite)],
            ..Entity::default()
        };
        new_entity.calc_dimensions();
        new_entity
    }
    fn calc_dimensions(&mut self) {
        self.height = self
            .frames
            .iter()
            .map(|f| f.lines.len())
            .max()
            .map(|n| i16::try_from(n).expect("Sprite size may not exceed i16::MAX!"))
            .expect("Tried to calc dimensions without any frames!");
        self.width = self
            .frames
            .iter()
            .map(|f| f.lines.iter().map(|l| l.0.len()).max().unwrap_or(0))
            .max()
            .map(|n| i16::try_from(n).expect("Sprite size may not exceed i16::MAX!"))
            .expect("Tried to calc dimensions without any frames!");
        // ...depth cannot be calculated unless we're somehow 3D.
    }
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
    pub fn intersects(&self, other: &Entity) -> bool {
        fn coord_intersects(my_coord: i16, other_coord: i16, my_d3: i16, other_d3: i16) -> bool {
            (other_coord <= my_coord && my_coord < other_coord + other_d3)
                || (my_coord <= other_coord && other_coord < my_coord + my_d3)
        }
        coord_intersects(self.pos.x, other.pos.x, self.height, other.height)
            && coord_intersects(self.pos.y, other.pos.y, self.width, other.width)
            && coord_intersects(self.pos.z, other.pos.z, self.depth, other.depth)
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Position {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

#[cfg(test)]
mod tests {
    use super::*;
    const SQUARE: &str = "-----------------
|                |
|                |
|                |
|                |
|                |
------------------";
    #[test]
    fn dimensions() {
        let entity = Entity::from_sprite_simple(SQUARE);
        assert_eq!(entity.width, 18);
        assert_eq!(entity.height, 7);

        let entity = Entity::from_sprite_simple("");
        assert_eq!(entity.width, 0);
        // Height is 1 because we have one line.
        assert_eq!(entity.height, 1);
    }

    #[test]
    fn intersects() {
        let mut entity1 = Entity::from_sprite_simple(SQUARE);
        entity1.depth = 1;
        let mut entity2 = Entity::from_sprite_simple(SQUARE);
        entity2.depth = 1;
        assert_eq!(entity1.pos, Position { x: 0, y: 0, z: 0 });
        assert_eq!(entity2.pos, Position { x: 0, y: 0, z: 0 });
        // We have to equally sized entities placed at position 0, 0, 0
        // It's pretty clear that they should intersect.
        assert!(entity1.intersects(&entity2));
        // Now let's move entity2 away a bit...
        entity2.pos.x += entity1.height + 2;
        entity2.pos.y += entity1.width + 2;
        assert!(!entity1.intersects(&entity2));
        // And now, partial intersections
        entity2.pos.x -= entity1.height / 2;
        entity2.pos.y -= entity1.width / 2;
        assert!(entity1.intersects(&entity2));
        entity1.pos.x += 5;
        entity1.pos.y += 3;
        assert!(entity1.intersects(&entity2));
        // 3D intersections...
        entity2.pos.z = 3;
        assert!(!entity1.intersects(&entity2));
        entity1.depth = 4;
        assert!(entity1.intersects(&entity2));
    }
}
