mod chip;
mod debug;
mod decode;
mod get_line;
mod hex;
mod types;

use crate::chip::Chip;

fn main() {
	println!("Hello, CHIP 8");

	let args: Vec<String> = std::env::args().collect();
	let mut debug = false;
	let mut path = None;

	for arg in args[1..].iter() {
		if arg.eq("-d") {
			debug = true;
		} else {
			path = Some(arg.clone());
		}
	}

	if path.is_none() {
		println!("usage: nn [-d] rom_path");
		return;
	}

	let rom = open_rom(&path.unwrap());

	if debug {
		debug::debug(rom);
	} else {
		let mut chip = Chip::new();
		chip.load_rom(rom);
		chip.run();
	}
}

fn open_rom(path: &str) -> Vec<u8> {
	std::fs::read(path).unwrap()
}
