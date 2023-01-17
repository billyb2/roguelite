use gilrs::Gilrs;
use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use macroquad::prelude::*;
use macroquad::ui::{root_ui, Skin};

use crate::attacks::Attack;
use crate::map::{Map, Object};
use crate::math::AsPolygon;
use crate::player::{Player, PlayerClass, PlayerConfigInfo};
use crate::{CAMERA_ZOOM, DEFAULT_FRAGMENT_SHADER, DEFAULT_VERTEX_SHADER, NUM_PLAYERS};

#[cfg(feature = "native")]
pub struct GamepadInfo {
	pub gilrs: Gilrs,
	pub active_gamepad: Option<gilrs::GamepadId>,
}

pub struct GameInfo {
	pub players: Vec<Player>,
	pub attacks: Vec<Box<dyn Attack>>,
	pub map: Map,
	pub cameras: Vec<Camera2D>,
	#[cfg(feature = "native")]
	pub gamepad_info: GamepadInfo,
	pub viewport_screen_height: f32,
	pub material: Material,
	pub visible_objects: Vec<Vec<Object>>,
	pub game_started: bool,
	pub in_config: bool,
	pub config_info: PlayerConfigInfo,
}

pub fn init_players(class: PlayerClass, map: &Map) -> Vec<Player> {
	(0..NUM_PLAYERS)
		.into_iter()
		.map(|i| Player::new(i, class, map.current_floor().current_spawn()))
		.collect()
}

pub fn init_game() -> GameInfo {
	let attacks = Vec::new();
	let map = Map::new();

	let players: Vec<_> = init_players(PlayerClass::Wizard, &map);

	let viewport_screen_height = screen_height() * (1.0 / NUM_PLAYERS as f32);

	let cameras: Vec<Camera2D> = players
		.iter()
		.enumerate()
		.map(|(i, p)| Camera2D {
			target: p.center(),
			zoom: Vec2::new(
				CAMERA_ZOOM,
				-CAMERA_ZOOM * (screen_width() / viewport_screen_height),
			) * 0.7,
			viewport: Some((
				0,
				viewport_screen_height as i32 * i as i32,
				screen_width() as i32,
				viewport_screen_height as i32,
			)),
			..Default::default()
		})
		.collect();

	#[cfg(feature = "native")]
	let gilrs = Gilrs::new().unwrap();
	#[cfg(feature = "native")]
	let active_gamepad = None;

	let fragment_shader = DEFAULT_FRAGMENT_SHADER.to_string();
	let vertex_shader = DEFAULT_VERTEX_SHADER.to_string();

	let pipeline_params = PipelineParams {
		depth_write: true,
		depth_test: Comparison::LessOrEqual,
		color_blend: Some(BlendState::new(
			Equation::Add,
			BlendFactor::Value(BlendValue::SourceAlpha),
			BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
		)),
		..Default::default()
	};

	let material = load_material(
		&vertex_shader,
		&fragment_shader,
		MaterialParams {
			pipeline_params,
			uniforms: vec![
				("lowest_light_level".to_string(), UniformType::Float1),
				("window_height".to_string(), UniformType::Float1),
			],
			..Default::default()
		},
	)
	.unwrap();

	let label_style = root_ui().style_builder().text_color(WHITE).build();
	let skin = Skin {
		label_style,
		..root_ui().default_skin()
	};

	root_ui().push_skin(&skin);

	GameInfo {
		players,
		attacks,
		map,
		cameras,
		gamepad_info: GamepadInfo {
			active_gamepad,
			gilrs,
		},
		viewport_screen_height,
		material,
		visible_objects: Vec::new(),
		game_started: false,
		in_config: false,
		config_info: PlayerConfigInfo {
			class: PlayerClass::Warrior,
		},
	}
}
