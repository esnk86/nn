#[allow(unused_imports)]
use minifb::{Key, Window, WindowOptions};

use std::num::Wrapping;

use crate::decode::{self, Decoded};
use crate::hex;
use crate::types::*;

const MEMORY_SIZE: usize = 4096;
const PROGRAM_MEMORY_OFFSET: usize = 512;
const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const SQUARE_SIZE: usize = 10;
const WINDOW_WIDTH: usize = SQUARE_SIZE * DISPLAY_WIDTH;
const WINDOW_HEIGHT: usize = SQUARE_SIZE * DISPLAY_HEIGHT;

pub struct Chip {
	memory: Vec<Byte>,
	pc: Address,
	i: Address,
	v: [Byte; 16],
	stack: Vec<Address>,
	display: Vec<bool>,
	window: Window,
	frame_buffer: Vec<u32>,
}

impl Chip {
	pub fn new() -> Self {
		Self {
			memory: vec![0; MEMORY_SIZE],
			pc: PROGRAM_MEMORY_OFFSET as Address,
			i: 0,
			v: [0; 16],
			stack: Vec::new(),
			display: vec![false; DISPLAY_WIDTH * DISPLAY_HEIGHT],
			window: Self::new_window(),
			frame_buffer: vec![0; WINDOW_WIDTH * WINDOW_HEIGHT],
		}
	}

	fn new_window() -> Window {
		let mut window = Window::new(
			"CHIP 8",
			WINDOW_WIDTH,
			WINDOW_HEIGHT,
			WindowOptions::default(),
		)
		.unwrap_or_else(|e| {
			panic!("{}", e);
		});

		window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

		window
	}

	pub fn load_rom(&mut self, bytes: Vec<u8>) {
		for (i, byte) in bytes.iter().enumerate() {
			self.memory[i + PROGRAM_MEMORY_OFFSET] = *byte;
		}
	}

	pub fn run(&mut self) {
		while self.window.is_open() {
			self.step();
		}
	}

	pub fn step(&mut self) {
		let fetched = self.fetch();
		self.pc += 2;

		let decoded = decode::decode(fetched);
		self.exec(decoded);

		//self.draw();
	}

	//================================================================================
	// Debug
	//================================================================================

	pub fn dump_next_instruction(&self) {
		println!("Next instruction: {:04x}", self.fetch());
	}

	pub fn dump_registers(&self) {
		println!("Register dump:");
		println!("PC: {:04x}", self.pc);
		println!("I: {:04x}", self.i);
		println!("V:");
		hex::dump(&self.v.iter().cloned().collect());
	}

	pub fn dump_memory(&self) {
		println!("Memory dump:");
		hex::dump(&self.memory);
	}

	pub fn dump_stack(&self) {
		if self.stack.len() == 0 {
			println!("Call stack is empty");
		} else {
			println!("Call stack:");
			for addr in self.stack.iter().rev() {
				println!("{:04x}", addr);
			}
		}
	}

	//================================================================================
	// CPU
	//================================================================================

	fn fetch(&self) -> Instruction {
		let i = usize::from(self.pc);
		let a = u16::from(self.memory[i]);
		let b = u16::from(if i + 1 == MEMORY_SIZE { 0 } else { self.memory[i + 1] });
		((a << 8) | b) as Instruction
	}

