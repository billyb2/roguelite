mod blinding_light;
mod magic_missle;
mod slash;
mod stab;

use std::collections::HashMap;

use crate::{draw::Drawable, map::Floor, monsters::Monster, player::Player};

pub use blinding_light::*;
pub use magic_missle::*;
pub use slash::*;
pub use stab::*;

use macroquad::prelude::*;

pub trait Attack: Drawable + Send + Sync {
    fn new(
        player: &mut Player,
        angle: f32,
        textures: &HashMap<String, Texture2D>,
        floor: &Floor,
        is_primary: bool,
    ) -> Box<Self>
    where
        Self: Sized;
    // Returns whether or not the attack should be destroyed
    fn update(&mut self, monsters: &mut [Box<dyn Monster>], floor: &Floor) -> bool;
    fn cooldown(&self) -> u16;
}

pub fn attack(attack: Box<dyn Attack>, player: &mut Player, primary_attack: bool) {
    let cooldown = match primary_attack {
        true => &mut player.primary_cooldown,
        false => &mut player.secondary_cooldown,
    };

    if *cooldown != 0 {
        return;
    }

    *cooldown = attack.cooldown();
    player.attacks.push(attack);
}

pub fn update_attacks(players: &mut [Player], monsters: &mut [Box<dyn Monster>], floor: &Floor) {
    players.iter_mut().for_each(|p| {
        let mut i = 0;

        while i < p.attacks.len() {
            if p.attacks[i].update(monsters, floor) {
                p.attacks.remove(i);
            } else {
                i += 1;
            }
        }
    });
}
