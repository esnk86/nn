use minifb::{Key, Window, WindowOptions};

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

		self.draw();
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
			Decoded::Add(x, nn)          => self.exec_add(x, nn),
			Decoded::Call(nnn)           => self.exec_call(nnn),
			Decoded::ClearScreen         => self.exec_cls(),
			Decoded::Draw(x, y, n)       => self.exec_draw(x, y, n),
			Decoded::Jump(nnn)           => self.exec_jump(nnn),
			Decoded::Move(x, nn)         => self.exec_mov(x, nn),
			Decoded::MoveIndex(nnn)      => self.exec_movi(nnn),
			Decoded::Return              => self.exec_return(),
			Decoded::SkipEqual(x, nn)    => self.exec_skip_equal(x, nn),
			Decoded::SkipNotEqual(x, nn) => self.exec_skip_not_equal(x, nn),

			Decoded::Illegal(i)          => panic!("Illegal instruction: 0x{i:04x}"),
		}
	}

	//================================================================================
	// Execution
	//================================================================================

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

	fn exec_add(&mut self, x: Register, nn: Byte) {
		self.v[x] += nn;
	}

	fn exec_jump(&mut self, nnn: Address) {
		self.pc = nnn;
	}

	fn exec_skip_equal(&mut self, x: Register, nn: Byte) {
		if self.v[x] == nn {
			self.pc += 2;
		}
	}

	fn exec_skip_not_equal(&mut self, x: Register, nn: Byte) {
		if self.v[x] != nn {
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