	fn exec(&mut self, decoded: Decoded) {
		//println!("{:?}", decoded);
		match decoded {
			Decoded::Add(x, nn)              => self.exec_add(x, nn),
			Decoded::AddXY(x, y)             => self.exec_add_xy(x, y),
			Decoded::And(x, y)               => self.exec_and(x, y),
			Decoded::Call(nnn)               => self.exec_call(nnn),
			Decoded::ClearScreen             => self.exec_cls(),
			Decoded::Decimal(x)              => self.exec_decimal(x),
			Decoded::Draw(x, y, n)           => self.exec_draw(x, y, n),
			Decoded::GetKey(x)               => self.exec_get_key(x),
			Decoded::Jump(nnn)               => self.exec_jump(nnn),
			Decoded::Load(x)                 => self.exec_load(x),
			Decoded::Move(x, nn)             => self.exec_mov(x, nn),
			Decoded::MoveIndex(nnn)          => self.exec_movi(nnn),
			Decoded::MoveXY(x, y)            => self.exec_mov_xy(x, y),
			Decoded::Or(x, y)                => self.exec_or(x, y),
			Decoded::Return                  => self.exec_return(),
			Decoded::ShiftLeft(x, y)         => self.exec_shift_left(x, y),
			Decoded::ShiftRight(x, y)        => self.exec_shift_right(x, y),
			Decoded::SkipEqual(x, nn)        => self.exec_skip_eq(x, nn),
			Decoded::SkipEqualXY(x, y)       => self.exec_skip_eq_xy(x, y),
			Decoded::SkipNotEqual(x, nn)     => self.exec_skip_ne(x, nn),
			Decoded::SkipNotEqualXY(x, y)    => self.exec_skip_ne_xy(x, y),
			Decoded::Store(x)                => self.exec_store(x),
			Decoded::SubXY(x, y)             => self.exec_sub_xy(x, y),
			Decoded::SubYX(x, y)             => self.exec_sub_yx(x, y),
			Decoded::Xor(x, y)               => self.exec_xor(x, y),

			Decoded::Illegal(i)              => self.handle_illegal_instruction(i),
		}
	}

	//================================================================================
	// Execution
	//================================================================================

	fn handle_illegal_instruction(&mut self, i: Instruction) {
		println!("Illegal instruction: {i:04x}");
		while self.window.is_open() {
			self.draw();
		}
	}

	fn exec_cls(&mut self) {
		for p in self.display.iter_mut() {
			*p = false;
		}
	}

	fn exec_movi(&mut self, nnn: Address) {
		self.i = nnn;
	}

	fn exec_mov(&mut self, x: Register, nn: Byte) {
		self.v[x] = nn;
	}

	fn exec_mov_xy(&mut self, x: Register, y: Register) {
		self.v[x] = self.v[y];
	}

	fn exec_or(&mut self, x: Register, y: Register) {
		self.v[x] |= self.v[y];
	}

	fn exec_and(&mut self, x: Register, y: Register) {
		self.v[x] &= self.v[y];
	}

	fn exec_xor(&mut self, x: Register, y: Register) {
		self.v[x] ^= self.v[y];
	}

	fn exec_add_xy(&mut self, x: Register, y: Register) {
		self.exec_add(x, self.v[y]);
	}

	fn exec_sub_xy(&mut self, x: Register, y: Register) {
		let w = Wrapping(self.v[x]) - Wrapping(self.v[y]);
		self.v[x] = w.0;
	}

	fn exec_sub_yx(&mut self, x: Register, y: Register) {
		let w = Wrapping(self.v[y]) - Wrapping(self.v[x]);
		self.v[x] = w.0;
	}

	fn exec_shift_left(&mut self, x: Register, y: Register) {
		self.v[x] = self.v[y]; // TODO: configurable.
		self.v[0xF] = (self.v[x] & 0x80) >> 7;
		self.v[x] <<= 1;
	}

	fn exec_shift_right(&mut self, x: Register, y: Register) {
		self.v[x] = self.v[y]; // TODO: configurable.
		self.v[0xF] = self.v[x] & 0x1;
		self.v[x] >>= 1;
	}

	fn exec_store(&mut self, x: Register) {
		for i in 0 ..= x {
			self.memory[self.i as usize + i] = self.v[i];
		}
	}

	fn exec_load(&mut self, x: Register) {
		for i in 0 ..= x {
			self.v[i] = self.memory[self.i as usize + i];
		}
	}

	fn exec_decimal(&mut self, x: Register) {
		let vx = self.v[x];
		let i = self.i as usize;

		let mut div = 100;

		for addr in i .. i + 3 {
			self.memory[addr] = (vx / div) % 10;
			div /= 10;
		}
	}

	fn exec_get_key(&mut self, x: Register) {
		loop {
			let keys: Vec<Key> = self.window.get_keys();

			if keys.len() == 0 {
				self.draw();
			} else {
				self.v[x] = key_to_byte(keys[0]);
				break;
			}
		}
	}

	fn exec_add(&mut self, x: Register, nn: Byte) {
		let w = Wrapping(self.v[x]) + Wrapping(nn);
		self.v[x] = w.0;
	}

