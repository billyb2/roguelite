use std::collections::HashMap;

use crate::{
    player::{Player, move_player}, 
    attack::{Attack, player_attack, AttackType}, math::get_angle, map::Map,
};
use macroquad::prelude::*;

pub fn keyboard_input(player: &mut Player, attacks: &mut Vec<Attack>, textures: &HashMap<String, Texture2D>, map: &Map) {
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
        let mouse_pos = mouse_position_local();
        let angle = get_angle(mouse_pos.x, mouse_pos.y, 0.0, 0.0);

        let texture = textures.get("swipe.webp").unwrap();

        player_attack(player, 0, AttackType::Primary, attacks, angle, Some(*texture));

    }

    if is_mouse_button_down(MouseButton::Right) {
        let mouse_pos = mouse_position_local();
        let angle = get_angle(mouse_pos.x, mouse_pos.y, 0.0, 0.0);

        let texture = textures.get("stab.webp").unwrap();

        player_attack(player, 0, AttackType::Secondary, attacks, angle, Some(*texture));

    }

    if x_movement != 0.0 || y_movement != 0.0 {
        let angle = y_movement.atan2(x_movement);
        move_player(player, angle, None, map)

    }

}
