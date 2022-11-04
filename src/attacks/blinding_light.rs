use std::collections::HashMap;

use crate::{player::Player, draw::Drawable, map::Floor, monsters::Monster, math::{AsAABB, AxisAlignedBoundingBox, aabb_collision, get_angle}, enchantments::{Enchantment, EnchantmentKind}};
use macroquad::prelude::*;

use super::Attack;

const SIZE: Vec2 = Vec2::new(75.0, 25.0);

pub struct BlindingLight {
    pos: Vec2,
    angle: f32,
    texture: Texture2D,
    time: u16,

}

impl Attack for BlindingLight {
    fn new(player: &mut Player, angle: f32, textures: &HashMap<String, Texture2D>, floor: &Floor, is_primary: bool) -> Box<Self> {        
        Box::new(Self {
            pos: player.pos() + (Vec2::new(angle.cos(), angle.sin()) * SIZE),
            angle,
            texture: *textures.get("blinding_light.webp").unwrap(),
            time: 0,

        })
    }

    fn update(&mut self, monsters: &mut [Box<dyn Monster>], _floor: &Floor) -> bool {
        self.time += 1;

        if self.time >= 60 {
            return true;

        }

        // Check to see if it's collided with a monster
        monsters.iter_mut().filter(|m| aabb_collision(self, &m.as_aabb(), Vec2::ZERO)).for_each(|monster| {
            monster.apply_enchantment(Enchantment {
                kind: EnchantmentKind::Blinded,
                level: 0,
            });

        });


        false

    }

    fn cooldown(&self) -> u16 {
        60 * 8
    }

}

impl AsAABB for BlindingLight {
    fn as_aabb(&self) -> AxisAlignedBoundingBox {
        AxisAlignedBoundingBox {
            pos: self.pos, 
            size: SIZE,
        }
    }

}

impl Drawable for BlindingLight {
    fn pos(&self) -> Vec2 {
        self.pos

    }

    fn size(&self) -> Vec2 {
        SIZE
    }

    fn rotation(&self) -> f32 {
        self.angle
    }

    fn texture(&self) -> Option<Texture2D> {
        Some(self.texture)
        
    }

}