	fn exec_jump(&mut self, nnn: Address) {
		self.pc = nnn;
	}

	fn exec_skip_eq(&mut self, x: Register, nn: Byte) {
		if self.v[x] == nn {
			self.pc += 2;
		}
	}

	fn exec_skip_ne(&mut self, x: Register, nn: Byte) {
		if self.v[x] != nn {
			self.pc += 2;
		}
	}

	fn exec_skip_eq_xy(&mut self, x: Register, y: Register) {
		if self.v[x] == self.v[y] {
			self.pc += 2;
		}
	}

	fn exec_skip_ne_xy(&mut self, x: Register, y: Register) {
		if self.v[x] != self.v[y] {
			self.pc += 2;
		}
	}

	fn exec_call(&mut self, nnn: Address) {
		self.stack.push(self.pc);
		self.pc = nnn;
	}

	fn exec_return(&mut self) {
		self.pc = match self.stack.pop() {
			Some(addr) => addr,
			None => panic!("call stack empty"),
		}
	}

	fn exec_draw(&mut self, x: Register, y: Register, n: Nibble) {
		let vx = (self.v[x] % DISPLAY_WIDTH as u8) as usize;
		let vy = (self.v[y] % DISPLAY_WIDTH as u8) as usize;

		let mut y = vy;

		self.v[0xF] = 0;

		for i in 0 .. n as usize {
			let byte = self.memory[self.i as usize + i];
			let mut x = vx;
			for shift in (0 .. 8).rev() {
				if byte >> shift & 1 == 1 {
					if self.display[y * DISPLAY_WIDTH + x] {
						self.v[0xF] = 1;
						self.clear_pixel(x, y);
					} else {
						self.set_pixel(x, y);
					}
				}
				x += 1;
			}
			y += 1;
		}

		self.draw();
	}

	//================================================================================
	// Display
	//================================================================================

	fn set_pixel(&mut self, x: usize, y: usize) {
		self.display[y * DISPLAY_WIDTH + x] = true;
	}

	fn clear_pixel(&mut self, x: usize, y: usize) {
		self.display[y * DISPLAY_WIDTH + x] = false;
	}

	fn draw(&mut self) {
		for y in 0 .. DISPLAY_HEIGHT {
			for x in 0 .. DISPLAY_WIDTH {
				let c = if self.display[y * DISPLAY_WIDTH + x] {
					0xFFFFFF
				} else {
					0
				};
				self.draw_square(x, y, c);
			}
		}

		self.window
			.update_with_buffer(&self.frame_buffer, WINDOW_WIDTH, WINDOW_HEIGHT)
			.unwrap();
	}

	fn draw_square(&mut self, x: usize, y: usize, c: u32) {
		for py in SQUARE_SIZE * y .. SQUARE_SIZE * (y + 1) {
			for px in SQUARE_SIZE * x .. SQUARE_SIZE * (x + 1) {
				self.frame_buffer[py * WINDOW_WIDTH + px] = c;
			}
		}
	}
}

fn key_to_byte(key: Key) -> Byte {
	match key {
		Key::Key0 => 0x0,
		Key::Key1 => 0x1,
		Key::Key2 => 0x2,
		Key::Key3 => 0x3,
		Key::Key4 => 0x4,
		Key::Key5 => 0x5,
		Key::Key6 => 0x6,
		Key::Key7 => 0x7,
		Key::Key8 => 0x8,
		Key::Key9 => 0x9,
		Key::NumPad0 => 0x0,
		Key::NumPad1 => 0x1,
		Key::NumPad2 => 0x2,
		Key::NumPad3 => 0x3,
		Key::NumPad4 => 0x4,
		Key::NumPad5 => 0x5,
		Key::NumPad6 => 0x6,
		Key::NumPad7 => 0x7,
		Key::NumPad8 => 0x8,
		Key::NumPad9 => 0x9,
		Key::A => 0xA,
		Key::B => 0xB,
		Key::C => 0xC,
		Key::D => 0xD,
		Key::E => 0xE,
		Key::F => 0xF,
		_ => panic!("Unhandled key press: {:?}", key),
	}
}
