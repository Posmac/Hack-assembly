use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};

use anyhow::anyhow;

use crate::utils::symbols::{
    _RAX_, A_MINUS_D, A_MINUS_ONE, A_OUT, A_PLUS_ONE, C_INSTRUCTION, D_MINUS_A, D_MINUS_ONE, D_OUT,
    D_PLUS_A, D_PLUS_ONE, DATA_SECTION, DEST_A, DEST_D, DEST_M, DEST_NULL, JEQ, JGE, JGT, JLE, JLT,
    JMP, JNE, JNOT, M_ON, MINUS_ONE_OUT, ONE_OUT, PROGRAM_SECTION, R0, R1, R2, R3, R4, R5, R6, R7,
    R8, R9, R10, R11, R12, R13, R14, R15, RAX, RDX, ZERO_OUT,
};

pub mod utils;

enum Operand<'a> {
    Var(u16),
    Reg(&'a str),
    Label(u16),
    Const(&'a str),
}

fn op<'a>(
    s: &'a str,
    symbols: &HashMap<String, u16>,
    labels: &HashMap<String, u16>,
) -> Operand<'a> {
    if symbols.contains_key(s) {
        Operand::Var(*symbols.get(s).unwrap())
    } else if labels.contains_key(s) {
        Operand::Label(*labels.get(s).unwrap())
    } else if s.contains(RAX) || s.contains(RDX) {
        Operand::Reg(s)
    } else {
        Operand::Const(s)
    }
}

#[derive(Debug, Clone)]
struct Token {
    pub mnemonic: String,
    pub dst: String,
    pub var1: Option<String>,
    pub var2: Option<String>,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let mut labels: HashMap<String, u16> = HashMap::new();
    let mut symbols = generate_symbols_table();
    let mut tokens: Vec<Token> = vec![];
    let mut instructions: Vec<u16> = vec![];

    let mut buf_reader = open_file("asm/add.asm")?;
    process_data_section(&mut buf_reader, &mut symbols, &mut tokens)?;
    tokenize(&mut buf_reader, &mut tokens, &mut labels)?;
    let tokens = process_tokens(&mut tokens, &mut symbols, &mut labels)?;
    generate_instructions(&tokens, &mut instructions)?;
    for i in instructions.iter().enumerate() {
        println!("{} {:016b} {:#06x}", i.0, i.1, i.1);
    }
    save_file(&instructions, "bin/add.bin")?;
    Ok(())
}

pub fn open_file(file_path: &str) -> anyhow::Result<BufReader<File>> {
    let file = File::open(file_path)?;
    Ok(io::BufReader::new(file))
}

fn process_data_section(
    buf_reader: &mut io::BufReader<File>,
    symbols: &mut HashMap<String, u16>,
    tokens: &mut Vec<Token>,
) -> anyhow::Result<()> {
    let filters: [char; 1] = [' '];
    for line in buf_reader.lines() {
        let line = line?;

        if line.contains(DATA_SECTION) {
            continue;
        }
        if line.contains(PROGRAM_SECTION) {
            break;
        }

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

        log::info!("Data section line: {line}, data {:?}", data);

        let value = match data[2].parse::<u16>() {
            Ok(v) => v,
            Err(e) => {
                return Err(anyhow!(
                    "Failed to parse value of .data: {line}, value {} {}",
                    data[2],
                    e
                ));
            }
        };

        let len = symbols.len() as u16;
        match symbols.insert(data[0].to_string(), len) {
            Some(v) => {
                panic!("Value dublication: {} {}", line, v);
            }
            None => {
                tokens.push(mov(data[0], value.to_string().as_str()));
            }
        };
    }
    Ok(())
}

