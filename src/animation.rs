use crossterm::style::Color;
use std::collections::{HashMap, HashSet};
use std::io::Stdout;

use crate::entity::{CallbackResult, Entity};

type DeletedList = HashSet<String>;
type Collision = (String, String);
type Collisions = HashSet<Collision>;

pub struct Animation {
    // in theory, we'll only need this for storing entities...
    pub entities: HashMap<String, Entity>,
    pub physical_count: usize,
    // ...and these are here for the future
    pub track_framerate: bool,
    framerate: u16,
    pub frames_this_second: u16,
    // target output thing to write to
    pub target: Stdout,
    // general terminal stuff
    pub width: i16,
    pub height: i16,
    pub assumed_size: bool,
    pub bg: Option<Color>,
}

impl Animation {
    pub fn new(target: Option<Stdout>) -> Self {
        let (width, height, assumed_size) = Self::get_term_size();
        Self {
            entities: HashMap::new(),
            physical_count: 0,
            track_framerate: false,
            framerate: 0,
            frames_this_second: 0,
            target: target.unwrap_or_else(std::io::stdout),
            width,
            height,
            assumed_size,
            bg: None,
        }
    }
    pub fn set_track_framerate(&mut self, track_framerate: bool) -> &mut Self {
        self.track_framerate = track_framerate;
        self
    }
    pub fn background(&mut self, bg: Color) -> &mut Self {
        self.bg = Some(bg);
        self
    }
    pub fn framerate(&self) -> u16 {
        self.framerate
    }
    pub fn add_entity(&mut self, entity: Entity) {
        if entity.physical {
            self.physical_count += 1;
        }
        self.entities.insert(entity.name.clone(), entity);
    }
    pub fn animate(&mut self) {
        let mut deleted = DeletedList::new();
        deleted.extend(self.do_callbacks());
        if self.physical_count > 0 {
            self.find_collisions();
            self.collision_handlers();
        }
        self.remove_deleted_entries();
        self.move_followers();
        self.build_screen();
        self.display_screen();
        if self.track_framerate {
            self.track_framerate();
        }
    }
}

// Internal methods
impl Animation {
    fn do_callbacks(&mut self) -> DeletedList {
        let mut deleted = DeletedList::new();
        let all_ents: Vec<String> = self.entities.keys().cloned().collect();
        let mut entities = HashMap::with_capacity(self.entities.capacity());
        // Pull out the entities into a new hashmap.
        std::mem::swap(&mut entities, &mut self.entities);
        for (_, mut entity) in entities.iter_mut() {
            if let Some(ref _time) = entity.die_time {
                todo!("Handling die_time is not implemented yet!")
            }
            if let Some(ref mut frame) = entity.die_frame {
                *frame -= 1;
                if *frame <= 0 {
                    deleted.insert(entity.name.clone());
                    continue;
                }
            }
            if let Some(ref mut die_entity) = entity.die_entity {
                // If we don't know that guy anymore, or we know he's gonna die...
                if !all_ents.contains(die_entity) || deleted.contains(die_entity) {
                    deleted.insert(entity.name.clone());
                    continue;
                }
            }
            if entity.die_offscreen
                // If our width or height is higher than 32.767, we have other problems.
                //                  v                     v
                && (entity.pos.x >= (self.width as i16)
                    || entity.pos.y >= (self.height as i16)
                    || entity.pos.x < -entity.width
                    || entity.pos.y < -entity.height)
            {
                deleted.insert(entity.name.clone());
                continue;
            }
            if let Some(callback) = entity.callback.take() {
                let CallbackResult {
                    new_x,
                    new_y,
                    new_z,
                    new_frame,
                } = callback(entity, self);
                if let Some(x) = new_x {
                    entity.set_x(x, self.width);
                }
                if let Some(y) = new_y {
                    entity.set_y(y, self.height);
                }
                if let Some(z) = new_z {
                    entity.set_z(z);
                }
                if let Some(frame) = new_frame {
                    entity.set_frame(frame);
                }
                entity.callback = Some(callback);
            }
        }
        // And put them back in.
        std::mem::swap(&mut entities, &mut self.entities);
        deleted
    }
    fn find_collisions(&self) -> Collisions {
        let mut collisions = Collisions::new();
        for me in self.entities.values() {
            if !me.physical {
                continue;
            }
            for other in self.entities.values() {
                if other.name == me.name {
                    // Don't check for self
                    continue;
                }
                if me.intersects(other) {
                    let already_there = collisions.iter().any(|(ent1, ent2)| {
                        ent1 == &me.name && ent2 == &other.name
                            || ent1 == &other.name && ent2 == &me.name
                    });
                    if !already_there {
                        collisions.insert((me.name.clone(), other.name.clone()));
                    }
                }
            }
        }
        collisions
    }
    fn collision_handlers(&mut self) {}
    fn remove_deleted_entries(&mut self) {}
    fn move_followers(&mut self) {}
    fn build_screen(&mut self) {}
    fn display_screen(&mut self) {}
    fn track_framerate(&mut self) {}
}

// Internal helper functions
impl Animation {
    fn get_term_size() -> (i16, i16, bool) {
        crossterm::terminal::size()
            .map(|(width, height)| (width as i16, height as i16, false))
            .unwrap_or_else(|_| (80, 24, true))
    }
}
