use crate::types::*;

#[derive(Debug)]
pub enum Decoded {
	Add(Register, Byte),
	Call(Address),
	ClearScreen,
	Draw(Register, Register, Nibble),
	Jump(Address),
	Move(Register, Byte),
	MoveIndex(Address),
	MoveXY(Register, Register),
	Return,
	SkipEqual(Register, Byte),
	SkipEqualXY(Register, Register, Nibble),
	SkipNotEqual(Register, Byte),
	SkipNotEqualXY(Register, Register, Nibble),

	Illegal(Instruction),
}

pub fn decode(i: Instruction) -> Decoded {
	if i == 0x00E0 {
		return Decoded::ClearScreen;
	}

	if i == 0x00EE {
		return Decoded::Return;
	}

	match a(i) {
		0x1 => Decoded::Jump(nnn(i)),
		0x2 => Decoded::Call(nnn(i)),
		0x3 => Decoded::SkipEqual(x(i), nn(i)),
		0x4 => Decoded::SkipNotEqual(x(i), nn(i)),
		0x5 => Decoded::SkipEqualXY(x(i), y(i), n(i)),
		0x6 => Decoded::Move(x(i), nn(i)),
		0x7 => Decoded::Add(x(i), nn(i)),
		0x8 => a8(i),
		0x9 => Decoded::SkipNotEqualXY(x(i), y(i), n(i)),
		0xA => Decoded::MoveIndex(nnn(i)),
		0xD => Decoded::Draw(x(i), y(i), n(i)),
		_ => Decoded::Illegal(i),
	}
}

fn a(i: Instruction) -> Nibble {
	((i >> 12) & 0xF) as Nibble
}

fn x(i: Instruction) -> Register {
	((i >> 8) & 0xF) as Register
}

fn y(i: Instruction) -> Register {
	((i >> 4) & 0xF) as Register
}

fn nnn(i: Instruction) -> Address {
	(i & 0xFFF) as Address
}

fn nn(i: Instruction) -> Byte {
	(i & 0xFF) as Byte
}

fn n(i: Instruction) -> Nibble {
	(i & 0xF) as Nibble
}

fn a8(i: Instruction) -> Decoded {
	match n(i) {
		0x0 => Decoded::MoveXY(x(i), y(i)),
		_ => Decoded::Illegal(i),
	}
}
