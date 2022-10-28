use std::collections::HashMap;

use crate::{
    player::{Player, move_player}, 
    math::get_angle,
    map::Map, 
    attacks::*,
};
use macroquad::prelude::*;

pub fn keyboard_input(player: &mut Player, textures: &HashMap<String, Texture2D>, map: &Map) {
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

    let mouse_pos = mouse_position_local();
    player.angle = get_angle(mouse_pos.x, mouse_pos.y, 0.0, 0.0);

    if is_mouse_button_down(MouseButton::Left) {
        let slash = Slash::new(player, player.angle, textures, map);
        attack(slash, player);


    }

    if is_mouse_button_down(MouseButton::Right) {
        let stab = Stab::new(player, player.angle, textures, map);
        attack(stab, player);

    }

    if x_movement != 0.0 || y_movement != 0.0 {
        let angle = y_movement.atan2(x_movement);
        move_player(player, angle, None, map)

    }

}
