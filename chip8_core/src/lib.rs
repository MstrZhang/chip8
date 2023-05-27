use rand::random;

// exposed to the "frontend" for rendering purposes
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

// size in bytes
const RAM_SIZE: usize = 4096;
// implementing stack from scratch since wasm doesn't fully support std
const STACK_SIZE: usize = 16;
// 16 possible keys numbered 0x0 to 0xF
const NUM_KEYS: usize = 16;
// CHIP-8 loads ROM into RAM at an offset of 512 bytes
const START_ADDR: u16 = 0x200;
// 16 V registers (from V0 to VF)
const NUM_REGS: usize = 16;
// sprites are 8 pixels wide and 5 pixels high
const FONTSET_SIZE: usize = 80;

// number sprites
const FONTSET: [u8; FONTSET_SIZE] = [
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

pub struct Emu {
    // program counter: keeps track of index of current instruction
    pc: u16,

    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REGS],

    // i register: used for indexing into RAM for reads and writes
    i_reg: u16,

    // stack implementation
    sp: u16,
    stack: [u16; STACK_SIZE],

    keys: [bool; NUM_KEYS],

    // delay timer (countdown) and sound timer (emits sound at 0)
    dt: u8,
    st: u8,
}

impl Emu {
    // constructor
    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            // screen is a 1D array of boolean values (represents flipped / unflipped pixels)
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };

        // copies all font sprites into RAM
        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        new_emu
    }

    // stack push operation
    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    // stack pop operation
    fn pop(&mut self) -> u16 {
        // pop at 0 will cause underflow (i.e. rust panic)
        // this situation will only be caused if there is a bug in the emulator / game so is left unhandled
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    // reset back to initial state
    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    // cpu tick operation
    pub fn tick(&mut self) {
        // fetch
        let op = self.fetch();
        // decode & execute
        self.execute(op);
    }

    // cpu fetch operation
    fn fetch(&mut self) -> u16 {
        // CHIP-8 opcodes are exactly 2 bytes
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;

        // store values in RAM as 8-bit values (fetch two and combine as Big Endian)
        //  - bitshift left `higher_byte` by 8 bytes (to convert to 8-bit)
        //  - don't need to do this for `lower_byte` since it already has correct bytes
        //  - bitwise OR to combine into single `op` variable (https://en.wikipedia.org/wiki/Bitwise_operation#OR)
        let op = (higher_byte << 8) | lower_byte;

        // proceed to next opcode
        self.pc += 2;

        op
    }

    // handle dt and st timers
    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            if self.st == 1 {
                // BEEP! (not covered in the guide)
            }
            self.st -= 1
        }
    }

    // pass pointer to screen buffer array to frontend
    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    // handle keypress
    pub fn keypress(&mut self, index: usize, pressed: bool) {
        // frontend handles key presses and sends it to this function
        // write the caught keypresses into the `keys` array
        // handles both keyup and keydown (toggles `pressed` to true or false accordingly)
        // `index` needs to be under 16 or rust will panic (assume its correct here; handle in the frontend)
        self.keys[index] = pressed;
    }

    // load ROM file into RAM
    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        // copy all values from input `data` and slice it into RAM beginning at 0x200 (i.e. `START_ADDR`)
        self.ram[start..end].copy_from_slice(data);
    }

    // cpu execute operation
    fn execute(&mut self, op: u16) {
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            // NOP: 0x0000 - no operation
            (0, 0, 0, 0) => return,

            // CLS: 0x00E0 - clear screen
            (0, 0, 0xE, 0) => {
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            }

            // RET: 0x00EE - return from subroutine
            (0, 0, 0xE, 0xE) => {
                // subroutine is like a jump but is expected to complete at some point (i.e. need to return to entry at some point)
                //  - store current address in stack
                //  - pop from stack when we need to return
                let ret_addr = self.pop();
                self.pc = ret_addr;
            }

            // JMP NNN: 0x1NNN - jump to given address
            (1, _, _, _) => {
                let nnn = op & 0xFFF;
                self.pc = nnn;
            }

            // CALL NNN: 0x2NNN - call subroutine
            (2, _, _, _) => {
                let nnn = op & 0xFFF;

                // add current address to stack
                self.push(self.pc);
                // move pc to address
                self.pc = nnn;
            }

            // SKIP VX == NN: 0x3XNN - skip next if VX == NN
            (3, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;

                // skip to next operation if VX == NN
                if self.v_reg[x] == nn {
                    self.pc += 2;
                }
            }
            // SKIP VX != NN: 0x4XNN - skip next if VX != NN
            (4, _, _, _) => {
                let x: usize = digit2 as usize;
                let nn: u8 = (op & 0xFF) as u8;

                // skip to next operation if VX != NN
                if self.v_reg[x] != nn {
                    self.pc += 2;
                }
            }

            // SKIP VX == VY: 0x5XY0 - skip next if VX == VY
            (5, _, _, 0) => {
                // least significant digit is not used in this operation
                // opcode requires it to be 0

                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            }

            // VX = NN: 0x6XNN - assign VX = NN
            (6, _, _, _) => {
                let x: usize = digit2 as usize;
                let nn: u8 = (op & 0xFF) as u8;
                self.v_reg[x] = nn;
            }

            // VX += NN: 0x7XNN - add NN to VX register
            (7, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;

                // rust will panic in the event of an overflow
                // `wrapping_add` will wrap the value around 0 in the event of an overflow
                self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
            }

            // VX = VY: 0x8XY0 - assign VX to value in VY
            (8, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] = self.v_reg[y];
            }

            // VX |= VY: 0x8XY1 - bitwise OR
            (8, _, _, 1) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] |= self.v_reg[y];
            }

            // VX &= VY: 0x8XY2 - bitwise AND
            (8, _, _, 2) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] &= self.v_reg[y];
            }

            // VX ^= VY: 0x8XY3 - bitwise XOR
            (8, _, _, 3) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] ^= self.v_reg[y];
            }

            // VX += VY: 0x8XY4 - addition assignment of VX and VY
            (8, _, _, 4) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                // this operation could potentially overflow
                let (new_vx, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);
                // assign carry bit if necessary
                let new_vf = if carry { 1 } else { 0 };

                // assign new value
                self.v_reg[x] = new_vx;
                // assign carry bit (16th V-register); 1 for overflow and 0 otherwise
                self.v_reg[0xF] = new_vf;
            }

            // VX -= VY: 0x8XY5 - subtraction assignment of VX and VY
            (8, _, _, 5) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                // same idea as VX += VY but could potentially underflow
                let (new_vx, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);
                // assign value to 0 if underflow occurred otherwise 1
                let new_vf = if borrow { 0 } else { 1 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            }

            // VX >>= 1: 0x8XY6 - bitwise right shift on VX
            (8, _, _, 6) => {
                let x = digit2 as usize;
                // catch dropped least-significant bit
                let lsb = self.v_reg[x] & 1;

                self.v_reg[x] >>= 1;
                // store least-significant bit in flag register
                self.v_reg[0xF] = lsb;
            }

            // VX = VY - VX: 0x8XY7 - subtraction assignment of VX and VY with operands reversed
            (8, _, _, 7) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_reg[y].overflowing_sub(self.v_reg[x]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            }

            // VX <<= 1: 0x0XYE - bitwise left shift on VX
            (8, _, _, 0xE) => {
                let x = digit2 as usize;
                // catch dropped most-significant bit
                let msb = (self.v_reg[x] >> 7) & 1;

                self.v_reg[x] <<= 1;
                // store most-significant bit in flag register
                self.v_reg[0xF] = msb;
            }

            // SKIP VX != VY: 0x9XY0 - skip if VX != VY
            (9, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                if self.v_reg[x] != self.v_reg[y] {
                    self.pc += 2;
                }
            }

            // I = NNN: 0xANNN - assign I-register to 0xNNN
            (0xA, _, _, _) => {
                let nnn = op & 0xFFF;
                self.i_reg = nnn;
            }

            // JMP V0 + NNN: 0xBNNN - jump to V0 + 0xNNN
            (0xB, _, _, _) => {
                let nnn = op & 0xFFF;
                self.pc = (self.v_reg[0] as u16) + nnn;
            }

            // VX = rand() & NN: 0xCXNN - random number generator
            (0xC, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                let rng: u8 = random();

                // CHIP-8 rng AND's the value with the given 0xNN value
                self.v_reg[x] = rng & nn;
            }

            // DRAW: 0xDXYN - draw sprite at (X, Y) of height N
            (0xD, _, _, _) => {
                // overview:
                //  - CHIP-8 sprites are always 8 pixels wide but can be between 1 to 16 pixels tall
                //  - the height is specified by the `N` value in the opcode
                //  - sprites are stored row-by-row beginning at the address stored in the I-register
                //  - if any pixel is flipped from black to white (or vice-versa) the VF register is set and cleared

                // get the (x, y) coordinates of our sprite
                let x_coord = self.v_reg[digit2 as usize] as u16;
                let y_coord = self.v_reg[digit3 as usize] as u16;
                // the last digit determines how many rows high the sprite is
                let num_rows = digit4;

                // keep track if any pixels were flipped
                let mut flipped = false;

                for y_line in 0..num_rows {
                    // determine which memory address the row's data is stored
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];

                    for x_line in 0..8 {
                        // use a mask to fetch the current pixel's bit. only flip if it is a 1
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            // sprites should wrap around the screen so apply a modulo
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;

                            // get pixel's index
                            // screen is a 1D array so calculate the index value accordingly
                            let index = x + SCREEN_WIDTH * y;
                            // check if we're about to flip the pixel and set
                            flipped |= self.screen[index];
                            self.screen[index] ^= true;
                        }
                    }
                }

                // populate the VF register
                if flipped {
                    self.v_reg[0xF] = 1;
                } else {
                    self.v_reg[0xF] = 0;
                }
            }

            // SKIP KEY PRESS: 0xEX9E - skip if key pressed
            (0xE, _, 9, 0xE) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];

                // skip operation if key in VX is the key being pressed
                if key {
                    self.pc += 2;
                }
            }

            // SKIP KEY RELEASE: 0xEXA1 - skip if key not pressed
            (0xE, _, 0xA, 1) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];

                // skip operation if key in VX is not the key being pressed
                if !key {
                    self.pc += 2;
                }
            }

            // VX = DT: 0xFX07 - stores delay timer value in VX
            (0xF, _, 0, 7) => {
                // delay timer ticks automatically
                // this instruction stores it so the value can be read

                let x = digit2 as usize;
                self.v_reg[x] = self.dt;
            }

            // WAIT KEY: 0xFX0A - wait for key press
            (0xF, _, 0, 0xA) => {
                let x = digit2 as usize;
                let mut pressed = false;

                // loop through all the keys currently being pressed
                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        // break out of the loop if the key pressed is the given key
                        self.v_reg[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }

                if !pressed {
                    // if the key isn't pressed we need to block execution
                    // redo the previous opcode
                    // we don't loop endlessly because we need to poll for potential new key presses
                    self.pc -= 2;
                }
            }

            // DT = VX: 0xFX15 - assign delay timer to value in VX
            (0xF, _, 1, 5) => {
                // delay timer does not reset once it hits 0
                // this operation allows us to change its value
                let x = digit2 as usize;
                self.dt = self.v_reg[x];
            }

            // ST = VX: 0xFX18 - assign sound timer to value in VX
            (0xF, _, 1, 8) => {
                let x = digit2 as usize;
                self.st = self.v_reg[x];
            }

            // I += VX: 0xFX1E - increment I-register value by VX
            (0xF, _, 1, 0xE) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x] as u16;
                self.i_reg = self.i_reg.wrapping_add(vx);
            }

            // I = FONT: 0xFX29 - set I to font address
            (0xF, _, 2, 9) => {
                // in the beginning, we stored every number sprite in the beginning of RAM
                // each sprite is 8 pixels wide and 5 pixels tall
                // thus, the RAM address for each sprite is its number * 5 (numbers from 0x0 to 0xF)
                let x = digit2 as usize;
                let c = self.v_reg[x] as u16;
                // offset is conveniently 5 due to how we built the sprites initially
                self.i_reg = c * 5;
            }

            // BCD: 0xFX33 - convert hex number to pseudo-decimal number for display purposes
            (0xF, _, 3, 3) => {
                // this a really naive BCD (binary-coded decimal) algorithm
                //  - converts VX value to float to use division and modulo to get each decimal digit
                //  - not fastest or shortest solution (easiest to understand)

                let x = digit2 as usize;
                let vx = self.v_reg[x] as f32;

                // fetch hundreds digit by dividing by 100 and tossing the decimal
                let hundreds = (vx / 100.0).floor() as u8;
                // fetch the tens digit by dividing by 10 and tossing the ones digit and the decimal
                let tens = ((vx / 10.0) % 10.0).floor() as u8;
                // fetch the ones digit by tossing the hundreds and the tens
                let ones = (vx % 10.0) as u8;

                // store the BCD with 3 bytes in the I-register
                self.ram[self.i_reg as usize] = hundreds;
                self.ram[(self.i_reg + 1) as usize] = tens;
                self.ram[(self.i_reg + 2) as usize] = ones;
            }

            // STORE V0 - VX: 0xFX55 - populate registers V0 to VX (inclusive) into I-register
            (0xF, _, 5, 5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;

                // ..= is inclusive range
                for index in 0..=x {
                    self.ram[i + index] = self.v_reg[index];
                }
            }

            // LOAD V0 - VX: 0xFX65 - load I-register contents into registers V0 to VX (inclusive)
            (0xF, _, 6, 5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;
                for index in 0..=x {
                    self.v_reg[index] = self.ram[i + index];
                }
            }

            // base case: unimplemented op code; force rust to panic
            (_, _, _, _) => unimplemented!("unimplemented opcode: {}", op),
        }
    }
}
