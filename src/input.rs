use crate::attacks::{Attack, AttackObj};

use crate::map::FloorInfo;
use crate::math::{get_angle, AsPolygon};
use crate::player::{move_player, player_attack, Player};
use bytemuck::{Pod, Zeroable};
#[cfg(feature = "native")]
use gilrs::{Axis, Button, Gamepad};
use macroquad::prelude::*;

type FlagSize = u32;

const PRIMARY_ATTACK: FlagSize = 0b1;
const SECONDARY_ATTACK: FlagSize = 0b10;
const MOVING: FlagSize = 0b100;
const OPENING_DOOR: FlagSize = 0b1000;
const CLOSING_DOOR: FlagSize = 0b10000;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Pod, Zeroable)]
pub struct PlayerInput {
	movement_angle: f32,
	rotation: f32,
	flags: FlagSize,
}

impl PlayerInput {
	pub fn movement_angle(&self) -> f32 { self.movement_angle }

	pub fn rotation(&self) -> f32 { self.rotation }

	fn set_primary_attacking(&mut self) { self.flags |= PRIMARY_ATTACK; }

	fn set_secondary_attacking(&mut self) { self.flags |= SECONDARY_ATTACK; }

	fn set_moving(&mut self) { self.flags |= MOVING; }

	fn set_opening_door(&mut self) { self.flags |= OPENING_DOOR }

	fn set_closing_door(&mut self) { self.flags |= CLOSING_DOOR }

	pub fn using_primary(&self) -> bool { self.flags & PRIMARY_ATTACK == PRIMARY_ATTACK }

	pub fn using_secondary(&self) -> bool { self.flags & SECONDARY_ATTACK == SECONDARY_ATTACK }

	pub fn is_moving(&self) -> bool { self.flags & MOVING == MOVING }

	pub fn opening_door(&self) -> bool { self.flags & OPENING_DOOR == OPENING_DOOR }

	pub fn closing_door(&self) -> bool { self.flags & CLOSING_DOOR == CLOSING_DOOR }
}

impl Default for PlayerInput {
	fn default() -> Self { Self::zeroed() }
}

pub fn movement_input(player: &Player, _index: Option<usize>, camera: &Camera2D) -> PlayerInput {
	let mut input = PlayerInput::default();

	if player.hp() == 0 {
		return input;
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

	/*
	if is_key_down(KeyCode::Z) {
		player.changing_spell = true;
		player.time_til_change_spell = 15;
	}
	*/

	let mouse_pos: Vec2 = mouse_position().into();

	let rotation = get_angle(mouse_pos, camera.world_to_screen(player.center()));

	input.rotation = rotation;

	/*

	if player.get_item_selection_type() != Some(&SelectionType::Selected) {
		let mut possible_selected_item = (0..player.inventory().items.len()).find_map(|i| {
			let pos = item_pos_from_index(i);
			const HALF_INVENTORY_SIZE: Vec2 =
				Vec2::new(ITEM_INVENTORY_SIZE.x * 0.5, ITEM_INVENTORY_SIZE.y * 0.5);
			let polygon = easy_polygon(pos + HALF_INVENTORY_SIZE, HALF_INVENTORY_SIZE, 0.0);

			match point_in_polygon(&polygon, mouse_pos) {
				true => Some(ItemSelectedInfo {
					index: i,
					selection_type: match is_mouse_button_pressed(MouseButton::Left) {
						true => SelectionType::Selected,
						false => SelectionType::Hovered,
					},
				}),
				false => None,
			}
		});

		if let Some(selected_item) = &possible_selected_item {
			if selected_item.selection_type == SelectionType::Selected {
				let item = player
					.inventory()
					.items
					.get(selected_item.index)
					.unwrap()
					.clone();

				if let Some(use_item_fn) = use_item(&item.item_type) {
					use_item_fn(&item, player, &mut floor_info.floor);
					player.inventory.items.remove(selected_item.index);
					possible_selected_item = None;
				}
			}
		}

		player.set_selected_item(possible_selected_item);
	}
	*/

	if is_mouse_button_down(MouseButton::Left) {
		input.set_primary_attacking();
	}

	if is_mouse_button_down(MouseButton::Right) {
		input.set_secondary_attacking();
	}

	if is_key_pressed(KeyCode::O) {
		input.set_opening_door();
	}

	if is_key_pressed(KeyCode::C) {
		input.set_closing_door();
	}

	/*
	if is_key_down(KeyCode::LeftShift) {
		pickup_items(player, &mut floor_info.floor);
	}

	if is_key_pressed(KeyCode::I) {
		toggle_inventory(player);
	}
	*/

	if x_movement != 0.0 || y_movement != 0.0 {
		input.movement_angle = get_angle(Vec2::new(x_movement, y_movement), Vec2::ZERO);
		input.set_moving();
	}

	input
}

#[cfg(feature = "native")]
pub fn movement_input_controller(
	player: &mut Player, index: Option<usize>, attacks: &mut Vec<AttackObj>,
	floor_info: &mut FloorInfo, gamepad: &Gamepad,
) {
	let x_movement = gamepad
		.axis_data(Axis::LeftStickX)
		.map(|a| a.value())
		.unwrap_or_default();

	let y_movement = -gamepad
		.axis_data(Axis::LeftStickY)
		.map(|a| a.value())
		.unwrap_or_default();

	if x_movement.abs() > f32::EPSILON || y_movement.abs() > f32::EPSILON {
		let angle = y_movement.atan2(x_movement);
		move_player(player, angle, None, &floor_info.floor);
	}

	let x_movement_r = gamepad
		.axis_data(Axis::RightStickX)
		.map(|a| a.value())
		.unwrap_or_default();

	let y_movement_r = -gamepad
		.axis_data(Axis::RightStickY)
		.map(|a| a.value())
		.unwrap_or_default();

	player.angle = y_movement_r.atan2(x_movement_r);

	if let Some(button_data) = gamepad.button_data(Button::LeftTrigger2) {
		if button_data.is_pressed() {
			player_attack(player, index, attacks, floor_info, false);
		}
	}

	if let Some(button_data) = gamepad.button_data(Button::RightTrigger2) {
		if button_data.is_pressed() {
			player_attack(player, index, attacks, floor_info, true);
		}
	}
}
