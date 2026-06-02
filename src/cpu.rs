use rand::Rng;

pub struct Cpu {
    pub memory: [u8; 4096],
    pub v: [u8; 16], // V0-VF registers
    pub i: u16, // index register
    pub pc: u16, // program counter
    pub stack: [u16; 16],
    pub sp: u8, // stack pointer
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub display: [bool; 64 * 32],
    pub keypad: [bool; 16],
    pub draw_flag: bool,
}

const FONTS: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

impl Cpu {
    pub fn new() -> Self {
	let mut cpu = Cpu {
	    memory: [0; 4096],
	    v: [0; 16],
	    i: 0,
	    pc: 0x200, // program entry point
	    stack: [0; 16],
	    sp: 0,
	    delay_timer: 0,
	    sound_timer: 0,
	    display: [false; 64 * 32],
	    keypad: [false; 16],
	    draw_flag: false,
	};
	cpu.load_fonts();
	cpu
    }

    fn load_fonts(&mut self) {
	self.memory[0..80].copy_from_slice(&FONTS);
    }

    pub fn load_rom(&mut self, data: &[u8]) {
	self.memory[0x200..0x200 + data.len()].copy_from_slice(data);
    }

    pub fn fetch(&mut self) -> u16 {
	let hi = self.memory[self.pc as usize] as u16;
	let lo = self.memory[self.pc as usize + 1] as u16;
	self.pc += 2;
	(hi << 8) | lo
    }

    pub fn execute(&mut self, opcode: u16) {
	// break opcode into nibbles
	// first nibble gives instruction type
	// remaining give arguments for instruction
	let nnn = opcode & 0x0FFF;
	let kk = (opcode & 0x00FF) as u8;
	let n = (opcode & 0x000F) as u8;
	let x = ((opcode & 0x0F00) >> 8) as usize; // register index
	let y = ((opcode & 0x00F0) >> 4) as usize; // register index

	match (opcode & 0xF000) >> 12 {
	    0x0 => match opcode {
		0x00E0 => self.op_clear_screen(),
		0x00EE => self.op_return(),
		_ => {}
	    },
	    0x1 => self.op_jump(nnn),
	    0x2 => self.op_call(nnn),
	    0x3 => self.op_skip_eq_byte(x, kk),
	    0x4 => self.op_skip_neq_byte(x, kk),
	    0x5 => self.op_skip_eq_reg(x, y),
	    0x6 => self.op_set(x, kk),
	    0x7 => self.op_set(x, kk),
	    0x8 => match n {
		0x0 => self.op_mov(x, y),
		0x1 => self.op_or(x, y),
		0x2 => self.op_and(x, y),
		0x3 => self.op_xor(x, y),
		0x4 => self.op_add_reg(x, y),
		0x5 => self.op_sub(x, y),
		0x6 => self.op_shr(x),
		0x7 => self.op_subn(x, y),
		0xE => self.op_shl(x),
		_ => {}
	    },
	    0x9 => self.op_skip_neq_reg(x, y),
	    0xA => self.op_set_i(nnn),
	    0xB => self.op_jump_v0(nnn),
	    0xC => self.op_rand(x, kk),
	    0xD => self.op_draw(x, y, n),
	    0xE => match kk {
		0x9E => self.op_skip_key(x),
		0xA1 => self.op_skip_nkey(x),
		_ => {}
	    },
	    0xF => match kk {
		0x07 => self.op_get_delay(x),
		0x0A => self.op_wait_key(x),
		0x15 => self.op_set_delay(x),
		0x18 => self.op_set_sound(x),
		0x1E => self.op_add_i(x),
		0x29 => self.op_font(x),
		0x33 => self.op_bcd(x),
		0x55 => self.op_store(x),
		0x65 => self.op_load(x),
		_ => {}
	    },
	    _ => {}
	}
    }

    fn op_clear_screen(&mut self) {
	self.display.fill(false);
	self.draw_flag = true;
    }
    
    fn op_return(&mut self) {
	self.sp -= 1;
	self.pc = self.stack[self.sp as usize];
    }
    
    fn op_jump(&mut self, nnn: u16) {
	self.pc = nnn;
    }
    
    fn op_call(&mut self, nnn: u16) {
	self.stack[self.sp as usize] = self.pc;
	self.sp += 1;
	self.pc = nnn;
    }
    
    fn op_skip_eq_byte(&mut self, x: usize, kk: u8) {
	if self.v[x] == kk {
	    self.pc += 2;
	}
    }
    
    fn op_skip_neq_byte(&mut self, x: usize, kk: u8) {
	if self.v[x] != kk {
	    self.pc += 2;
	}
    }
    
    fn op_skip_eq_reg(&mut self, x: usize, y: usize) {
	if self.v[x] == self.v[y] {
	    self.pc += 2;
	}
    }

