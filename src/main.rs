#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPage_X,
    ZeroPage_Y,
    Absolute,
    Absolute_X,
    Absolute_Y,
    Indirect_X,
    Indirect_Y,
    NoneAddressing,
}

pub struct OpCode {
    pub instruction: u8,
    pub label: String,
    pub bytes: u8,
    pub cycles: u8,
    pub mode: AddressingMode
}

// Define CPU and its registers
pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: u8,
    pub program_counter: u16,
    memory: [u8; 0xFFFF]
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: 0,
            program_counter: 0,
            memory: [0; 0xFFFF]
        }
    }


    pub fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    } 

    pub fn mem_read_u16(&self, pos: u16) -> u16 {
        // read byte at lower address
        let lo = self.mem_read(pos) as u16;
        // read by at higher address
        let hi = self.mem_read(pos + 1) as u16;
        // combines the hi and lo byte with little endian ordering.
        // shifts the hi byte 8 bits to the left of the lo byte, uses the OR operator to combine
        (hi << 8) | (lo as u16)
    }

    pub fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }

    pub fn mem_write_u16(&mut self, addr: u16, data: u16) {
        // From data, shift the most significant 8 bits into the position of the least significant
        // then truncate, preserving least significant bits
        let hi = (data >> 8) as u8;
        // preserve only the least significant bits by comparing data (16bits) to 8 set bits and
        // then truncate, preserving least significant bits again
        let lo = (data & 0xFF) as u8;

        self.mem_write(addr, lo);
        self.mem_write(addr + 1, hi);
        
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run()
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.status = 0;

        // Reset program to special program start point defined by program ROMs
        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000 .. (0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x8000);
    }

    pub fn run(&mut self) {
        loop {
            let opscode = self.mem_read(self.program_counter);
            self.program_counter += 1;

            // Match by instruction name rather than hexadecimal representation


            match opscode {
                0xA9 => {
                    self.lda(&AddressingMode::Immediate);
                    self.program_counter += 1;
                },
                0xA5 => {
                    self.lda(&AddressingMode::ZeroPage);
                    self.program_counter += 1;
                }
                0xAD => {
                    self.lda(&AddressingMode::Absolute);
                    self.program_counter += 2;
                }
                0xAA => {
                    self.tax();
                },
                0xE8 => {
                    self.inx();
                },
                0x85 => {
                    self.sta(&AddressingMode::ZeroPage);
                    self.program_counter += 1;
                },
                0x95 => {
                    self.sta(&AddressingMode::ZeroPage_X);
                    self.program_counter += 1;
                }
                0x00 => return,
                _ => todo!()
            }
        }
    }

    fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative_flags();
    }
    
    fn tax(&mut self) {
        // Copies contents of register_a into register_x 
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags();
    }
    
    fn lda(&mut self, mode: &AddressingMode) {  
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        // Load parameter into register_a
        self.register_a = value;
        self.update_zero_and_negative_flags();
    }

    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);
    }
    
    pub fn update_zero_and_negative_flags(&mut self) {
        if self.register_a == 0 {
            self.status = self.status | 0b0000_0010;
        } else {
            self.status = self.status & 0b1111_1101;
        }

        if self.register_a & 0b1000_0000 != 0 {
            self.status = self.status | 0b1000_0000;
        } else {
            self.status = self.status & 0b0111_1111;
        }
    }

    pub fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
        match mode {
            // Immediate -> For returning the current memory address
            AddressingMode::Immediate => self.program_counter,

            // Zero Page -> For accessing memory within the first 256 bits of address space, uses 1 byte addressing
            AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,

            // Absolute -> For accessing memory in the whole address space, uses 2 byte addressing
            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),

            // Zero Page + X -> Uses zero page but offets with value in X register
            AddressingMode::ZeroPage_X => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_x) as u16;
                addr
            },

            // Zero Page + Y -> Uses zero page (single byte addressing) with Y register offset
            AddressingMode::ZeroPage_Y => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_y) as u16;
                addr
            },

            // Absolute + X -> Uses 2 double byte addressing (whole address space access) with X register offset
            AddressingMode::Absolute_X => {
                let pos = self.mem_read_u16(self.program_counter);
                let addr = pos.wrapping_add(self.register_x as u16);
                addr
            },

            // Absolute + Y -> Uses 2 byte addressing with Y register offset
            AddressingMode::Absolute_Y => {
                let pos = self.mem_read_u16(self.program_counter);
                let addr = pos.wrapping_add(self.register_y as u16);
                addr
            },

            // Indirect X -> Uses zero page, X addressing and returns the two bytes found. Those two bytes are used as
            // a reference to another address in memory
            AddressingMode::Indirect_X => {
                let base = self.mem_read(self.program_counter);

                let ptr: u8 = (base as u8).wrapping_add(self.register_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
            },

            AddressingMode::Indirect_Y => {
                // Read zero page address
                let base = self.mem_read(self.program_counter);

                // Add Y register to base address 
                let ptr: u8 = (base as u8).wrapping_add(self.register_y);

                // Get the least significant byte first, then the more significant (next) byte
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                // The data is stored with little-endian ordering but returned as normal?
                (hi as u16) << 8 | (lo as u16)
            },
            _ => panic!("OH NO! Addressing mode: {:?} is not supported", mode)
        }
    }
}