fn generate_symbols_table() -> HashMap<String, u16> {
    let mut symbols: HashMap<String, u16> = HashMap::new();
    symbols.insert(R0.0.to_string(), R0.1);
    symbols.insert(R1.0.to_string(), R1.1);
    symbols.insert(R2.0.to_string(), R2.1);
    symbols.insert(R3.0.to_string(), R3.1);
    symbols.insert(R4.0.to_string(), R4.1);
    symbols.insert(R5.0.to_string(), R5.1);
    symbols.insert(R6.0.to_string(), R6.1);
    symbols.insert(R7.0.to_string(), R7.1);
    symbols.insert(R8.0.to_string(), R8.1);
    symbols.insert(R9.0.to_string(), R9.1);
    symbols.insert(R10.0.to_string(), R10.1);
    symbols.insert(R11.0.to_string(), R11.1);
    symbols.insert(R12.0.to_string(), R12.1);
    symbols.insert(R13.0.to_string(), R13.1);
    symbols.insert(R14.0.to_string(), R14.1);
    symbols.insert(R15.0.to_string(), R15.1);
    symbols
}

fn save_file(instructions: &Vec<u16>, path: &str) -> anyhow::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    for instr in instructions {
        writeln!(writer, "{:04X}", instr)?;
    }

    writer.flush()?;
    Ok(())
}

fn tokenize(
    buf_reader: &mut io::BufReader<File>,
    tokens: &mut Vec<Token>,
    labels: &mut HashMap<String, u16>,
) -> anyhow::Result<()> {
    let filters: [char; 1] = [' '];

    for line in buf_reader.lines() {
        let line = line?;

        if line.contains(DATA_SECTION) || line.contains(PROGRAM_SECTION) {
            continue;
        }

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

        if data[0].contains('@') {
            match labels.insert(data[0].to_string(), tokens.len() as u16) {
                Some(v) => return Err(anyhow!("Label {:?} already exists {}", data, v)),
                None => {}
            };
            continue;
        }

        let mnemonic = match data.get(0) {
            Some(v) => v.to_string(),
            None => panic!(),
        };
        let dst = match data.get(1) {
            Some(v) => v.to_string(),
            None => panic!(),
        };
        let var1 = match data.get(2) {
            Some(v) => Some(v.to_string()),
            None => None,
        };
        let var2 = match data.get(3) {
            Some(v) => Some(v.to_string()),
            None => None,
        };

        let token = Token {
            mnemonic,
            dst,
            var1,
            var2,
        };

        tokens.push(token);
    }

    Ok(())
}

fn process_tokens(
    tokens: &mut Vec<Token>,
    symbols: &HashMap<String, u16>,
    labels: &HashMap<String, u16>,
) -> anyhow::Result<Vec<Token>> {
    let mut expanded_tokens: Vec<Token> = Vec::with_capacity(symbols.len());
    for t in tokens {
        match t.mnemonic.as_str() {
            "LI" => {
                expanded_tokens.push(t.clone());
            }
            "MOV" => {
                expand_mov(t, symbols, labels, &mut expanded_tokens);
            }
            "ADD" | "SUB" => {
                expand_comp(t, symbols, labels, &mut expanded_tokens);
            }

            "JMP" | "JNT" | "JGT" | "JGE" | "JEQ" | "JNE" | "JLT" | "JLE" => {
                expand_jump(t, symbols, labels, &mut expanded_tokens);
            }
            _ => {
                unreachable!()
            }
        }
    }

    log::info!("Expanded: {:#?}", expanded_tokens);

    Ok(expanded_tokens)
}

fn generate_instructions(tokens: &Vec<Token>, instructions: &mut Vec<u16>) -> anyhow::Result<()> {
    for (i, t) in tokens.iter().enumerate() {
        let line = i + 1;

        let instr = match (t.mnemonic.as_str(), t.dst.as_str(), &t.var1, &t.var2) {
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
                let comp: u16 = match is_jump {
                    true => 0,
                    false => get_comp(mnemonic, v1, v2),
                };
                let dest = get_destination(dest, is_jump);

                build_c_instruction(comp, dest, jump)
            }
        };
        instructions.push(instr);
    }

    Ok(())
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
        RAX => match is_jump {
            true => A_OUT,
            false => DEST_A,
        },
        RDX => match is_jump {
            true => D_OUT,
            false => DEST_D,
        },
        _RAX_ => match is_jump {
            true => A_OUT | M_ON,
            false => DEST_M,
        },
        "0" => DEST_NULL,
        _ => {
            log::error!("Unknown dest {dest}");
            DEST_NULL
        }
    }
}

