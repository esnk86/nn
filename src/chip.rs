#[allow(unused_imports)]
use minifb::{Key, Window, WindowOptions};
use rand::Rng;

use std::num::Wrapping;

use crate::decode::{self, Decoded};
use crate::font;
use crate::hex;
use crate::timer::Timer;
use crate::types::*;

const MEMORY_SIZE: usize = 4096;
const FONT_BYTE_COUNT: Address = 5;
const FONT_MEMORY_OFFSET: usize = 0;
const PROGRAM_MEMORY_OFFSET: usize = 512;
const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const SQUARE_SIZE: usize = 10;
const WINDOW_WIDTH: usize = SQUARE_SIZE * DISPLAY_WIDTH;
const WINDOW_HEIGHT: usize = SQUARE_SIZE * DISPLAY_HEIGHT;

pub struct Chip {
	pc: Address,
	i: Address,
	v: [Byte; 16],
	memory: Vec<Byte>,
	stack: Vec<Address>,
	delay: Timer,
	display: Vec<bool>,
	window: Window,
	frame_buffer: Vec<u32>,
}

// Public interface.
impl Chip {
	pub fn new() -> Self {
		Self {
			pc: PROGRAM_MEMORY_OFFSET as Address,
			i: 0,
			v: [0; 16],
			memory: vec![0; MEMORY_SIZE],
			stack: Vec::new(),
			delay: Timer::new(),
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

	pub fn load_font(&mut self) {
		let font = font::get();
		for (i, byte) in font.iter().enumerate() {
			self.memory[i + FONT_MEMORY_OFFSET] = *byte;
		}
	}

	pub fn load_rom(&mut self, bytes: Vec<u8>) {
		for (i, byte) in bytes.iter().enumerate() {
			self.memory[i + PROGRAM_MEMORY_OFFSET] = *byte;
		}
	}

	pub fn run(&mut self) {
		self.load_font();
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
}

// Debugging methods.
impl Chip {
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
}

// CPU emulation.
impl Chip {
	fn fetch(&self) -> Instruction {
		let i = self.pc as usize;
		let a = self.memory[i] as Instruction;
		let b = self.memory[i + 1] as Instruction;

		a << 8 | b
	}

	fn exec(&mut self, decoded: Decoded) {
		//println!("{:?}", decoded);
		match decoded {
			Decoded::Add(x, nn)              => self.exec_add(x, nn),
			Decoded::AddIndex(x)             => self.exec_add_index(x),
			Decoded::AddXY(x, y)             => self.exec_add_xy(x, y),
			Decoded::And(x, y)               => self.exec_and(x, y),
			Decoded::Call(nnn)               => self.exec_call(nnn),
			Decoded::ClearScreen             => self.exec_cls(),
			Decoded::Decimal(x)              => self.exec_decimal(x),
			Decoded::DelayTimerGet(x)        => self.exec_delay_timer_get(x),
			Decoded::DelayTimerSet(x)        => self.exec_delay_timer_set(x),
			Decoded::Draw(x, y, n)           => self.exec_draw(x, y, n),
			Decoded::FontChar(x)             => self.exec_font_char(x),
			Decoded::GetKey(x)               => self.exec_get_key(x),
			Decoded::Jump(nnn)               => self.exec_jump(nnn),
			Decoded::Load(x)                 => self.exec_load(x),
			Decoded::Move(x, nn)             => self.exec_mov(x, nn),
			Decoded::MoveIndex(nnn)          => self.exec_movi(nnn),
			Decoded::MoveXY(x, y)            => self.exec_mov_xy(x, y),
			Decoded::Or(x, y)                => self.exec_or(x, y),
			Decoded::Random(x, nn)           => self.exec_rand(x, nn),
			Decoded::Return                  => self.exec_return(),
			Decoded::SetSoundTimer(x)        => self.exec_set_sound_timer(x),
			Decoded::ShiftLeft(x, y)         => self.exec_shift_left(x, y),
			Decoded::ShiftRight(x, y)        => self.exec_shift_right(x, y),
			Decoded::SkipEqual(x, nn)        => self.exec_skip_eq(x, nn),
			Decoded::SkipEqualXY(x, y)       => self.exec_skip_eq_xy(x, y),
			Decoded::SkipKey(x)              => self.exec_skip_key(x),
			Decoded::SkipNotEqual(x, nn)     => self.exec_skip_ne(x, nn),
			Decoded::SkipNotEqualXY(x, y)    => self.exec_skip_ne_xy(x, y),
			Decoded::SkipNotKey(x)           => self.exec_skip_not_key(x),
			Decoded::Store(x)                => self.exec_store(x),
			Decoded::SubXY(x, y)             => self.exec_sub_xy(x, y),
			Decoded::SubYX(x, y)             => self.exec_sub_yx(x, y),
			Decoded::Xor(x, y)               => self.exec_xor(x, y),

			Decoded::Illegal(i)              => self.handle_illegal_instruction(i),
		}
	}

	fn handle_illegal_instruction(&mut self, i: Instruction) {
		println!("Illegal instruction: {i:04x}");
		while self.window.is_open() {
			self.draw();
		}
	}
}

// Move, load, and store instructions.
impl Chip {
	fn exec_mov(&mut self, x: Register, nn: Byte) {
		self.v[x] = nn;
	}

	fn exec_mov_xy(&mut self, x: Register, y: Register) {
		self.v[x] = self.v[y];
	}

	fn exec_movi(&mut self, nnn: Address) {
		self.i = nnn;
	}

	fn exec_load(&mut self, x: Register) {
		for i in 0 ..= x {
			self.v[i] = self.memory[self.i as usize + i];
		}
	}

	fn exec_store(&mut self, x: Register) {
		for i in 0 ..= x {
			self.memory[self.i as usize + i] = self.v[i];
		}
	}
}

// Instructions for control flow.
impl Chip {
	fn exec_jump(&mut self, nnn: Address) {
		self.pc = nnn;
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

	fn exec_skip_key(&mut self, x: Register) {
		let want = byte_to_key(self.v[x]);
		let have = self.window.get_keys();

		if let Some(key) = have.get(0) {
			if *key == want {
				self.pc += 2;
			}
		}
	}

	fn exec_skip_not_key(&mut self, x: Register) {
		let want = byte_to_key(self.v[x]);
		let have = self.window.get_keys();

		match have.get(0) {
			Some(key) => if *key != want { self.pc += 2; },
			None => self.pc += 2,
		}
	}

	fn exec_get_key(&mut self, x: Register) {
		while self.window.is_open() {
			let keys: Vec<Key> = self.window.get_keys();

			if keys.len() == 0 {
				self.draw();
			} else {
				self.v[x] = key_to_byte(keys[0]);
				break;
			}
		}
	}
}

// Instructions for logic.
impl Chip {
	fn exec_and(&mut self, x: Register, y: Register) {
		self.v[x] &= self.v[y];
	}

	fn exec_or(&mut self, x: Register, y: Register) {
		self.v[x] |= self.v[y];
	}

	fn exec_xor(&mut self, x: Register, y: Register) {
		self.v[x] ^= self.v[y];
	}

	fn exec_shift_left(&mut self, x: Register, y: Register) {
		self.v[x] = self.v[y]; // TODO: configurable.
		self.v[0xF] = self.v[x] >> 7;
		self.v[x] <<= 1;
	}

	fn exec_shift_right(&mut self, x: Register, y: Register) {
		self.v[x] = self.v[y]; // TODO: configurable.
		self.v[0xF] = self.v[x] & 0x1;
		self.v[x] >>= 1;
	}
}

// Instructions for maths.
impl Chip {
	fn exec_add(&mut self, x: Register, nn: Byte) {
		let w = Wrapping(self.v[x]) + Wrapping(nn);
		self.v[x] = w.0;
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

	fn exec_add_index(&mut self, x: Register) {
		self.i += self.v[x] as Address;
	}

	fn exec_rand(&mut self, x: Register, nn: Byte) {
		let rn = rand::thread_rng().gen_range(0 ..= 255);
		self.v[x] = rn & nn;
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
}

// Instructions for timers.
impl Chip {
	fn exec_set_sound_timer(&mut self, _x: Register) {
	}

	fn exec_delay_timer_set(&mut self, x: Register) {
		self.delay.set(self.v[x]);
	}

	fn exec_delay_timer_get(&mut self, x: Register) {
		self.v[x] = self.delay.get();
	}
}

// Instructions for the display.
impl Chip {
	fn exec_cls(&mut self) {
		for p in self.display.iter_mut() {
			*p = false;
		}
	}

	fn exec_font_char(&mut self, x: Register) {
		self.i = FONT_BYTE_COUNT * (self.v[x] as Address + FONT_MEMORY_OFFSET as Address);
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
}

// Methods for handling the display and window.
impl Chip {
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
		Key::Key1 => 0x1,
		Key::Key2 => 0x2,
		Key::Key3 => 0x3,
		Key::Key4 => 0xC,
		Key::Q => 0x4,
		Key::W => 0x5,
		Key::E => 0x6,
		Key::R => 0xD,
		Key::A => 0x7,
		Key::S => 0x8,
		Key::D => 0x9,
		Key::F => 0xE,
		Key::Z => 0xA,
		Key::X => 0x0,
		Key::C => 0xB,
		Key::V => 0xF,
		_ => panic!("Unhandled key press: {:?}", key),
	}
}

fn byte_to_key(byte: Byte) -> Key {
	match byte {
		0x1 => Key::Key1,
		0x2 => Key::Key2,
		0x3 => Key::Key3,
		0xC => Key::Key4,
		0x4 => Key::Q,
		0x5 => Key::W,
		0x6 => Key::E,
		0xD => Key::R,
		0x7 => Key::A,
		0x8 => Key::S,
		0x9 => Key::D,
		0xE => Key::F,
		0xA => Key::Z,
		0x0 => Key::X,
		0xB => Key::C,
		0xF => Key::V,
		_ => panic!("Unhandled byte-to-key: {byte}"),
	}
}
