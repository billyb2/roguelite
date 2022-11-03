mod small_rat;

use std::collections::HashMap;

use crate::{
    draw::Drawable, 
    math::{AsAABB, AxisAlignedBoundingBox}, 
    player::Player, 
    map::{TILE_SIZE, MAP_WIDTH_TILES, MAP_HEIGHT_TILES, Floor}
};

use macroquad::prelude::*;

use rayon::{prelude::{IntoParallelRefMutIterator, ParallelIterator}, slice::ParallelSliceMut};
pub use small_rat::*;

// All monsters are required to have a drawable AABB and be drawable
pub trait Monster: AsAABB + Drawable + Send {
    fn new(textures: &HashMap<String, Texture2D>, floor: &Floor) -> Self where Self: Sized;
    // Movement and damaging players are seperate so that the movement part can be run in parallel
    fn movement(&mut self, players: &[Player], floor: &Floor);
    fn damage_players(&mut self, players: &mut [Player], floor: &Floor);
    fn take_damage(&mut self, damage: f32, damage_direction: f32, floor: &Floor);
    fn living(&self) -> bool;
    fn into_aabb_obj(&self) -> AxisAlignedBoundingBox{
        self.as_aabb()

    }

}

pub fn update_monsters(monsters: &mut Vec<Box<dyn Monster>>, players: &mut [Player], floor: &Floor) {
    // Each thread does 4 monsters at a time, since inidividual monsters aren't too expensive
    monsters.par_chunks_mut(4).for_each(|monsters| {
        // Only move monsters that are within a certain distance of any player
        monsters.iter_mut().for_each(|m| m.movement(players, floor));

    });

    let mut i  = 0;

    while i < monsters.len() {
        monsters[i].damage_players(players, floor);

        if !monsters[i].living() {
            monsters.remove(i);

        } else {
            i += 1;

        }

    }

}
