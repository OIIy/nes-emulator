// Define CPU and its registers
pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub status: u8,
    pub program_counter: u16,
    memory: [u8; 0xFFFF]
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            status: 0,
            program_counter: 0,
            memory: [0; 0xFFFF]
        }
    }

    pub fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    } 

    pub fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.run()
    }

    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000 .. (0x8000 + program.len())].copy_from_slice(&program[..]);
        self.program_counter = 0x8000;
    }

    pub fn run(&mut self) {
        self.program_counter = 0;

        loop {
            let opscode = self.mem_read(self.program_counter);
            self.program_counter += 1;

            match opscode {
                0xA9 => {
                    // Get parameter from program counter register
                    let param = self.mem_read(self.program_counter);
                    // Causes param to be skipped for next loop iteration
                    self.lda(param);
                }
                0xAA => self.tax(),
                0xE8 => self.inx(),
                0x00 => return,
                _ => todo!()
            }
        }
    }

    fn inx(&mut self) {
        if self.register_x == 255 {
            self.register_x = 0
        } else {
            self.register_x += 1;
        }
        self.update_zero_and_negative_flags();
    }
    
    fn tax(&mut self) {
        // Copies contents of register_a into register_x 
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags();
    }
    
    fn lda(&mut self, value: u8) {  
        self.program_counter += 1;
        // Load parameter into register_a
        self.register_a = value;
        self.update_zero_and_negative_flags();
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
        cpu.register_a = 5;
        cpu.load_and_run(vec![0xAA, 0xE8, 0x00]);
        assert_eq!(cpu.register_x, 6);
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.register_x = 0xff;
        cpu.load_and_run(vec![0xe8, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 1)
    }

    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);
  
        assert_eq!(cpu.register_x, 0xc1)
    }
}