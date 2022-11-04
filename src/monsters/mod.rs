mod small_rat;

use std::collections::HashMap;

use crate::{
    draw::Drawable, 
    math::{AsAABB, AxisAlignedBoundingBox}, 
    player::Player, 
    map::{TILE_SIZE, MAP_WIDTH_TILES, MAP_HEIGHT_TILES, Floor}, enchantments::Enchantment
};

use macroquad::prelude::*;

use rayon::prelude::*;
pub use small_rat::*;

#[derive(PartialEq, Eq, Hash)]
struct Effect {
    enchantment: Enchantment,
    frames_left: u8,

}

// All monsters are required to have a drawable AABB and be drawable
pub trait Monster: AsAABB + Drawable + Send {
    fn new(textures: &HashMap<String, Texture2D>, floor: &Floor) -> Self where Self: Sized;
    // Movement and damaging players are seperate so that the movement part can be run in parallel
    fn movement(&mut self, players: &[Player], floor: &Floor);
    fn damage_players(&mut self, players: &mut [Player], floor: &Floor);
    fn take_damage(&mut self, damage: f32, damage_direction: f32, floor: &Floor);
    fn living(&self) -> bool;
    fn as_aabb_obj(&self) -> AxisAlignedBoundingBox{
        self.as_aabb()

    }
    fn apply_enchantment(&mut self, enchantment: Enchantment);
    fn update_enchantments(&mut self);

}

pub fn update_monsters(monsters: &mut Vec<Box<dyn Monster>>, players: &mut [Player], floor: &Floor) {
    // Each thread does 4 monsters at a time, since inidividual monsters aren't too expensive
    monsters.par_chunks_mut(4).for_each(|monsters| {
        // Only move monsters that are within a certain distance of any player
        monsters.iter_mut().for_each(|m| {
            m.update_enchantments();
            m.movement(players, floor)

        });

    });

    monsters.retain_mut(|m| {
        m.damage_players(players, floor);
        m.living()

    })

}
