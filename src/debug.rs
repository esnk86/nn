use std::io::{self, Write};

use crate::chip::Chip;
use crate::get_line::get_line;

pub fn debug(rom: Vec<u8>) {
	println!("Debug mode");

	let mut chip = Chip::new();

	chip.load_rom(rom);

	prompt();
	while let Some(line) = get_line() {
		match line.trim_start().chars().next() {
			Some('.') => chip.dump_next_instruction(),
			Some('r') => chip.dump_registers(),
			Some('m') => chip.dump_memory(),
			Some('c') => chip.dump_stack(),
			Some('s') => chip.step(),
			Some('h') => help(),
			Some('q') => break,
			Some(_) => println!("Unknown command."),
			None => (),
		}
		prompt();
	}
}

fn prompt() {
	print!("> ");
	io::stdout().flush();
}

fn help() {
	println!(". - dump next instruction");
	println!("r - dump registers");
	println!("m - dump memory");
	println!("c - dump call stack");
	println!("s - step program by one instruction");
	println!("h - help");
	println!("q - quit");
}
