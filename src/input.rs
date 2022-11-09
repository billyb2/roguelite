use std::collections::HashMap;

use crate::attacks::*;
use crate::map::Floor;
use crate::math::get_angle;
use crate::monsters::Monster;
use crate::player::{
	interact_with_door, move_player, primary_attack, secondary_attack, DoorInteraction, Player,
};
use macroquad::prelude::*;

pub fn keyboard_input(
	player: &mut Player, monsters: &mut [Box<dyn Monster>], textures: &HashMap<String, Texture2D>,
	floor: &mut Floor,
) {
	if player.health() == 0.0 {
		return;
	}

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

	if is_key_pressed(KeyCode::O) {
		interact_with_door(player, DoorInteraction::Opening, floor);
	}

	if is_key_pressed(KeyCode::C) {
		interact_with_door(player, DoorInteraction::Closing, floor);
	}

	let mouse_pos = mouse_position_local();
	player.angle = get_angle(mouse_pos.x, mouse_pos.y, 0.0, 0.0);

	if is_mouse_button_down(MouseButton::Left) {
		attack(
			primary_attack(player, textures, monsters, floor),
			player,
			true,
		);
	}

	if is_mouse_button_down(MouseButton::Right) {
		attack(
			secondary_attack(player, textures, monsters, floor),
			player,
			false,
		);
	}

	if x_movement != 0.0 || y_movement != 0.0 {
		let angle = y_movement.atan2(x_movement);
		move_player(player, angle, None, floor)
	}
}
