use crate::types::*;

#[derive(Debug)]
pub enum Decoded {
    Add(Register, Byte),
    AddIndex(Register),
    AddXY(Register, Register),
    And(Register, Register),
    Call(Address),
    ClearScreen,
    Decimal(Register),
    DelayTimerGet(Register),
    DelayTimerSet(Register),
    Draw(Register, Register, Nibble),
    FontChar(Register),
    GetKey(Register),
    Jump(Address),
    Load(Register),
    Move(Register, Byte),
    MoveIndex(Address),
    MoveXY(Register, Register),
    Or(Register, Register),
    Random(Register, Byte),
    Return,
    SetSoundTimer(Register),
    ShiftLeft(Register, Register),
    ShiftRight(Register, Register),
    SkipEqual(Register, Byte),
    SkipEqualXY(Register, Register),
    SkipKey(Register),
    SkipNotEqual(Register, Byte),
    SkipNotEqualXY(Register, Register),
    SkipNotKey(Register),
    Store(Register),
    SubXY(Register, Register),
    SubYX(Register, Register),
    Xor(Register, Register),

    Illegal(Instruction),
}

pub fn decode(i: Instruction) -> Decoded {
    match a(i) {
        0x0 => a0(i),
        0x1 => Decoded::Jump(nnn(i)),
        0x2 => Decoded::Call(nnn(i)),
        0x3 => Decoded::SkipEqual(x(i), nn(i)),
        0x4 => Decoded::SkipNotEqual(x(i), nn(i)),
        0x5 => Decoded::SkipEqualXY(x(i), y(i)),
        0x6 => Decoded::Move(x(i), nn(i)),
        0x7 => Decoded::Add(x(i), nn(i)),
        0x8 => a8(i),
        0x9 => Decoded::SkipNotEqualXY(x(i), y(i)),
        0xA => Decoded::MoveIndex(nnn(i)),
        0xC => Decoded::Random(x(i), nn(i)),
        0xD => Decoded::Draw(x(i), y(i), n(i)),
        0xE => ae(i),
        0xF => af(i),
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

fn a0(i: Instruction) -> Decoded {
    match i {
        0x00E0 => Decoded::ClearScreen,
        0x00EE => Decoded::Return,
        _ => Decoded::Illegal(i),
    }
}

fn a8(i: Instruction) -> Decoded {
    match n(i) {
        0x0 => Decoded::MoveXY(x(i), y(i)),
        0x1 => Decoded::Or(x(i), y(i)),
        0x2 => Decoded::And(x(i), y(i)),
        0x3 => Decoded::Xor(x(i), y(i)),
        0x4 => Decoded::AddXY(x(i), y(i)),
        0x5 => Decoded::SubXY(x(i), y(i)),
        0x6 => Decoded::ShiftRight(x(i), y(i)),
        0x7 => Decoded::SubYX(x(i), y(i)),
        0xE => Decoded::ShiftLeft(x(i), y(i)),
        _ => Decoded::Illegal(i),
    }
}

fn ae(i: Instruction) -> Decoded {
    match nn(i) {
        0x9E => Decoded::SkipKey(x(i)),
        0xA1 => Decoded::SkipNotKey(x(i)),
        _ => Decoded::Illegal(i),
    }
}

fn af(i: Instruction) -> Decoded {
    match nn(i) {
        0x07 => Decoded::DelayTimerGet(x(i)),
        0x0A => Decoded::GetKey(x(i)),
        0x15 => Decoded::DelayTimerSet(x(i)),
        0x18 => Decoded::SetSoundTimer(x(i)),
        0x1E => Decoded::AddIndex(x(i)),
        0x29 => Decoded::FontChar(x(i)),
        0x33 => Decoded::Decimal(x(i)),
        0x55 => Decoded::Store(x(i)),
        0x65 => Decoded::Load(x(i)),
        _ => Decoded::Illegal(i),
    }
}
