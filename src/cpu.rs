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

    fn op_clear_screen(&mut self) { todo!() }
    fn op_return(&mut self) { todo!() }
    fn op_jump(&mut self, nnn: u16) { todo!() }
    fn op_call(&mut self, nnn: u16) { todo!() }
    fn op_skip_eq_byte(&mut self, x: usize, kk: u8) { todo!() }
    fn op_skip_neq_byte(&mut self, x: usize, kk: u8) { todo!() }
    fn op_skip_eq_reg(&mut self, x: usize, y: usize) { todo!() }
    fn op_set(&mut self, x: usize, kk: u8) { todo!() }
    fn op_add(&mut self, x: usize, kk: u8) { todo!() }
    fn op_mov(&mut self, x: usize, y: usize) { todo!() }
    fn op_or(&mut self, x: usize, y: usize) { todo!() }
    fn op_and(&mut self, x: usize, y: usize) { todo!() }
    fn op_xor(&mut self, x: usize, y: usize) { todo!() }
    fn op_add_reg(&mut self, x: usize, y: usize) { todo!() }
    fn op_sub(&mut self, x: usize, y: usize) { todo!() }
    fn op_shr(&mut self, x: usize) { todo!() }
    fn op_subn(&mut self, x: usize, y: usize) { todo!() }
    fn op_shl(&mut self, x: usize) { todo!() }
    fn op_skip_neq_reg(&mut self, x: usize, y: usize) { todo!() }
    fn op_set_i(&mut self, nnn: u16) { todo!() }
    fn op_jump_v0(&mut self, nnn: u16) { todo!() }
    fn op_rand(&mut self, x: usize, kk: u8) { todo!() }
    fn op_draw(&mut self, x: usize, y: usize, n: u8) { todo!() }
    fn op_skip_key(&mut self, x: usize) { todo!() }
    fn op_skip_nkey(&mut self, x: usize) { todo!() }
    fn op_get_delay(&mut self, x: usize) { todo!() }
    fn op_wait_key(&mut self, x: usize) { todo!() }
    fn op_set_delay(&mut self, x: usize) { todo!() }
    fn op_set_sound(&mut self, x: usize) { todo!() }
    fn op_add_i(&mut self, x: usize) { todo!() }
    fn op_font(&mut self, x: usize) { todo!() }
    fn op_bcd(&mut self, x: usize) { todo!() }
    fn op_store(&mut self, x: usize) { todo!() }
    fn op_load(&mut self, x: usize) { todo!() }
}
