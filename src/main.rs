use std::fs::File;
use std::io::Read;
use std::thread;
use std::time::Duration;

static SPRITE_FOR_CHARS: [i32; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0x10, // 3
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

/// Things to mention:
/// * vx means register number x.
/// * nn is a constant number (called `number_in`) supplied in the opcode.
#[derive(Debug, PartialEq, Eq)]
struct Chip8 {
    mem: [u8; 4096],
    /// Registers (V) called V0, V1, ..., V9, VA, VB, ..., VF (hex number of the register is appended).
    registers: [u8; 16],
    /// Address register (I).
    address_register: u16,
    /// Program counter (PC).
    pc: usize,
    stack: [usize; 12],
    stack_pointer: u8,

    /// The display as a bit array. Access like `display[y][x]`.
    display: [[u8; 8]; 32],
    /// Current key pressed by the user.
    current_key: u8,

    delay_timer: u8,
    sound_timer: u8,
}

impl Chip8 {
    /// Loads an opcode from memory by fetching two bytes and combing them in big-endian fashion.
    fn load_opcode(&self) -> u16 {
        // Instructions are stored in big endian, so the most significant byte is placed at the byte with the lowest
        // address.
        let upper = self.mem[self.pc] as u16;
        let lower = self.mem[self.pc + 1] as u16;
        let opcode = (upper << 8) | lower;
        opcode
    }

    fn print_display(&self) {
        for row in self.display {
            for mut cell in row {
                for i in 0..8 { // Loop through the byte
                    let pixel = cell & 0x80 != 0; // Extract each pixel
                    cell <<= 1;
                    match pixel {
                        true => print!("X"),
                        false => print!(" "),
                    }
                }
            }
            println!();
        }
    }

    fn run(&mut self) {
        for _ in 0..10000 {
            self.exec_instruction();
            self.print_display();
            self.sound_timer.saturating_sub(1);
            self.delay_timer.saturating_sub(1);
            thread::sleep(Duration::from_secs_f64(1.0 / 60.0)); // Run at 60Hz
        }
    }

    fn exec_instruction(&mut self) {
        let opcode = self.load_opcode();
        if opcode & 0xF000 == 0x0000 {
            self.call_machine_routine(opcode);
        } else if opcode & 0x00F0 == 0x00E0 {
            self.clear_display();
        } else if opcode & 0x00FF == 0x00EE {
            self.subroutine_return();
        } else if opcode & 0xF000 == 0x1000 {
            self.jump(opcode);
        } else if opcode & 0xF000 == 0x2000 {
            self.call_subroutine(opcode)
        } else if opcode & 0xF000 == 0x3000 {
            self.skip_if_vx_eq_nn(opcode);
        } else if opcode & 0xF000 == 0x4000 {
            self.skip_if_vx_ne_nn(opcode);
        } else if opcode & 0xF00F == 0x5000 {
            self.skip_if_vx_eq_vy(opcode);
        } else if opcode & 0xF000 == 0x6000 {
            self.set_vx_to_n(opcode);
        } else if opcode & 0xF000 == 0x7000 {
            self.add_n_to_vx(opcode);
        } else if opcode & 0xF00F == 0x8000 {
            self.set_vx_to_vy(opcode);
        } else if opcode & 0xF00F == 0x8001 {
            self.set_vx_to_vx_bitor_vy(opcode);
        } else if opcode & 0xF00F == 0x8002 {
            self.set_vx_to_vx_bitand_vy(opcode);
        } else if opcode & 0xF00F == 0x8003 {
            self.set_vx_to_vx_xor_vy(opcode);
        } else if opcode & 0xF00F == 0x8004 {
            self.add_vy_to_vx(opcode);
        } else if opcode & 0xF00F == 0x8005 {
            self.subtract_vy_from_vx(opcode);
        } else if opcode & 0xF00F == 0x8006 {
            self.right_shift_vx(opcode);
        } else if opcode & 0xF00F == 0x8007 {
            self.set_vx_to_vy_minus_vx(opcode);
        } else if opcode & 0xF00F == 0x800E {
            self.left_shift_vx(opcode);
        } else if opcode & 0xF00F == 0x9000 {
            self.skip_if_vx_ne_vy(opcode);
        } else if opcode & 0xF000 == 0xA000 {
            self.set_i_addr_to_n(opcode);
        } else if opcode & 0xF000 == 0xB000 {
            self.jump_to_n_plus_v0(opcode);
        } else if opcode & 0xF000 == 0xC000 {
            self.set_to_vx_rand_bitand_n(opcode);
        } else if opcode & 0xF000 == 0xD000 {
            self.draw_sprite_at_coordinates_vx_vy_with_height_n(opcode);
        } else if opcode & 0xF0FF == 0xE09E {
            self.skip_if_key_in_vk_pressed(opcode);
        } else if opcode & 0xF0FF == 0xE0A1 {
            self.skip_if_key_in_vk_not_pressed(opcode);
        }
    }

    /// Call machine routine. Opcode: `0NNN` - `SYS addr`.
    fn call_machine_routine(&mut self, opcode: u16) {
        let machine_routine_nr = opcode & 0x0FFF;
        eprintln!("Call to machine routine {} ignored", machine_routine_nr);
        self.pc += 1;
    }

    /// Clears the display, i.e. sets all bytes to zero. Opcode: `00E0` - `CLS`.
    fn clear_display(&mut self) {
        self.display = Default::default();
    }

    /// Return from subroutine. Opcode: `00EE` - `RET`.
    fn subroutine_return(&mut self) {
        self.pc = self.stack[self.stack_pointer as usize] as usize;
        self.stack_pointer -= 1;
        self.pc += 1;
    }

    /// Set the program counter to NNN. Opcode: `1NNN` - `JP addr`.
    fn jump(&mut self, opcode: u16) {
        let jump_addr = opcode & 0x0FFF;
        self.pc = jump_addr as usize;
    }

    /// Call subroutine. Opcode: `2NNN` - `CALL addr`.
    fn call_subroutine(&mut self, opcode: u16) {
        self.stack_pointer += 1;
        self.stack[self.stack_pointer as usize] = self.pc;
        let subroutine_mem_addr = opcode & 0x0FFF;
        self.pc = subroutine_mem_addr as usize;
    }

    /// Skip next instruction if vx (register) == nn (constant in). Opcode: `3XNN` - `SE vx, byte`.
    fn skip_if_vx_eq_nn(&mut self, opcode: u16) {
        let register_number = (opcode & 0x0F00) >> 8;
        let register_value = self.registers[register_number as usize];
        let number_in = opcode & 0x00FF;
        if register_value == number_in as u8 {
            self.pc += 1;
        }
        self.pc += 1;
    }

    /// Skip next instruction if vx (register) != nn (constant in). Opcode: `4XNN` - `SNE vx, byte`.
    fn skip_if_vx_ne_nn(&mut self, opcode: u16) {
        let register_number = (opcode & 0x0F00) >> 8;
        let register_value = self.registers[register_number as usize];
        let number_in = opcode & 0x00FF;
        if register_value != number_in as u8 {
            self.pc += 1;
        }
        self.pc += 1;
    }

    /// Skip next instruction if vx (register) == vy (register). Opcode: `5XY0` - `SE vx, vy`.
    fn skip_if_vx_eq_vy(&mut self, opcode: u16) {
        // Get number of the registers vx and vy
        let vx_number = (opcode & 0x0F00) >> 8;
        let vy_number = (opcode & 0x00F0) >> 4;
        // Get their values
        let vx_value = self.registers[vx_number as usize];
        let vy_value = self.registers[vy_number as usize];
        if vx_value == vy_value {
            self.pc += 1;
        }
        self.pc += 1;
    }

    /// vx = n., i.e. put value nn into register vx. Opcode: `6XNN` - `LD vx, byte`.
    fn set_vx_to_n(&mut self, opcode: u16) {
        let vx_number = (opcode & 0x0F00) >> 8;
        let number_in = opcode & 0x00FF;
        self.registers[vx_number as usize] = number_in as u8;
        self.pc += 1;
    }

    /// vx += n, i.e. adds the constant n to register vx. Opcode: `7XNN` - `ADD vx, byte`.
    fn add_n_to_vx(&mut self, opcode: u16) {
        let vx_number = (opcode & 0x0F00) >> 8;
        let number_in = opcode & 0x00FF;
        self.registers[vx_number as usize] += number_in as u8;
        self.pc += 1;
    }

    /// vx = vy, i.e. sets register vx to the value of register vy. Opcode: `8XY0` - `LD vx, vy`.
    fn set_vx_to_vy(&mut self, opcode: u16) {
        let vx_number = (opcode & 0x0F00) >> 8;
        let vy_number = (opcode & 0x00F0) >> 4;
        self.registers[vx_number as usize] = self.registers[vy_number as usize];
        self.pc += 1;
    }

    /// vx |= vy, i.e. sets register vx to vx bitwise or vy. Opcode: `8XY1` - `OR vx, vy`.
    fn set_vx_to_vx_bitor_vy(&mut self, opcode: u16) {
        let vx = (opcode & 0x0F00) >> 8;
        let vy = (opcode & 0x00F0) >> 4;
        self.registers[vx as usize] |= self.registers[vy as usize];
        self.pc += 1;
    }

    /// vx &= vy, i.e. sets register vx to vx bitwise and vy. Opcode: `8XY2` - `AND vx, vy`.
    fn set_vx_to_vx_bitand_vy(&mut self, opcode: u16) {
        let vx = (opcode & 0x0F00) >> 8;
        let vy = (opcode & 0x00F0) >> 4;
        self.registers[vx as usize] &= self.registers[vy as usize];
        self.pc += 1;
    }

    /// vx ^= vy, i.e. sets register vx to vx xor vy. Opcode: `8XY3` - `XOR vx, vy`.
    fn set_vx_to_vx_xor_vy(&mut self, opcode: u16) {
        let vx = (opcode & 0x0F00) >> 8;
        let vy = (opcode & 0x00F0) >> 4;
        self.registers[vx as usize] ^= self.registers[vy as usize];
        self.pc += 1;
    }

    /// vx += vy, i.e. sets register vx to vx plus vy. Opcode: `8XY4` - `ADD vx, vy`.
    fn add_vy_to_vx(&mut self, opcode: u16) {
        let vx = (opcode & 0x0F00) >> 8;
        let vy = (opcode & 0x00F0) >> 4;
        self.registers[vx as usize] += self.registers[vy as usize];
        self.pc += 1;
    }

    /// vx -= vy, i.e. sets register vx to vx minus vy. Opcode: `8XY5` - `SUB vx, vy`.
    fn subtract_vy_from_vx(&mut self, opcode: u16) {
        let vx = (opcode & 0x0F00) >> 8;
        let vy = (opcode & 0x00F0) >> 4;
        self.registers[vx as usize] -= self.registers[vy as usize];
        self.pc += 1;
    }

    /// vx >>= 1, i.e. stores the least significant bit of VX in VF and shift the register VX one to the right.
    /// Opcode: `8XY6` - `SHR vx`. `Y` is a don't care.
    fn right_shift_vx(&mut self, opcode: u16) {
        let vx = (opcode & 0x0F00) >> 8;
        self.registers[0xF] = self.registers[vx as usize] & 0b1;
        self.registers[vx as usize] >>= 1;
        self.pc += 1;
    }

    /// vx = vy - vx, i.e. sets register vx to vx minus vy. Opcode: `8XY7` - `SUBN vx, vy`.
    fn set_vx_to_vy_minus_vx(&mut self, opcode: u16) {
        let vx = (opcode & 0x0F00) >> 8;
        let vy = (opcode & 0x00F0) >> 4;
        self.registers[vx as usize] = self.registers[vy as usize] - self.registers[vx as usize];
        self.pc += 1;
    }

    /// vx <<= 1, i.e. stores the most significant bit of VX in VF and shift the register VX one to the left.
    /// Opcode: `8XYE` - `SHL vx`. `Y` is a don't care.
    fn left_shift_vx(&mut self, opcode: u16) {
        let vx = (opcode & 0x0F00) >> 8;
        self.registers[0xF] = (self.registers[vx as usize] & 0x80) >> 7;
        self.registers[vx as usize] <<= 1;
        self.pc += 1;
    }

    /// Skip next instruction if vx (register) != vy (register). Opcode: `9XY0` - `SNE vx, vy`.
    fn skip_if_vx_ne_vy(&mut self, opcode: u16) {
        // Get number of the registers vx and vy
        let vx_number = (opcode & 0x0F00) >> 8;
        let vy_number = (opcode & 0x00F0) >> 4;
        // Get their values
        let vx_value = self.registers[vx_number as usize];
        let vy_value = self.registers[vy_number as usize];
        if vx_value != vy_value {
            self.pc += 1;
        }
        self.pc += 1;
    }

    /// I = n, i.e. sets the I address register to the number n. Opcode: `ANNN` - `LD I, addr`.
    fn set_i_addr_to_n(&mut self, opcode: u16) {
        let n = opcode & 0x0FFF;
        self.address_register = n;
        self.pc += 1;
    }

    /// I = V0 + n, i.e. sets the I address register to register V0 plus n. Opcode: `BNNN` - `JP V0, addr`.
    fn jump_to_n_plus_v0(&mut self, opcode: u16) {
        let n = opcode & 0x0FFF;
        self.address_register + self.registers[0] as u16 + n;
        self.pc += 1;
    }

    /// `vx = rand()`, i.e. sets `vx` to a random number combined with a bitwise or with n to limit the maximum value.
    /// Opcode: `CXNN` - `RND vx, byte`
    fn set_to_vx_rand_bitand_n(&mut self, opcode: u16) {
        let vx = (opcode & 0x0F00) >> 8;
        let n = opcode & 0x00FF;
        let rand = self.pc + self.current_key as usize + self.stack_pointer as usize;
        self.registers[vx as usize] = (rand as u8) & n as u8;
        self.pc += 1;
    }

    /// Draws a sprite at the coordinates (vx, vy), so the numbers stored in the registers vx and vy, with height n
    /// and width 8. The data is fetched from the memory address stored in the register I. Register vf is set to 1 if
    /// any screen pixels are flipped from set to unset to allow for collision detection.
    fn draw_sprite_at_coordinates_vx_vy_with_height_n(&mut self, opcode: u16) {
        let height = (opcode & 0x000F) as usize;
        let register_vx = (opcode & 0x0F00) >> 8;
        let register_vy = (opcode & 0x00F0) >> 4;
        // Coordinates
        let x = self.registers[register_vx as usize] as usize;
        let y = self.registers[register_vy as usize] as usize;

        for line_nr in 0..height {
            let sprite = self.mem[self.address_register as usize + line_nr];
            let display_line = self.display[y + line_nr][x];
            let new_display_line = display_line ^ sprite;
            self.display[y + line_nr][x] = new_display_line;

            let flip_from_set_to_unset = new_display_line < display_line; // TODO
            self.registers[0xF] = flip_from_set_to_unset as u8;
        }
        self.pc += 1;
    }

    /// Skips the next instruction if the key stored in vx is pressed. Opcode: `EX9E` - `SKP vx`.
    fn skip_if_key_in_vk_pressed(&mut self, opcode: u16) {
        let vx = (opcode & 0x0F00) >> 8;
        if self.current_key == self.registers[vx as usize] {
            self.pc += 1;
        }
        self.pc += 1;
    }

    /// Skips the next instruction if the key stored in vx is not pressed. Opcode: `EX9E` - `SKNP vx`.
    fn skip_if_key_in_vk_not_pressed(&mut self, opcode: u16) {
        let vx = (opcode & 0x0F00) >> 8;
        if self.current_key != self.registers[vx as usize] {
            self.pc += 1;
        }
        self.pc += 1;
    }

    /// `vx = get_delay_timer()`, i.e. sets register `vx` to the value of the delay time. Opcode: `CXNN` - `RND vx,
    /// byte`.
    fn set_vx_to_delay_timer(&mut self, opcode: u16) {
        let vx = (opcode & 0x0F00) >> 8;
        self.registers[vx as usize] = self.delay_timer;
        self.pc += 1;
    }

    /// `vx = get_key()`, i.e. waits for a user input and writes that key into register `vx`. Opcode: `FX0A` - `LD
    /// vx, key`.
    fn set_vx_to_get_key_blocking(&mut self, opcode: u16) {
        let vx = (opcode & 0x0F00) >> 8;
        let key = 0; // TODO
        self.registers[vx as usize] = key;
        self.pc += 1;
    }

    /// `delay_timer = vx`, i.e. sets the delay timer to the value of the register `vx`. Opcode: `FX15` - `LD DT, vx`.
    fn set_delay_timer(&mut self, opcode: u16) {
        let vx = (opcode & 0x0F00) >> 8;
        self.delay_timer = self.registers[vx as usize];
        self.pc += 1;
    }

    /// `sound_timer = vx`, i.e. sets the sound timer to the value of the register `vx`. Opcode: `FX18` - `LD ST, vx`.
    fn set_sound_timer(&mut self, opcode: u16) {
        let vx = (opcode & 0x0F00) >> 8;
        self.sound_timer = self.registers[vx as usize];
        self.pc += 1;
    }

    /// `I += vx`, i.e. adds the register `vx` to the address register `I`. Opcode: `FX1E` - `ADD I, vx`.
    fn add_vx_to_i(&mut self, opcode: u16) {
        let vx = (opcode & 0x0F00) >> 8;
        self.address_register += self.registers[vx as usize] as u16;
        self.pc += 1;
    }

    // `I = sprite_addr[vx]`, i.e. sets the address register `I` to the address of the sprite for the char in `vx`.
    // Opcode: `FX1E` - `LD F, vx`.
    fn set_addr_register_to_char(&mut self, opcode: u16) {
        let vx = (opcode & 0x0F00) >> 8;
        self.address_register = vx * 5; // Each char uses 5 bytes of memory
        self.pc += 1;
    }

    /// Writes the binary-coded decimal representation of `vx` with the most significant of the three bcd digits at
    /// the address `I`, the middle at `I + 1`, the least significant bit at `I + 2`. Opcode: `FX33` - `LD B, vx`.
    fn write_bcd_of_vx_at_i(&mut self, opcode: u16) {
        let vx = (opcode & 0x0F00) >> 8;
        let vx_value: u8 = self.registers[vx as usize];
        self.mem[self.address_register as usize] = vx_value / 100; // Most significant bit
        self.mem[(self.address_register + 1) as usize] = (vx_value / 10) % 10;
        self.mem[(self.address_register + 2) as usize] = vx_value % 10; // Least significant bit
        self.pc += 1;
    }

    /// `reg_dump(vx, &I)`, i.e. writes the value of the registers `v0` to `vx` to memory starting at address `I`.
    /// Opcode: `FX55` -`LD [I], vx`.
    fn dump_registers_to_mem(&mut self, opcode: u16) {
        let vx = ((opcode & 0x0F00) >> 8) as usize;
        for vi in 0..=vx {
            self.mem[self.address_register as usize + vi] = self.registers[vi];
        }
        self.pc += 1;
    }

    /// `reg_load(vx, &I)`, i.e. writes the value of memory starting at address `I` to the registers `v0` to `vx`.
    /// Opcode: `FX65` - `LD vx, [I]`.
    fn load_registers_from_memory(&mut self, opcode: u16) {
        let vx = ((opcode & 0x0F00) >> 8) as usize;
        for vi in 0..=vx {
            self.registers[vi] = self.mem[self.address_register as usize + vi];
        }
        self.pc += 1;
    }
}

impl Default for Chip8 {
    fn default() -> Self {
        Self {
            mem: [0; 4096],
            registers: Default::default(),
            address_register: 0,
            pc: 0,
            stack: Default::default(),
            stack_pointer: 0,
            display: [[0; 8]; 32],
            current_key: 0,
            delay_timer: 0,
            sound_timer: 0
        }
    }
}

fn main() {
    let file_path = "src/PONG";
    let mut file = File::open(file_path).expect("Can't open file");
    let mut chip8 = Chip8::default();
    file.read_exact(&mut chip8.mem);

    chip8.run();
}
