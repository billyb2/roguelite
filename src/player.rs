use macroquad::math::Vec2;
use crate::{draw::Drawable, attack::Attack};

#[derive(Copy, Clone)]
pub enum PlayerClass {
    Warrior,

}

pub struct Player {
    class: PlayerClass,
    pos: Vec2,
    speed: f32,
    pub attack_cooldown: u16,

}

impl Player {
    pub fn new() -> Self {
        Self {
            pos: Vec2::new(50.0, 50.0),
            class: PlayerClass::Warrior,
            speed: 5.0,
            attack_cooldown: 0,
        }
    }

    pub fn pos(&self) -> Vec2 {
        self.pos

    }

    pub fn speed(&self) -> f32 {
        self.speed

    }

    pub fn class(&self) -> PlayerClass {
        self.class

    }

}

impl Drawable for Player {
    fn pos(&self) -> Vec2 {
        self.pos

    }

    fn size(&self) -> Vec2 {
        Vec2::new(15.0, 15.0)

    }

}

pub fn move_player(player: &mut Player, angle: f32) {
    let direction: Vec2 = angle.sin_cos().into();
    player.pos += direction * player.speed;

}

pub fn update_cooldowns(players: &mut [Player]) {
    players.iter_mut().for_each(|p| {
        p.attack_cooldown = p.attack_cooldown.saturating_sub(1);

    });

}
