use std::time::{Duration, Instant};

#[cfg(feature = "native")]
use gilrs::Gilrs;
use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use macroquad::prelude::*;
use macroquad::ui::{root_ui, Skin};

use serde::Serialize;

use crate::attacks::{Attack, AttackObj};
use crate::config::ConfigInfo;

use crate::map::Map;
use crate::math::AsPolygon;

use crate::player::{Player, PlayerClass};
use crate::{CAMERA_ZOOM, DEFAULT_FRAGMENT_SHADER, DEFAULT_VERTEX_SHADER, NUM_PLAYERS};

#[cfg(feature = "native")]
pub struct GamepadInfo {
	pub gilrs: Gilrs,
	pub active_gamepad: Option<gilrs::GamepadId>,
}

#[derive(Clone, Serialize)]
pub struct GameState {
	pub frame: u64,
	pub players: Vec<Player>,
	pub attacks: Vec<AttackObj>,
	pub map: Map,
}

pub struct GameInfo {
	pub accumulator: Duration,
	pub last_update: Instant,

	pub frames_to_skip: u32,

	pub game_state: GameState,
	pub cameras: Vec<Camera2D>,

	#[cfg(feature = "native")]
	pub gamepad_info: GamepadInfo,

	pub viewport_screen_height: f32,
	pub material: Material,
	pub game_started: bool,
	pub in_config: bool,
	pub config_info: ConfigInfo,
}

pub fn init_players(class: PlayerClass, map: &Map, num_players: usize) -> Vec<Player> {
	(0..num_players)
		.into_iter()
		.map(|_| Player::new(class, map.current_floor().current_spawn()))
		.collect()
}

pub fn init_game() -> GameInfo {
	let attacks = Vec::new();
	let map = Map::new();

	let players: Vec<_> = init_players(PlayerClass::Wizard, &map, 1);

	let viewport_screen_height = screen_height(); // * (1.0 / NUM_PLAYERS as f32);

	let cameras: Vec<Camera2D> = players[0..1]
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

	let config_info = ConfigInfo::new("./.game_config").unwrap_or_default();

	GameInfo {
		accumulator: Duration::ZERO,
		last_update: Instant::now(),
		frames_to_skip: 0,
		game_state: GameState {
			frame: 0,
			players,
			attacks,
			map,
		},
		cameras,
		#[cfg(feature = "native")]
		gamepad_info: GamepadInfo {
			active_gamepad,
			gilrs,
		},

		viewport_screen_height,
		material,
		game_started: false,
		in_config: false,
		config_info,
	}
}
