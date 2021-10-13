
/// Things to mention:
/// * vx means register number x.
/// * nn is a constant number (called `number_in`) supplied in the opcode.
struct Chip8 {
    mem: [u8; 4096],
    /// Registers (V) called V0, V1, ..., V9, VA, VB, ..., VF (hex number of the register is appended).
    registers: [u8; 16],
    /// Address register (I).
    addr_register_i: u16,
    /// Program counter (PC).
    pc: usize,
    stack: [u16; 12],
    stack_pointer: u8,

    /// The display as a bit array. Access like `display[y][x]`.
    display: [[u8; 8]; 32],
    /// Current key pressed by the user.
    current_key: u8,
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

    fn exec_instruction(&mut self) {
        let opcode = self.load_opcode();

        if opcode & 0xF000 == 0x0000 {
            self.call_machine_routine(opcode);
        } else if opcode & 0x00F0 == 0x00E0 {
            // Clear display
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
            self.set_vx_to_rand_with_bitwise_and(opcode);
        } else if opcode & 0xF000 == 0xD000 {
            self.draw_sprite_at_coordinates_vx_vy_with_height_n(opcode);
        }

        else if opcode & 0xF0FF == 0xE09E {
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

    /// Return from subroutine. Opcode: `00EE` - `RET`.
    fn subroutine_return(&mut self) {
        self.pc = self.stack[self.stack_pointer];
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
        self.stack[self.stack_pointer] = self.pc;
        let subroutine_mem_addr = opcode & 0x0FFF;
        self.pc = subroutine_mem_addr as usize;
    }

    /// Skip next instruction if vx (register) == nn (constant in). Opcode: `3XNN` - `SE vx, byte`.
    fn skip_if_vx_eq_nn(&mut self, opcode: u16) {
        let register_number = (opcode & 0x0F00) >> 8;
        let register_value = self.registers[register_number];
        let number_in = opcode & 0x00FF;
        if register_value == number_in {
            self.pc += 1;
        }
        self.pc += 1;
    }

    /// Skip next instruction if vx (register) != nn (constant in). Opcode: `4XNN` - `SNE vx, byte`.
    fn skip_if_vx_ne_nn(&mut self, opcode: u16) {
        let register_number = (opcode & 0x0F00) >> 8;
        let register_value = self.registers[register_number];
        let number_in = opcode & 0x00FF;
        if register_value != number_in {
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
        let vx_value = self.registers[vx_number];
        let vy_value = self.registers[vy_number];
        if vx_value == vy_value {
            self.pc += 1;
        }
        self.pc += 1;
    }

    /// vx = n., i.e. put value nn into register vx. Opcode: `6XNN` - `LD vx, byte`.
    fn set_vx_to_n(&mut self, opcode: u16) {
        let vx_number = (opcode & 0x0F00) >> 4;
        let number_in = opcode & 0x00FF;
        self.registers[vx_number] = number_in;
        self.pc += 1;
    }

    /// vx += n, i.e. adds the constant n to register vx. Opcode: `7XNN` - `ADD vx, byte`.
    fn add_n_to_vx(&mut self, opcode: u16) {
        let vx_number = (opcode & 0x0F00) >> 8;
        let number_in = opcode & 0x00FF;
        self.registers[vx_number] += number_in;
        self.pc += 1;
    }

    /// vx = vy, i.e. sets register vx to the value of register vy. Opcode: `8XY0` - `LD vx, vy`.
    fn set_vx_to_vy(&mut self, opcode: u16) {
        let vx_number = (opcode & 0x0F00) >> 8;
        let vy_number = (opcode & 0x00F0) >> 4;
        self.registers[vx_number] = self.registers[vy_number];
        self.pc += 1;
    }

    /// vx |= vy, i.e. sets register vx to vx bitwise or vy. Opcode: `8XY1` - `OR vx, vy`.
    fn set_vx_to_vx_bitor_vy(&mut self, opcode: u16) {
        let vx = (opcode & 0x0F00) >> 8;
        let vy = (opcode & 0x00F0) >> 4;
        self.registers[vx] |= self.registers[vy];
        self.pc += 1;
    }

    /// vx &= vy, i.e. sets register vx to vx bitwise and vy. Opcode: `8XY2` - `AND vx, vy`.
    fn set_vx_to_vx_bitand_vy(&mut self, opcode: u16) {
        let vx = (opcode & 0x0F00) >> 8;
        let vy = (opcode & 0x00F0) >> 4;
        self.registers[vx] &= self.registers[vy];
        self.pc += 1;
    }

    /// vx ^= vy, i.e. sets register vx to vx xor vy. Opcode: `8XY3` - `XOR vx, vy`.
    fn set_vx_to_vx_xor_vy(&mut self, opcode: u16) {
        let vx = (opcode & 0x0F00) >> 8;
        let vy = (opcode & 0x00F0) >> 4;
        self.registers[vx] ^= self.registers[vy];
        self.pc += 1;
    }

    /// vx += vy, i.e. sets register vx to vx plus vy. Opcode: `8XY4` - `ADD vx, vy`.
    fn add_vy_to_vx(&mut self, opcode: u16) {
        let vx = (opcode & 0x0F00) >> 8;
        let vy = (opcode & 0x00F0) >> 4;
        self.registers[vx] += self.registers[vy];
        self.pc += 1;
    }

    /// vx -= vy, i.e. sets register vx to vx minus vy. Opcode: `8XY5` - `SUB vx, vy`.
    fn subtract_vy_from_vx(&mut self, opcode: u16) {
        let vx = (opcode & 0x0F00) >> 8;
        let vy = (opcode & 0x00F0) >> 4;
        self.registers[vx] -= self.registers[vy];
        self.pc += 1;
    }

    /// vx >>= 1, i.e. stores the least significant bit of VX in VF and shift the register VX one to the right.
    /// Opcode: `8XY6` - `SHR vx`. `Y` is a don't care.
    fn right_shift_vx(&mut self, opcode: u16) {
        let vx = (opcode & 0x0F00) >> 8;
        self.registers[0xF] = self.registers[vx] & 0b1;
        self.registers[vx] >>= 1;
        self.pc += 1;
    }

    /// vx -= vy, i.e. sets register vx to vx minus vy. Opcode: `8XY7` - `SUBN vx, vy`.
    fn subtract_vx_from_vy_write_to_vx(&mut self, opcode: u16) {
        let vx = (opcode & 0x0F00) >> 8;
        let vy = (opcode & 0x00F0) >> 4;
        self.registers[vx] = self.registers[vy] - self.registers[vx];
        self.pc += 1;
    }

    /// vx <<= 1, i.e. stores the most significant bit of VX in VF and shift the register VX one to the left.
    /// Opcode: `8XYE` - `SHL vx`. `Y` is a don't care.
    fn left_shift_vx(&mut self, opcode: u16) {
        let vx = (opcode & 0x0F00) >> 8;
        self.registers[0xF] = (self.registers[vx] & 0x80) >> 7;
        self.registers[vx] <<= 1;
        self.pc += 1;
    }

    /// Skip next instruction if vx (register) != vy (register). Opcode: `9XY0` - `SNE vx, vy`.
    fn skip_if_vx_ne_vy(&mut self, opcode: u16) {
        // Get number of the registers vx and vy
        let vx_number = (opcode & 0x0F00) >> 8;
        let vy_number = (opcode & 0x00F0) >> 4;
        // Get their values
        let vx_value = self.registers[vx_number];
        let vy_value = self.registers[vy_number];
        if vx_value != vy_value {
            self.pc += 1;
        }
        self.pc += 1;
    }

    /// Draws a sprite at the coordinates (vx, vy), so the numbers stored in the registers vx and vy, with height n
    /// and width 8. The data is fetched from the memory address stored in the register I. Register vf is set to 1 if
    /// any screen pixels are flipped from set to unset to allow for collision detection.
    fn draw_sprite_at_coordinates_vx_vy_with_height_n(&mut self, opcode: u16) {
        let height = opcode & 0x000F;
        let register_vx = (opcode & 0x0F00) >> 12;
        let register_vy = (opcode & 0x00F0) >> 8;
        // Coordinates
        let x = self.registers[register_vx];
        let y = self.registers[register_vy];

        for line_nr in 0..height {
            let sprite = self.mem[self.addr_register_i + line_nr];
            let display_line = self.display[y + line_nr][x];
            let new_display_line = display_line ^ sprite;
            self.display[y + line_nr][x] = new_display_line;

            let flip_from_set_to_unset = new_display_line < display_line; // TODO
            self.registers[0xF] = flip_from_set_to_unset as u8;
        }
    }

    /// Skips the next instruction if the key stored in vx is pressed. `EX9E` - `SKP vx`.
    fn skip_if_key_in_vk_pressed(&mut self, opcode: u16) {
        let vx = (opcode & 0x0F00) >> 12;
        if self.current_key == self.registers[vx] {
            self.pc += 1;
        }
        self.pc += 1;
    }

    /// Skips the next instruction if the key stored in vx is not pressed. `EX9E` - `SKNP vx`.
    fn skip_if_key_in_vk_not_pressed(&mut self, opcode: u16) {
        let vx = (opcode & 0x0F00) >> 12;
        if self.current_key != self.registers[vx] {
            self.pc += 1;
        }
        self.pc += 1;
    }
}

fn main() {
    println!("Hello, world!");
}