fn get_comp(mnemonic: &str, v1: &Option<String>, v2: &Option<String>) -> u16 {
    let comp = match mnemonic {
        "MOV" => match v1.as_deref() {
            Some(RAX) => A_OUT,
            Some(RDX) => D_OUT,
            Some(_RAX_) => A_OUT | M_ON,
            _ => {
                log::error!("Unknown mov src {:?}", v1);
                0
            }
        },
        "ADD" => match (v1.as_deref(), v2.as_deref()) {
            (Some(RDX), Some(RAX)) => D_PLUS_A,
            (Some(RDX), Some(_RAX_)) => D_PLUS_A | M_ON,
            (Some(_RAX_), Some(RDX)) => D_PLUS_A | M_ON,
            (Some(RDX), Some("1")) => D_PLUS_ONE,
            (Some(RAX), Some("1")) => A_PLUS_ONE,
            (Some(_RAX_), Some("1")) => A_PLUS_ONE | M_ON,
            _ => {
                log::error!("Unknown ADD {}", mnemonic);
                0
            }
        },
        "SUB" => match (v1.as_deref(), v2.as_deref()) {
            (Some(RDX), Some(RAX)) => D_MINUS_A,
            (Some(RDX), Some(_RAX_)) => D_MINUS_A | M_ON,
            (Some(RAX), Some(RDX)) => A_MINUS_D,
            (Some(_RAX_), Some(RDX)) => A_MINUS_D | M_ON,
            (Some(RDX), Some("1")) => D_MINUS_ONE,
            (Some(RAX), Some("1")) => A_MINUS_ONE,
            (Some(_RAX_), Some("1")) => A_MINUS_ONE | M_ON,
            _ => {
                log::error!("Unknown SUB {}", mnemonic);
                0
            }
        },
        _ => 0,
    };
    comp
}

fn li(v: impl ToString) -> Token {
    Token {
        mnemonic: "LI".into(),
        dst: RAX.into(),
        var1: Some(v.to_string()),
        var2: None,
    }
}

fn mov(dst: &str, src: &str) -> Token {
    Token {
        mnemonic: "MOV".into(),
        dst: dst.into(),
        var1: Some(src.into()),
        var2: None,
    }
}

fn comp(mnemonic: &str, dst: &str, src_1: &str, src_2: &str) -> Token {
    Token {
        mnemonic: mnemonic.into(),
        dst: dst.into(),
        var1: Some(src_1.into()),
        var2: Some(src_2.into()),
    }
}

