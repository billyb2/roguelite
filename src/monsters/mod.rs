mod small_rat;

use std::collections::HashMap;

use crate::{
    draw::Drawable, 
    math::AsAABB, 
    player::Player, 
    map::Map
};

use macroquad::prelude::*;

pub use small_rat::*;

// All monsters are required to have a drawable AABB and be drawable
pub trait Monster: AsAABB + Drawable {
    fn new(textures: &HashMap<String, Texture2D>, map: &Map) -> Self where Self: Sized;
    fn ai(&mut self, players: &mut [Player], map: &Map);

}

pub fn update_monsters(monsters: &mut [Box<dyn Monster>], players: &mut [Player], map: &Map) {
    monsters.iter_mut().for_each(|m| m.ai(players, map));

}

const fn seconds_to_frames(time: f32) -> u16 {
    let time_as_frames = time * 60.0;

    if time_as_frames <= u16::MAX as f32 {
        time_as_frames as u16

    } else {
        panic!("Value too large");

    }


}