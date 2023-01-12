use crate::attacks::Attack;
use crate::draw::Textures;
use crate::items::use_item;
use crate::map::FloorInfo;
use crate::math::{easy_polygon, get_angle, point_in_polygon, AsPolygon};
use crate::player::{
	interact_with_door,
	item_pos_from_index,
	move_player,
	pickup_items,
	player_attack,
	toggle_inventory,
	DoorInteraction,
	ItemSelectedInfo,
	Player,
	SelectionType,
	ITEM_INVENTORY_SIZE,
};
use crate::NUM_PLAYERS;
use gilrs::{Axis, Button, Gamepad};
use macroquad::prelude::*;

pub fn movement_input(
	player: &mut Player, index: Option<usize>, attacks: &mut Vec<Box<dyn Attack>>,
	textures: &Textures, floor_info: &mut FloorInfo, camera: &Camera2D,
) {
	if player.hp() == 0 {
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

	if is_key_down(KeyCode::Z) {
		player.changing_spell = true;
		player.time_til_change_spell = 15;
	}

	let mouse_pos: Vec2 = mouse_position().into();

	player.angle = get_angle(
		mouse_pos,
		match NUM_PLAYERS == 1 {
			true => camera.world_to_screen(player.center()),
			false => {
				camera.world_to_screen(player.center()) +
					Vec2::new(
						0.0,
						(camera.viewport.unwrap().3 as f32) * (1.0 / NUM_PLAYERS as f32),
					)
			},
		},
	);

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

	if is_mouse_button_down(MouseButton::Left) {
		player_attack(player, index, textures, attacks, floor_info, true);
	}

	if is_mouse_button_down(MouseButton::Right) {
		player_attack(player, index, textures, attacks, floor_info, false);
	}

	if is_key_pressed(KeyCode::P) {
		pickup_items(player, &mut floor_info.floor);
	}

	if is_key_pressed(KeyCode::I) {
		toggle_inventory(player);
	}

	if x_movement != 0.0 || y_movement != 0.0 {
		let angle = y_movement.atan2(x_movement);
		move_player(player, angle, None, &floor_info.floor);
	}
}

pub fn movement_input_controller(
	player: &mut Player, index: Option<usize>, attacks: &mut Vec<Box<dyn Attack>>,
	textures: &Textures, floor_info: &mut FloorInfo, gamepad: &Gamepad,
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
			player_attack(player, index, textures, attacks, floor_info, false);
		}
	}

	if let Some(button_data) = gamepad.button_data(Button::RightTrigger2) {
		if button_data.is_pressed() {
			player_attack(player, index, textures, attacks, floor_info, true);
		}
	}
}

pub fn door_interaction_input(
	player: &Player, players: &[Player], floor: &mut FloorInfo, textures: &Textures,
) {
	if is_key_pressed(KeyCode::O) {
		interact_with_door(player, players, DoorInteraction::Opening, floor, textures);
	}

	if is_key_pressed(KeyCode::C) {
		interact_with_door(player, players, DoorInteraction::Toggle, floor, textures);
	}
}

pub fn door_interaction_input_controller(
	player: &Player, players: &[Player], floor: &mut FloorInfo, textures: &Textures,
	gamepad: &Gamepad,
) {
	if let Some(button_data) = gamepad.button_data(Button::South) {
		if button_data.is_pressed() {
			interact_with_door(player, players, DoorInteraction::Opening, floor, textures);
		}
	}
}
