pub const RAX: &str = "RAX";
pub const RDX: &str = "RDX";
pub const _RAX_: &str = "[RAX]";

pub const PROGRAM_SECTION: &str = ".program";
pub const DATA_SECTION: &str = ".data";

//(Dest)
pub const DEST_NULL: u16 = 0b000 << 3;
pub const DEST_M: u16 = 0b001 << 3;
pub const DEST_D: u16 = 0b010 << 3;
pub const DEST_A: u16 = 0b100 << 3;
pub const DEST_MD: u16 = 0b011 << 3;
pub const DEST_AM: u16 = 0b101 << 3;
pub const DEST_AD: u16 = 0b110 << 3;
pub const DEST_AMD: u16 = 0b111 << 3;

//(Jump)
pub const JNOT: u16 = 0b000;
pub const JGT: u16 = 0b001;
pub const JEQ: u16 = 0b010;
pub const JGE: u16 = 0b011;
pub const JLT: u16 = 0b100;
pub const JNE: u16 = 0b101;
pub const JLE: u16 = 0b110;
pub const JMP: u16 = 0b111;

// Операции ALU (Comp) - примеры для a=0 и a=1
pub const ZERO_OUT: u16 = 0b101010 << 6;
pub const ONE_OUT: u16 = 0b111111 << 6;
pub const MINUS_ONE_OUT: u16 = 0b111010 << 6;
pub const D_OUT: u16 = 0b001100 << 6;
pub const A_OUT: u16 = 0b110000 << 6;
pub const NOT_D: u16 = 0b001101 << 6;
pub const NOT_A: u16 = 0b110001 << 6;
pub const MINUS_D: u16 = 0b001111 << 6;
pub const MINUS_A: u16 = 0b110011 << 6;
pub const D_PLUS_ONE: u16 = 0b011111 << 6;
pub const A_PLUS_ONE: u16 = 0b110111 << 6;
pub const D_MINUS_ONE: u16 = 0b001110 << 6;
pub const A_MINUS_ONE: u16 = 0b110010 << 6;
pub const D_PLUS_A: u16 = 0b000010 << 6;
pub const D_MINUS_A: u16 = 0b010011 << 6;
pub const A_MINUS_D: u16 = 0b000111 << 6;
pub const D_AND_A: u16 = 0b000000 << 6;
pub const D_OR_A: u16 = 0b010101 << 6;

//Instruction type
pub const C_INSTRUCTION: u16 = 0b1 << 15;
pub const A_INSTRUCTION: u16 = 0b0 << 15;

//Memory on/off
pub const M_ON: u16 = 0b1 << 12;
pub const M_OFF: u16 = 0b0 << 12;

//Registers
pub const R1: (&str, u16) = ("R1", 0);
pub const R2: (&str, u16) = ("R2", 1);
pub const R3: (&str, u16) = ("R3", 2);
pub const R4: (&str, u16) = ("R4", 3);
pub const R5: (&str, u16) = ("R5", 4);
pub const R6: (&str, u16) = ("R6", 5);
pub const R7: (&str, u16) = ("R7", 6);
pub const R8: (&str, u16) = ("R8", 7);
pub const R9: (&str, u16) = ("R9", 8);
pub const R10: (&str, u16) = ("R10", 9);
pub const R11: (&str, u16) = ("R11", 10);
pub const R12: (&str, u16) = ("R12", 11);
pub const R13: (&str, u16) = ("R13", 12);
pub const R14: (&str, u16) = ("R14", 13);
pub const R15: (&str, u16) = ("R15", 14);
pub const R16: (&str, u16) = ("R16", 15);