fn expand_comp(
    t: &Token,
    symbols: &HashMap<String, u16>,
    labels: &HashMap<String, u16>,
    out: &mut Vec<Token>,
) {
    log::info!("TOKEN: {:#?}, SYMBOLS: {:#?}", t, symbols);
    let src_1 = t.var1.as_ref().expect("COMP is not full!");
    let src_2 = t.var2.as_ref().expect("COMP is not full!");

    match (
        op(&t.dst, symbols, labels),
        op(&src_1, symbols, labels),
        op(&src_2, symbols, labels),
    ) {
        // ; ADD temp temp_1 temp_2
        (Operand::Var(d), Operand::Var(v1), Operand::Var(v2)) => {
            out.extend([
                li(v1),
                mov(RDX, _RAX_),
                li(v2),
                comp(t.mnemonic.as_str(), RDX, RDX, _RAX_),
                li(d),
                mov(_RAX_, RDX),
            ]);
        }
        // ; ADD temp RAX/RDX/[RAX] temp_2
        (Operand::Var(d), Operand::Reg(r), Operand::Var(v)) => match r {
            RAX => {
                out.extend([
                    mov(RAX, RDX),
                    li(v),
                    comp(t.mnemonic.as_str(), RDX, RDX, _RAX_),
                    li(d),
                    mov(_RAX_, RDX),
                ]);
            }
            RDX => {
                out.extend([
                    li(v),
                    comp(t.mnemonic.as_str(), RDX, RDX, _RAX_),
                    li(d),
                    mov(_RAX_, RDX),
                ]);
            }
            _ => unreachable!(),
        },
        // ; ADD temp temp_1 RAX/RDX/[RAX]
        (Operand::Var(d), Operand::Var(v), Operand::Reg(r)) => match r {
            RAX => {
                out.extend([
                    mov(RAX, RDX),
                    li(v),
                    comp(t.mnemonic.as_str(), RDX, _RAX_, RDX),
                    li(d),
                    mov(_RAX_, RDX),
                ]);
            }
            RDX => {
                out.extend([
                    li(v),
                    comp(t.mnemonic.as_str(), RDX, _RAX_, RDX),
                    li(d),
                    mov(_RAX_, RDX),
                ]);
            }
            _ => unreachable!(),
        },
        // ; ADD temp temp_1 CONST
        (Operand::Var(d), Operand::Var(v), Operand::Const(c)) => {
            out.extend([
                li(c),
                mov(RDX, RAX),
                li(v),
                comp(t.mnemonic.as_str(), RDX, _RAX_, RDX),
                li(d),
                mov(_RAX_, RDX),
            ]);
        }
        // ; ADD temp CONST temp_2
        (Operand::Var(d), Operand::Const(c), Operand::Var(v)) => {
            out.extend([
                li(c),
                mov(RDX, RAX),
                li(v),
                comp(t.mnemonic.as_str(), RDX, RDX, _RAX_),
                li(d),
                mov(_RAX_, RDX),
            ]);
        }

        // ; ADD temp RAX/RDX/[RAX] RAX/RDX/[RAX]
        (Operand::Var(v), Operand::Reg(r1), Operand::Reg(r2)) => {
            log::warn!("Simple register operations already supported!");
        }
        // ; ADD temp RAX/RDX/[RAX] CONST
        (Operand::Var(v), Operand::Reg(r), Operand::Const(c)) => match r {
            RDX => {
                out.extend([
                    li(c),
                    comp(t.mnemonic.as_str(), RDX, RDX, RAX),
                    li(v),
                    mov(_RAX_, RDX),
                ]);
            }
            RAX => {
                out.extend([
                    mov(RDX, RAX),
                    li(c),
                    comp(t.mnemonic.as_str(), RDX, RDX, RAX),
                    li(v),
                    mov(_RAX_, RDX),
                ]);
            }
            _ => unreachable!(),
        },

        // ; ADD temp CONST RAXRDX/[RAX]
        (Operand::Var(v), Operand::Const(c), Operand::Reg(r)) => match r {
            RDX => {
                out.extend([
                    li(c),
                    comp(t.mnemonic.as_str(), RDX, RAX, RDX),
                    li(v),
                    mov(_RAX_, RDX),
                ]);
            }
            RAX => {
                out.extend([
                    mov(RDX, RAX),
                    li(c),
                    comp(t.mnemonic.as_str(), RDX, RAX, RDX),
                    li(v),
                    mov(_RAX_, RDX),
                ]);
            }
            _ => unreachable!(),
        },

        // ; ADD RAX/RDX/[RAX] temp_1 temp_2
        (Operand::Reg(d), Operand::Var(v1), Operand::Var(v2)) => {
            out.extend([
                li(v1),
                mov(_RAX_, RDX),
                li(v2),
                comp(t.mnemonic.as_str(), RDX, _RAX_, RDX),
            ]);

            match d {
                RAX => {
                    out.push(mov(RDX, RAX));
                }
                RDX => {
                    //already stays in RDX
                }
                _ => unreachable!(),
            }
        }
        // ; ADD RAX/RDX/[RAX] RAX/RDX/[RAX] temp_2
        (Operand::Reg(d), Operand::Reg(r), Operand::Var(v)) => {
            match r {
                RAX => {
                    out.extend([
                        mov(RAX, RDX),
                        li(v),
                        comp(t.mnemonic.as_str(), RDX, RDX, _RAX_),
                    ]);
                }
                RDX => {
                    out.extend([li(v), comp(t.mnemonic.as_str(), RDX, RDX, _RAX_)]);
                }
                _ => unreachable!(),
            };
            match d {
                RAX => {
                    out.extend([mov(RDX, RAX)]);
                }
                RDX => {}
                _ => unreachable!(),
            };
        }
        // ; ADD  RAX/RDX/[RAX] temp_1 RAX/RDX/[RAX]
        (Operand::Reg(d), Operand::Var(v), Operand::Reg(r)) => {
            match r {
                RAX => {
                    out.extend([
                        mov(RAX, RDX),
                        li(v),
                        comp(t.mnemonic.as_str(), RDX, _RAX_, RDX),
                    ]);
                }
                RDX => {
                    out.extend([li(v), comp(t.mnemonic.as_str(), RDX, _RAX_, RDX)]);
                }
                _ => unreachable!(),
            };
            match d {
                RAX => {
                    out.extend([mov(RDX, RAX)]);
                }
                RDX => {}
                _ => unreachable!(),
            };
        }
        // ; ADD RAX/RDX/[RAX] temp_1 CONST
        (Operand::Reg(d), Operand::Var(v), Operand::Const(c)) => match d {
            RAX => {
                out.extend([
                    li(c),
                    mov(RDX, RAX),
                    li(v),
                    comp(t.mnemonic.as_str(), RAX, _RAX_, RDX),
                ]);
            }
            RDX => {
                out.extend([
                    li(c),
                    mov(RDX, RAX),
                    li(v),
                    comp(t.mnemonic.as_str(), RDX, _RAX_, RDX),
                ]);
            }
            _ => unreachable!(),
        },
        // ; ADD RAX/RDX/[RAX] CONST temp_2
        (Operand::Reg(d), Operand::Const(c), Operand::Var(v)) => match d {
            RAX => {
                out.extend([
                    li(c),
                    mov(RDX, RAX),
                    li(v),
                    comp(t.mnemonic.as_str(), RAX, RDX, _RAX_),
                ]);
            }
            RDX => {
                out.extend([
                    li(c),
                    mov(RDX, RAX),
                    li(v),
                    comp(t.mnemonic.as_str(), RDX, RDX, _RAX_),
                ]);
            }
            _ => unreachable!(),
        },
        // ; ADD RAX/RDX/[RAX] RAX/RDX/[RAX] RAX/RDX/[RAX]
        (Operand::Reg(v), Operand::Reg(r1), Operand::Reg(r2)) => {
            log::info!("Already supported");
        }
        // ; ADD RAX/RDX/[RAX] RAX/RDX/[RAX] CONST
        (Operand::Reg(v), Operand::Reg(r), Operand::Const(c)) => {
            match r {
                RAX => {
                    out.extend([
                        mov(RDX, RAX),
                        li(c),
                        comp(t.mnemonic.as_str(), RDX, RDX, RAX),
                    ]);
                }
                RDX => {
                    out.extend([li(c), comp(t.mnemonic.as_str(), RDX, RDX, RAX)]);
                }
                _ => unreachable!(),
            };
            match v {
                RAX => {
                    out.push(mov(RDX, RAX));
                }
                RDX => {}
                _ => unreachable!(),
            };
        }
        // ; ADD RAX/RDX/[RAX] CONST RAXRDX/[RAX]
        (Operand::Reg(v), Operand::Const(c), Operand::Reg(r)) => {
            match r {
                RAX => {
                    out.extend([
                        mov(RDX, RAX),
                        li(c),
                        comp(t.mnemonic.as_str(), RDX, RAX, RDX),
                    ]);
                }
                RDX => {
                    out.extend([li(c), comp(t.mnemonic.as_str(), RDX, RAX, RDX)]);
                }
                _ => unreachable!(),
            };
            match v {
                RAX => {
                    out.push(mov(RDX, RAX));
                }
                RDX => {}
                _ => unreachable!(),
            };
        }
        _ => unreachable!(),
    }
}

