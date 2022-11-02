mod small_rat;

use std::collections::HashMap;

use crate::{
    draw::Drawable, 
    math::{AsAABB, AxisAlignedBoundingBox}, 
    player::Player, 
    map::{Map, TILE_SIZE, MAP_WIDTH_TILES, MAP_HEIGHT_TILES}
};

use macroquad::prelude::*;

use rayon::prelude::{IntoParallelRefMutIterator, ParallelIterator};
pub use small_rat::*;

// All monsters are required to have a drawable AABB and be drawable
pub trait Monster: AsAABB + Drawable + Send {
    fn new(textures: &HashMap<String, Texture2D>, map: &Map) -> Self where Self: Sized;
    // Movement and damaging players are seperate so that the movement part can be run in parallel
    fn movement(&mut self, players: &[Player], map: &Map);
    fn damage_players(&mut self, players: &mut [Player], map: &Map);
    fn take_damage(&mut self, damage: f32, damage_direction: f32, map: &Map);
    fn living(&self) -> bool;
    fn into_aabb_obj(&self) -> AxisAlignedBoundingBox{
        self.as_aabb()

    }

}

pub fn update_monsters(monsters: &mut Vec<Box<dyn Monster>>, players: &mut [Player], map: &Map) {
    monsters.par_iter_mut().for_each(|m| {
        // Only move monsters that are within a certain distance of any player
        let close_to_a_player = players.iter().any(|p| {
            p.pos().distance(m.pos()) <= (TILE_SIZE * (MAP_WIDTH_TILES + MAP_HEIGHT_TILES) / 2) as f32 * 0.25

        });

        if close_to_a_player {
            m.movement(players, map);

        }

    });

    let mut i  = 0;

    while i < monsters.len() {
        monsters[i].damage_players(players, map);

        if !monsters[i].living() {
            monsters.remove(i);

        } else {
            i += 1;

        }

    }

}
