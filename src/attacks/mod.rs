mod slash;
mod stab;

use std::collections::HashMap;

use crate::{
    draw::Drawable, 
    player::Player,
    map::Map,
    monsters::Monster,

};

pub use slash::*;
pub use stab::*;

use macroquad::prelude::*;

pub trait Attack: Drawable + Send + Sync {
    fn new(player: &mut Player, angle: f32, textures: &HashMap<String, Texture2D>, map: &Map) -> Box<Self> where Self: Sized;
    // Returns whether or not the attack should be destroyed
    fn update(&mut self, monsters: &mut [Box<dyn Monster>], map: &Map) -> bool;
    fn cooldown(&self) -> u16;

}

pub fn attack(attack: Box<dyn Attack>, player: &mut Player) {
    if player.attack_cooldown != 0 {
        return;

    }

    player.attack_cooldown = attack.cooldown();
    player.attacks.push(attack);

}

pub fn update_attacks(players: &mut [Player], monsters: &mut [Box<dyn Monster>], map: &Map) {
    players.iter_mut().for_each(|p|  {
        let mut i = 0; 

        while i < p.attacks.len() {
            if p.attacks[i].update(monsters, map) {
                p.attacks.remove(i);

            } else {
                i += 1;

            }
            

        }

    });

}
