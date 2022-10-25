use std::collections::HashMap;

use crate::{
    player::{Player, move_player}, 
    attack::{Attack, player_attack, AttackType}, math::get_angle,
};
use macroquad::prelude::*;

pub fn keyboard_input(player: &mut Player, attacks: &mut Vec<Attack>, textures: &HashMap<String, Texture2D>) {
    let mut x_movement: f32 = 0.0;
    let mut y_movement: f32 = 0.0;

    if is_key_down(KeyCode::W) {
        y_movement -= 1.0;

    }

    if is_key_down(KeyCode::S) {
        y_movement += 1.0;

    }

    if is_key_down(KeyCode::A) {
        x_movement -= 1.0;

    }

    if is_key_down(KeyCode::D) {
        x_movement += 1.0;

    }

    if is_mouse_button_down(MouseButton::Left) {
        let player_pos = player.pos();
        let mouse_pos = mouse_position();

        let angle = get_angle(mouse_pos.0, mouse_pos.1, player_pos.x, player_pos.y);

        let texture = textures.get("swipe.webp").unwrap();

        player_attack(player, AttackType::Primary, attacks, angle, Some(*texture));

    }

    if is_mouse_button_down(MouseButton::Right) {
        let player_pos = player.pos();
        let mouse_pos = mouse_position();

        let angle = get_angle(mouse_pos.0, mouse_pos.1, player_pos.x, player_pos.y);

        let texture = textures.get("stab.webp").unwrap();

        player_attack(player, AttackType::Secondary, attacks, angle, Some(*texture));

    }

    if x_movement != 0.0 || y_movement != 0.0 {
        let angle = x_movement.atan2(y_movement);
        move_player(player, angle)

    }

}
