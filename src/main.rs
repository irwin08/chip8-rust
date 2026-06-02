use minifb::{Key, Window, WindowOptions};

mod cpu;

const SCALE : usize = 10;

fn main() {
    let mut window = Window::new(
	"CHIP-8",
	64 * SCALE,
	32 * SCALE,
	WindowOptions::default(),
    ).unwrap();

    let mut buffer : Vec<u32> = vec![0; 64 * SCALE * 32 * SCALE];

    let args: Vec<String> = std::env::args().collect();
    let rom = std::fs::read(&args[1]).expect("could not read ROM");
    let mut cpu = cpu::Cpu::new();
    cpu.load_rom(&rom);
    
    while window.is_open() && !window.is_key_down(Key::Escape) {
	handle_input(&mut cpu, &window);
	
	for _ in 0..10 {
	    let opcode = cpu.fetch();
	    cpu.execute(opcode);
	}

	if cpu.delay_timer > 0 {
	    cpu.delay_timer -= 1;
	}
	if cpu.sound_timer > 0 {
	    cpu.sound_timer -= 1;
	}

	if cpu.draw_flag {
	    for x in 0..64 {
		for y in 0..32 {
		    let color = if cpu.display[y * 64 + x] { 0x00FF_FFFF } else { 0x0000_0000 };
		    // handle SCALE
		    for sy in 0..SCALE {
			for sx in 0..SCALE {
			    buffer[(y * SCALE + sy) * (64 * SCALE) + (x * SCALE + sx)] = color;
			}
		    }
		}
	    }

	    cpu.draw_flag = false;
	}
	
	window.update_with_buffer(&buffer, 64 * SCALE, 32 * SCALE).unwrap();
    }
}

fn handle_input(cpu: &mut cpu::Cpu, window: &minifb::Window) {
    cpu.keypad[0x0] = window.is_key_down(Key::X);
    cpu.keypad[0x1] = window.is_key_down(Key::Key1);
    cpu.keypad[0x2] = window.is_key_down(Key::Key2);
    cpu.keypad[0x3] = window.is_key_down(Key::Key3);
    cpu.keypad[0x4] = window.is_key_down(Key::Q);
    cpu.keypad[0x5] = window.is_key_down(Key::W);
    cpu.keypad[0x6] = window.is_key_down(Key::E);
    cpu.keypad[0x7] = window.is_key_down(Key::A);
    cpu.keypad[0x8] = window.is_key_down(Key::S);
    cpu.keypad[0x9] = window.is_key_down(Key::D);
    cpu.keypad[0xA] = window.is_key_down(Key::Z);
    cpu.keypad[0xB] = window.is_key_down(Key::C);
    cpu.keypad[0xC] = window.is_key_down(Key::Key4);
    cpu.keypad[0xD] = window.is_key_down(Key::R);
    cpu.keypad[0xE] = window.is_key_down(Key::F);
    cpu.keypad[0xF] = window.is_key_down(Key::V);
}
