use std::fs::File;
use std::io::{self, BufRead, BufReader};

use anyhow::anyhow;
use phf::phf_ordered_set;

use crate::utils::symbols::{
    A_MINUS_D, A_MINUS_ONE, A_OUT, A_PLUS_ONE, C_INSTRUCTION, D_MINUS_A, D_MINUS_ONE, D_OUT,
    D_PLUS_A, D_PLUS_ONE, DEST_A, DEST_D, DEST_M, DEST_NULL, JEQ, JGE, JGT, JLE, JLT, JMP, JNE,
    JNOT, M_ON, MINUS_ONE_OUT, ONE_OUT, ZERO_OUT,
};

// use crate::utils::symbols::MNEMONICS;

pub mod utils;

#[derive(Debug)]
struct Token {
    pub mnemonic: String,
    pub dst: String,
    pub var1: Option<String>,
    pub var2: Option<String>,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    //1.Open a file
    //
    //2.Tokenizer
    //3.0: prefill symbols hash table with rezerved data (R1-R16)
    //3.Symbols table
    //4.Code generator
    //5.Save in .hex, .bin formats
    let mut buf_reader = open_file("asm/multiplication.asm")?;
    let mut tokens = tokenize(&mut buf_reader)?;
    let mut instructions = generate_instructions(&tokens)?;

    Ok(())
}

//1. Remove comments, whitespaces
//2. Process consts
//3. Process data
//4. Process program

pub fn open_file(file_path: &str) -> anyhow::Result<BufReader<File>> {
    let file = File::open(file_path)?;
    Ok(io::BufReader::new(file))
}

fn process_data_section() {}
fn process_const_section() {}
fn process_program_section() {}
fn prefill_symbols_table() {}
fn generate_symbols_table() {}
fn save_file() {}

pub fn tokenize(buf_reader: &mut io::BufReader<File>) -> anyhow::Result<Vec<Token>> {
    let mut tokens: Vec<Token> = vec![];
    let filters: [char; 1] = [' '];

    for line in buf_reader.lines() {
        let line = line?;

        let mut skip = false;
        let data: Vec<&str> = line
            .split(&filters)
            .filter(|t| {
                if skip {
                    return false;
                }
                if t.contains(';') {
                    skip = true;
                    return false;
                }

                if t.is_empty() {
                    return false;
                }
                true
            })
            .collect();

        if data.is_empty() {
            continue;
        }

        let token = Token {
            mnemonic: match data.get(0) {
                Some(v) => v.to_string(),
                None => panic!(),
            },
            dst: match data.get(1) {
                Some(v) => v.to_string(),
                None => panic!(),
            },
            var1: match data.get(2) {
                Some(v) => Some(v.to_string()),
                None => None,
            },
            var2: match data.get(3) {
                Some(v) => Some(v.to_string()),
                None => None,
            },
        };

        tokens.push(token);
    }

    // log::info!("Tokens {:#?}", tokens);
    Ok(tokens)
}

fn generate_instructions(tokens: &[Token]) -> anyhow::Result<Vec<u16>> {
    let mut instructions = Vec::new();

    for (i, t) in tokens.iter().enumerate() {
        let line = i + 1;

        let instr: u16 = match (t.mnemonic.as_str(), t.dst.as_str(), &t.var1, &t.var2) {
            ("LI", reg, Some(val_str), None) => match reg {
                "RAX" => match val_str.parse::<u16>() {
                    Ok(v) => v,
                    Err(e) => {
                        return Err(anyhow!(
                            "Row {line}, failed to parse number {}: {:?}",
                            val_str,
                            e
                        ));
                    }
                },
                _ => {
                    return Err(anyhow!("Row {line}, LI: wrong REGISTER {}", reg));
                }
            },

            (mnemonic, dest, v1, v2) => {
                let (is_jump, jump) = get_jumps(mnemonic);
                let dest = get_destination(dest, is_jump);
                let comp: u16 = match is_jump {
                    true => 0,
                    false => get_comp(mnemonic, v1, v2),
                };

                build_c_instruction(comp, dest, jump)
            }
        };

        instructions.push(instr);
    }

    for i in instructions.iter().enumerate() {
        println!("{} {:016b} {:#06x}", i.0, i.1, i.1);
    }

    Ok(instructions)
}

fn get_jumps(mnemonic: &str) -> (bool, u16) {
    match mnemonic {
        "JMP" => (true, JMP),
        "JNT" => (true, JNOT),
        "JGT" => (true, JGT),
        "JGE" => (true, JGE),
        "JEQ" => (true, JEQ),
        "JNE" => (true, JNE),
        "JLT" => (true, JLT),
        "JLE" => (true, JLE),
        _ => (false, 0),
    }
}

fn build_c_instruction(comp: u16, dest: u16, jump: u16) -> u16 {
    C_INSTRUCTION | comp | dest | jump
}

fn get_destination(dest: &str, is_jump: bool) -> u16 {
    match dest {
        "RAX" => match is_jump {
            true => A_OUT,
            false => DEST_A,
        },
        "RDX" => match is_jump {
            true => D_OUT,
            false => DEST_D,
        },
        "[RAX]" => match is_jump {
            true => A_OUT | M_ON,
            false => DEST_M,
        },
        "0" => DEST_NULL,
        _ => {
            log::error!("Unknown dest {}", dest);
            DEST_NULL
        }
    }
}

fn get_comp(mnemonic: &str, v1: &Option<String>, v2: &Option<String>) -> u16 {
    let comp = match mnemonic {
        "MOV" => match v1.as_deref() {
            Some("RAX") => A_OUT,
            Some("RDX") => D_OUT,
            Some("[RAX]") => A_OUT | M_ON,
            _ => {
                log::error!("Unknown mov src {}", mnemonic);
                0
            }
        },
        "ADD" => match (v1.as_deref(), v2.as_deref()) {
            (Some("RDX"), Some("RAX")) => D_PLUS_A,
            (Some("RDX"), Some("[RAX]")) => D_PLUS_A | M_ON,
            (Some("[RAX]"), Some("RDX")) => D_PLUS_A | M_ON,
            (Some("RDX"), Some("1")) => D_PLUS_ONE,
            (Some("RAX"), Some("1")) => A_PLUS_ONE,
            (Some("[RAX]"), Some("1")) => A_PLUS_ONE | M_ON,
            _ => {
                log::error!("Unknown ADD {}", mnemonic);
                0
            }
        },
        "SUB" => match (v1.as_deref(), v2.as_deref()) {
            (Some("RDX"), Some("RAX")) => D_MINUS_A,
            (Some("RDX"), Some("[RAX]")) => D_MINUS_A | M_ON,
            (Some("RAX"), Some("RDX")) => A_MINUS_D,
            (Some("[RAX]"), Some("RDX")) => A_MINUS_D | M_ON,
            (Some("RDX"), Some("1")) => D_MINUS_ONE,
            (Some("RAX"), Some("1")) => A_MINUS_ONE,
            (Some("[RAX]"), Some("1")) => A_MINUS_ONE | M_ON,
            _ => {
                log::error!("Unknown SUB {}", mnemonic);
                0
            }
        },
        _ => 0,
    };
    comp
}