fn expand_mov(
    t: &Token,
    symbols: &HashMap<String, u16>,
    labels: &HashMap<String, u16>,
    out: &mut Vec<Token>,
) {
    let src = t.var1.as_ref().expect("MOV not full");

    match (op(&t.dst, symbols, labels), op(src, symbols, labels)) {
        // ========== mem <- mem ==========
        (Operand::Var(d), Operand::Var(s)) => {
            out.extend([li(s), mov(RDX, _RAX_), li(d), mov(_RAX_, RDX)]);
        }

        // ========== mem <- reg ==========
        (Operand::Var(d), Operand::Reg(s)) => match s {
            RAX => out.extend([mov(RDX, RAX), li(d), mov(_RAX_, RDX)]),
            RDX => out.extend([li(d), mov(_RAX_, RDX)]),
            _ => unreachable!(),
        },

        // ========== mem <- const ==========
        (Operand::Var(d), Operand::Const(v)) => {
            out.extend([li(v), mov(RDX, RAX), li(d), mov(_RAX_, RDX)]);
        }

        // ========== reg <- mem ==========
        (Operand::Reg(d), Operand::Var(s)) => match d {
            RAX => out.extend([li(s), mov(RAX, _RAX_)]),
            RDX => out.extend([li(s), mov(RDX, _RAX_)]),
            // _RAX_ => out.extend([
            //     mov(RDX, RAX),
            //     li(symbols.get(R9.0).unwrap()),
            //     mov(_RAX_, RDX),
            //     li(d),
            //     mov(RDX, _RAX_),
            //     li(symbols.get(R9.0).unwrap()),
            //     mov(RAX, _RAX_),
            //     mov(_RAX_, RDX),
            // ]),
            _ => unreachable!(),
        },

        // ========== reg <- const ==========
        (Operand::Reg(d), Operand::Const(v)) => match d {
            RDX => out.extend([li(v), mov(d, RAX)]),
            RAX => out.push(li(v)),
            _ => unreachable!(),
        },

        // ========== reg <- reg ==========
        (Operand::Reg(_), Operand::Reg(_)) => {
            out.push(t.clone());
        }

        _ => panic!("Unsupported MOV: {:?}", t),
    }
}