    fn op_skip_neq_reg(&mut self, x: usize, y: usize) {
	if self.v[x] != self.v[y] {
	    self.pc += 2;
	}
    }
    
    fn op_set(&mut self, x: usize, kk: u8) {
	self.v[x] = kk;
    }
    
    fn op_add(&mut self, x: usize, kk: u8) {
	self.v[x] = self.v[x].wrapping_add(kk);
    }
    
    fn op_mov(&mut self, x: usize, y: usize) {
	self.v[x] = self.v[y];
    }
    
    fn op_or(&mut self, x: usize, y: usize) {
	self.v[x] = self.v[x] | self.v[y];
    }
    
    fn op_and(&mut self, x: usize, y: usize) {
	self.v[x] = self.v[x] & self.v[y];
    }
    
    fn op_xor(&mut self, x: usize, y: usize) {
	self.v[x] = self.v[x] ^ self.v[y];
    }
    
    fn op_add_reg(&mut self, x: usize, y: usize) {
	let (result, overflowed) = self.v[x].overflowing_add(self.v[y]);
	self.v[x] = result;
	self.v[0xF] = if overflowed { 1 } else { 0 };
    }
    
    fn op_sub(&mut self, x: usize, y: usize) {
	let (result, overflowed) = self.v[x].overflowing_sub(self.v[y]);
	self.v[x] = result;
	self.v[0xF] = if overflowed { 0 } else { 1 };
    }

    fn op_subn(&mut self, x: usize, y: usize) {
	let (result, overflowed) = self.v[y].overflowing_sub(self.v[x]);
	self.v[x] = result;
	self.v[0xF] = if overflowed { 0 } else { 1 };
    }
    
    fn op_shr(&mut self, x: usize) {
	let vf = self.v[x] & 0x1;
	self.v[x] = self.v[x] >> 1;
	self.v[0xF] = vf;
    }
    
    fn op_shl(&mut self, x: usize) {
	let vf = (self.v[x] & 0x80) >> 7;
	self.v[x] = self.v[x] << 1;
	self.v[0xF] = vf;
    }
    
    fn op_set_i(&mut self, nnn: u16) {
	self.i = nnn;
    }
    
    fn op_jump_v0(&mut self, nnn: u16) {
	self.pc = nnn + self.v[0] as u16;
    }
    
    fn op_rand(&mut self, x: usize, kk: u8) {
	let rand = rand::random::<u8>();
	self.v[x] = rand & kk;
    }
    
    fn op_draw(&mut self, x: usize, y: usize, n: u8) {
	self.draw_flag = true;
	self.v[0xF] = 0;

	for row in 0..n {
	    let sprite = self.memory[(self.i + row as u16) as usize];

	    for col in 0..8 {
		let bit = (sprite >> (7 - col)) & 0x1;
		let sx = (self.v[x] + col) % 64;
		let sy = (self.v[y] + row) % 32;
		let di = sy * 64 + sx;
		if bit == 1 && self.display[di as usize] {
		    self.v[0xF] = 1;
		}

		self.display[di as usize] = self.display[di as usize] ^ (bit == 1);
	    }
	}
    }
    
    fn op_skip_key(&mut self, x: usize) {
	if self.keypad[self.v[x] as usize] {
	    self.pc += 2;
	}
    }
    
    fn op_skip_nkey(&mut self, x: usize) {
	if !self.keypad[self.v[x] as usize] {
	    self.pc += 2;
	}
    }
    
    fn op_get_delay(&mut self, x: usize) {
	self.v[x] = self.delay_timer;
    }
    
    fn op_wait_key(&mut self, x: usize) {
	for i in 0..16 {
	    if self.keypad[i] {
		self.v[x] = i as u8;
		return;
	    }
	}

	self.pc -= 2;
    }
    
    fn op_set_delay(&mut self, x: usize) {
	self.delay_timer = self.v[x];
    }
    
    fn op_set_sound(&mut self, x: usize) {
	self.sound_timer = self.v[x];
    }
    
    fn op_add_i(&mut self, x: usize) {
	self.i += self.v[x] as u16;
    }
    
    fn op_font(&mut self, x: usize) {
	self.i = self.v[x] as u16 * 5;
    }
    
    fn op_bcd(&mut self, x: usize) {
	self.memory[self.i as usize] = self.v[x] / 100;
	self.memory[(self.i + 1) as usize] = (self.v[x] % 100) / 10;
	self.memory[(self.i + 2) as usize] = self.v[x] % 10;
    }
    
    fn op_store(&mut self, x: usize) {
	for i in 0..=x {
	    self.memory[i + self.i as usize] = self.v[i];
	}
    }
    
