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
            let collisions = self.find_collisions();
            self.collision_handlers(collisions);
        }
        self.remove_deleted_entries(deleted);
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
        for mut entity in entities.values_mut() {
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
                && (entity.pos.x >= self.width
                    || entity.pos.y >= self.height
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
    fn collision_handlers(&mut self, collisions: Collisions) {
        for collision in collisions {
            let entities = (
                self.entities.remove_entry(&collision.0),
                self.entities.remove_entry(&collision.1),
            );
            match entities {
                (Some((key1, mut ent1)), Some((key2, mut ent2))) => {
                    // Process...
                    if let Some(callback) = ent1.coll_handler.take() {
                        callback(&mut ent1, self, &ent2);
                        ent1.coll_handler = Some(callback);
                    }
                    if let Some(callback) = ent2.coll_handler.take() {
                        callback(&mut ent2, self, &ent1);
                        ent2.coll_handler = Some(callback);
                    }
                    // Put them back in
                    self.entities.insert(key1, ent1);
                    self.entities.insert(key2, ent2);
                }
                (Some((key, ent)), None) | (None, Some((key, ent))) => {
                    self.entities.insert(key, ent);
                }
                (None, None) => {
                    panic!("Something is very wrong; collision failed: entities not found.")
                }
            };
        }
    }
    fn remove_deleted_entries(&mut self, deleted: DeletedList) {
        for ent_name in deleted {
            if let Some(mut entity) = self.entities.remove(&ent_name) {
                // Entity practically deleted at this point...
                if let Some(callback) = entity.death_callback.take() {
                    callback(&mut entity, self);
                }
            }
        }
    }
    fn move_followers(&mut self) {
        let following_entities: Vec<(Entity, String)> = self
            .entities
            .values()
            .cloned()
            .filter_map(|mut ent| {
                let follow_entity = ent.follow_entity.take()?;
                Some((ent, follow_entity))
            })
            .collect();
        for (mut follower, follow_entity_name) in following_entities {
            if let Some(leader) = self.entities.get(&follow_entity_name) {
                // Process follow
                if let Some(x) = follower.follow_offset.x {
                    follower.set_x(x + leader.pos.x, self.width);
                }
                if let Some(y) = follower.follow_offset.y {
                    follower.set_y(y + leader.pos.y, self.height);
                }
                if let Some(z) = follower.follow_offset.z {
                    follower.set_z(z + leader.pos.z);
                }
                if let Some(frame) = follower.follow_offset.frame {
                    follower.set_frame(frame + leader.current_frame);
                }
            }
            // Put the values back in
            follower.follow_entity = Some(follow_entity_name);
            if let Some(entry) = self.entities.get_mut(&follower.name) {
                *entry = follower;
            }
        }
    }
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