fn expand_jump(
    t: &mut Token,
    symbols: &HashMap<String, u16>,
    labels: &HashMap<String, u16>,
    out: &mut Vec<Token>,
) {
    // JGT R1 @loop_end
    // JMP 0 @loop_start

    //JMP R0-16/temp @label
    //JMP 0 @label
    //JMP RAX/RDX @label
    //
    // LI RAX 0 ;20          ;0000 0000 0000 0000 0000
    // MOV RDX [RAX] ;21     ;1001 110000 010 000 9c10
    // LI RAX 1 ;22          ;0000 0000 0000 0001 0001
    // SUB RDX RDX [RAX] ;23 ;1001 010011 010 000 94d0
    // LI RAX 38 ;24         ;0000 0000 0010 0110 0026
    // JLT RDX ;25           ;1000 001100 000 100 8304
    let src = t.var1.as_ref().expect("JMP not full");
    match (op(&t.dst, symbols, labels), op(&src, symbols, labels)) {
        (Operand::Var(v), Operand::Label(l)) => {
            out.extend([
                li(v),
                mov(RDX, _RAX_),
                li(l),
                comp(t.mnemonic.as_str(), RDX, "", ""),
            ]);
        }
        (Operand::Const(c), Operand::Label(l)) => {
            out.extend([
                li(c),
                mov(RDX, RAX),
                li(l),
                comp(t.mnemonic.as_str(), RDX, "", ""),
            ]);
        }
        (Operand::Reg(r), Operand::Label(l)) => match r {
            RAX => {
                out.extend([mov(RDX, RAX), li(l), comp(t.mnemonic.as_str(), RDX, "", "")]);
            }
            RDX => {
                out.extend([li(l), comp(t.mnemonic.as_str(), RDX, "", "")]);
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    };
}