    fn op_load(&mut self, x: usize) {
	for i in 0..=x {
	    self.v[i] = self.memory[i + self.i as usize];
	}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clear_screen() {
        let mut cpu = Cpu::new();
        cpu.display[0] = true;
        cpu.display[100] = true;
        cpu.op_clear_screen();
        assert!(cpu.display.iter().all(|&p| p == false));
        assert!(cpu.draw_flag);
    }

    #[test]
    fn test_jump() {
        let mut cpu = Cpu::new();
        cpu.op_jump(0x300);
        assert_eq!(cpu.pc, 0x300);
    }

    #[test]
    fn test_set() {
        let mut cpu = Cpu::new();
        cpu.op_set(3, 0xAB);
        assert_eq!(cpu.v[3], 0xAB);
    }

    #[test]
    fn test_call_and_return() {
	let mut cpu = Cpu::new();
	cpu.op_call(0x400);
	assert_eq!(cpu.pc, 0x400);
	assert_eq!(cpu.sp, 1);
	assert_eq!(cpu.stack[0], 0x200); // original pc
	cpu.op_return();
	assert_eq!(cpu.pc, 0x200);
	assert_eq!(cpu.sp, 0);
    }

    #[test]
    fn test_skip_eq_byte() {
	let mut cpu = Cpu::new();
	cpu.v[0] = 0xAB;
	cpu.op_skip_eq_byte(0, 0xAB);
	assert_eq!(cpu.pc, 0x202); // skipped
	cpu.op_skip_eq_byte(0, 0x00);
	assert_eq!(cpu.pc, 0x202); // did not skip
    }

    #[test]
    fn test_skip_neq_byte() {
	let mut cpu = Cpu::new();
	cpu.v[0] = 0xAB;
	cpu.op_skip_neq_byte(0, 0x00);
	assert_eq!(cpu.pc, 0x202); // skipped
	cpu.op_skip_neq_byte(0, 0xAB);
	assert_eq!(cpu.pc, 0x202); // did not skip
    }

    #[test]
    fn test_skip_eq_reg() {
	let mut cpu = Cpu::new();
	cpu.v[0] = 0xAB;
	cpu.v[1] = 0xAB;
	cpu.op_skip_eq_reg(0, 1);
	assert_eq!(cpu.pc, 0x202); // skipped
    }

    #[test]
    fn test_skip_neq_reg() {
	let mut cpu = Cpu::new();
	cpu.v[0] = 0xAB;
	cpu.v[1] = 0x00;
	cpu.op_skip_neq_reg(0, 1);
	assert_eq!(cpu.pc, 0x202); // skipped
    }

    #[test]
    fn test_add() {
	let mut cpu = Cpu::new();
	cpu.v[0] = 0xFF;
	cpu.op_add(0, 0x01);
	assert_eq!(cpu.v[0], 0x00); // wraps
    }

    #[test]
    fn test_mov() {
	let mut cpu = Cpu::new();
	cpu.v[1] = 0xAB;
	cpu.op_mov(0, 1);
	assert_eq!(cpu.v[0], 0xAB);
    }

    #[test]
    fn test_or() {
	let mut cpu = Cpu::new();
	cpu.v[0] = 0b1100;
	cpu.v[1] = 0b1010;
	cpu.op_or(0, 1);
	assert_eq!(cpu.v[0], 0b1110);
    }

    #[test]
    fn test_and() {
	let mut cpu = Cpu::new();
	cpu.v[0] = 0b1100;
	cpu.v[1] = 0b1010;
	cpu.op_and(0, 1);
	assert_eq!(cpu.v[0], 0b1000);
    }

    #[test]
    fn test_xor() {
	let mut cpu = Cpu::new();
	cpu.v[0] = 0b1100;
	cpu.v[1] = 0b1010;
	cpu.op_xor(0, 1);
	assert_eq!(cpu.v[0], 0b0110);
    }

    #[test]
    fn test_add_reg() {
	let mut cpu = Cpu::new();
	cpu.v[0] = 0xFF;
	cpu.v[1] = 0x01;
	cpu.op_add_reg(0, 1);
	assert_eq!(cpu.v[0], 0x00);
	assert_eq!(cpu.v[0xF], 1); // carry set
	cpu.v[0] = 0x01;
	cpu.v[1] = 0x01;
	cpu.op_add_reg(0, 1);
	assert_eq!(cpu.v[0], 0x02);
	assert_eq!(cpu.v[0xF], 0); // no carry
    }

    #[test]
    fn test_sub() {
	let mut cpu = Cpu::new();
	cpu.v[0] = 0x05;
	cpu.v[1] = 0x03;
	cpu.op_sub(0, 1);
	assert_eq!(cpu.v[0], 0x02);
	assert_eq!(cpu.v[0xF], 1); // no borrow
	cpu.v[0] = 0x03;
	cpu.v[1] = 0x05;
	cpu.op_sub(0, 1);
	assert_eq!(cpu.v[0xF], 0); // borrow
    }

    #[test]
    fn test_subn() {
	let mut cpu = Cpu::new();
	cpu.v[0] = 0x03;
	cpu.v[1] = 0x05;
	cpu.op_subn(0, 1);
	assert_eq!(cpu.v[0], 0x02);
	assert_eq!(cpu.v[0xF], 1); // no borrow
    }

    #[test]
    fn test_shr() {
	let mut cpu = Cpu::new();
	cpu.v[0] = 0b0000_0101;
	cpu.op_shr(0);
	assert_eq!(cpu.v[0], 0b0000_0010);
	assert_eq!(cpu.v[0xF], 1); // LSB was 1

	cpu.v[0] = 0b0000_0100;
	cpu.op_shr(0);
	assert_eq!(cpu.v[0], 0b0000_0010);
	assert_eq!(cpu.v[0xF], 0); // LSB was 0
    }

    #[test]
    fn test_shl() {
	let mut cpu = Cpu::new();
	cpu.v[0] = 0b1000_0001;
	cpu.op_shl(0);
	assert_eq!(cpu.v[0], 0b0000_0010);
	assert_eq!(cpu.v[0xF], 1); // MSB was 1

	cpu.v[0] = 0b0000_0001;
	cpu.op_shl(0);
	assert_eq!(cpu.v[0], 0b0000_0010);
	assert_eq!(cpu.v[0xF], 0); // MSB was 0
    }

    #[test]
    fn test_set_i() {
	let mut cpu = Cpu::new();
	cpu.op_set_i(0x300);
	assert_eq!(cpu.i, 0x300);
    }

    #[test]
    fn test_jump_v0() {
	let mut cpu = Cpu::new();
	cpu.v[0] = 0x10;
	cpu.op_jump_v0(0x300);
	assert_eq!(cpu.pc, 0x310);
    }

    #[test]
    fn test_delay_timer() {
	let mut cpu = Cpu::new();
	cpu.v[0] = 0x42;
	cpu.op_set_delay(0);
	assert_eq!(cpu.delay_timer, 0x42);
	cpu.op_get_delay(1);
	assert_eq!(cpu.v[1], 0x42);
    }

    #[test]
    fn test_set_sound() {
	let mut cpu = Cpu::new();
	cpu.v[0] = 0x42;
	cpu.op_set_sound(0);
	assert_eq!(cpu.sound_timer, 0x42);
    }

    #[test]
    fn test_add_i() {
	let mut cpu = Cpu::new();
	cpu.i = 0x100;
	cpu.v[0] = 0x10;
	cpu.op_add_i(0);
	assert_eq!(cpu.i, 0x110);
    }

    #[test]
    fn test_font() {
	let mut cpu = Cpu::new();
	cpu.v[0] = 0xA;
	cpu.op_font(0);
	assert_eq!(cpu.i, 0xA * 5);
    }

    #[test]
    fn test_bcd() {
	let mut cpu = Cpu::new();
	cpu.i = 0x300;
	cpu.v[0] = 123;
	cpu.op_bcd(0);
	assert_eq!(cpu.memory[0x300], 1);
	assert_eq!(cpu.memory[0x301], 2);
	assert_eq!(cpu.memory[0x302], 3);
    }

    #[test]
    fn test_store_and_load() {
	let mut cpu = Cpu::new();
	cpu.i = 0x300;
	cpu.v[0] = 0xAA;
	cpu.v[1] = 0xBB;
	cpu.v[2] = 0xCC;
	cpu.op_store(2);
	// clear registers
	cpu.v[0] = 0;
	cpu.v[1] = 0;
	cpu.v[2] = 0;
	cpu.op_load(2);
	assert_eq!(cpu.v[0], 0xAA);
	assert_eq!(cpu.v[1], 0xBB);
	assert_eq!(cpu.v[2], 0xCC);
    }

    #[test]
    fn test_draw() {
	let mut cpu = Cpu::new();
	cpu.i = 0x300;
	// a simple 1-row sprite: 0b11110000
	cpu.memory[0x300] = 0b1111_0000;
	cpu.v[0] = 0; // x
	cpu.v[1] = 0; // y
	cpu.op_draw(0, 1, 1);
	assert!(cpu.display[0]);
	assert!(cpu.display[1]);
	assert!(cpu.display[2]);
	assert!(cpu.display[3]);
	assert!(!cpu.display[4]);
	assert_eq!(cpu.v[0xF], 0); // no collision
	assert!(cpu.draw_flag);

	// draw again in same spot — should erase and set collision
	cpu.op_draw(0, 1, 1);
	assert!(!cpu.display[0]);
	assert_eq!(cpu.v[0xF], 1); // collision
    }
}
