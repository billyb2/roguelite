use std::path::Path;
use std::{env, fs};

fn main() {
	let mut assets_string = "[".to_string();
	let mut num_assets = 0;

	fs::read_dir("assets")
		.unwrap()
		.filter_map(|file| {
			if let Ok(file) = file {
				let file_name = file.file_name().to_str().unwrap().to_string();
				Some(file_name)
			} else {
				None
			}
		})
		.for_each(|asset| {
			num_assets += 1;

			assets_string.push('"');
			assets_string.push_str(&asset);
			assets_string.push('"');

			assets_string.push(',');
		});

	assets_string.push(']');

	let out_dir = env::var_os("OUT_DIR").unwrap();
	let dest_path = Path::new(&out_dir).join("assets.rs");

	fs::write(
		dest_path,
		format!("const ASSETS: [&str; {num_assets}] = {assets_string};"),
	)
	.unwrap()
}