fn main() {
    println!("Hello, world!");
}

/* Tests */
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0x05, 0x00]);
        assert_eq!(cpu.register_a, 0x05);
        assert!(cpu.status & 0b0000_0010 == 0);
        assert!(cpu.status & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xa5_lda_zero_page_load_data() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0x55);
        cpu.load_and_run(vec![0xA5, 0x10, 0x00]);

        assert_eq!(cpu.register_a, 0x55); 
    }

    #[test]
    fn test_0xad_lda_absolute_load_data() {
        let mut cpu = CPU::new();
        cpu.mem_write_u16(0x55DD, 0x4455);
        cpu.load_and_run(vec![0xAD, 0xDD, 0x55, 0x00]);

        assert_eq!(cpu.register_a, 0x55);
    }

    #[test]
    fn test_0x85_sta_zero_page_store_a_register() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0xFF, 0x85, 0x81, 0x00]);

        let value = cpu.mem_read(0x81);

        assert_eq!(value, 0xFF);
    }

    #[test]
    fn test_0x95_sta_zero_page_x_store_register_a() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0x01, 0xAA, 0x95, 0x01, 0x00]);

        let value = cpu.mem_read(0x02);

        assert_eq!(value, 0x01);
    }
    
    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0x00, 0x00]);
        assert!(cpu.status & 0b10 == 0b10);
    }

    #[test]
    fn test_0xa9_lda_negative_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0xFF, 0x00]);
        assert!(cpu.status & 0b1000_0000 == 0b1000_0000);
    }

    #[test]
    fn test_0xaa_tax_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0x05, 0xAA, 0x00]);
        assert_eq!(cpu.register_a, cpu.register_x);
    }

    #[test]
    fn test_0xaa_tax_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0x00, 0xAA, 0x00]);
        assert!(cpu.status & 0b10 == 0b10);
    }

    #[test]
    fn test_0xaa_tax_negative_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0xFF, 0xAA, 0x00]);
        assert!(cpu.status & 0b1000_0000 == 0b1000_0000);
    }

    #[test]
    fn test_0xe8_inx_immediate_increment() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0x05, 0xAA, 0xE8, 0x00]);
        assert_eq!(cpu.register_x, 6);
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0xFF, 0xAA, 0xE8, 0xE8, 0x00]);

        assert_eq!(cpu.register_x, 1)
    }

    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);
  
        assert_eq!(cpu.register_x, 0xc1)
    }
}