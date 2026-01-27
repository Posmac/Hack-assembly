use core::sync;
use std::collections::HashMap;
use std::env::consts::EXE_SUFFIX;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};

use anyhow::anyhow;

use crate::utils::symbols::{
    A_MINUS_D, A_MINUS_ONE, A_OUT, A_PLUS_ONE, C_INSTRUCTION, D_MINUS_A, D_MINUS_ONE, D_OUT,
    D_PLUS_A, D_PLUS_ONE, DEST_A, DEST_D, DEST_M, DEST_NULL, JEQ, JGE, JGT, JLE, JLT, JMP, JNE,
    JNOT, M_ON, MINUS_ONE_OUT, ONE_OUT, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13,
    R14, R15, R16, ZERO_OUT,
};

pub mod utils;

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
    // process_const_section(&mut buf_reader, &mut symbols)?;
    process_data_section(&mut buf_reader, &mut symbols, &mut tokens)?;
    // log::info!("Tokens: {:#?}", tokens);
    // log::info!("Symbols: {:#?}", symbols);
    tokenize(&mut buf_reader, &mut tokens, &mut labels)?;
    let tokens = process_tokens(&mut tokens, &mut symbols, &mut labels)?;
    // log::info!("Labels: {:#?}", labels);

    // for i in instructions.iter() {
    //     log::info!("{:04X}", i);
    // }

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

        if line.contains(".data") {
            continue;
        }
        if line.contains(".program") || line.contains(".const") {
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
                tokens.push(Token {
                    mnemonic: "MOV".to_string(),
                    dst: data[0].to_string(),
                    var1: Some(value.to_string()),
                    var2: None,
                });
            }
        };
    }
    Ok(())
}

fn process_const_section(
    buf_reader: &mut io::BufReader<File>,
    symbols: &mut HashMap<String, u16>,
) -> anyhow::Result<()> {
    let filters: [char; 1] = [' '];
    for line in buf_reader.lines() {
        let line = line?;

        if line.contains(".const") {
            continue;
        }
        if line.contains(".program") || line.contains(".data") {
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

        match symbols.insert(data[0].to_string(), value) {
            Some(v) => {
                panic!("Value dublication: {} {}", line, v);
            }
            None => todo!(),
        };
    }
    Ok(())
}

fn generate_symbols_table() -> HashMap<String, u16> {
    let mut symbols: HashMap<String, u16> = HashMap::new();
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
    symbols.insert(R16.0.to_string(), R16.1);
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

        if line.contains(".data") || line.contains(".const") || line.contains(".program") {
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
    symbols: &mut HashMap<String, u16>,
    labels: &mut HashMap<String, u16>,
) -> anyhow::Result<Vec<Token>> {
    let mut expanded_tokens: Vec<Token> = Vec::with_capacity(symbols.len());
    for t in tokens {
        match t.mnemonic.as_str() {
            "LI" => {
                expanded_tokens.push(t.clone());
            }
            "MOV" => {
                let src_var = match &t.var1 {
                    Some(v) => v,
                    None => panic!("MOV is not full"),
                };
                let dest_is_var = symbols.contains_key(t.dst.as_str());
                let src_is_var = symbols.contains_key(src_var);

                let dest = symbols.get(t.dst.as_str());
                let src = symbols.get(src_var);

                match (dest_is_var, src_is_var, dest, src) {
                    //both are variables
                    (true, true, Some(d), Some(s)) => {
                        //LI RAX s
                        //MOV RDX RAX
                        //LI RAX d
                        //MOV [RAX] RDX
                        log::info!("Both are variables");
                        expanded_tokens.push(Token {
                            mnemonic: "LI".to_string(),
                            dst: "RAX".to_string(),
                            var1: Some(s.to_string()),
                            var2: None,
                        });
                        expanded_tokens.push(Token {
                            mnemonic: "MOV".to_string(),
                            dst: "RDX".to_string(),
                            var1: Some("[RAX]".to_string()),
                            var2: None,
                        });
                        expanded_tokens.push(Token {
                            mnemonic: "LI".to_string(),
                            dst: "RAX".to_string(),
                            var1: Some(d.to_string()),
                            var2: None,
                        });
                        expanded_tokens.push(Token {
                            mnemonic: "MOV".to_string(),
                            dst: "[RAX]".to_string(),
                            var1: Some("RDX".to_string()),
                            var2: None,
                        });
                    }
                    //dest is variable
                    (true, false, Some(d), None) => {
                        log::info!("Src is variable {:?}", t);
                        //MOVE temp rax, rdx, [rax], 1-...
                        let src = match &t.var1 {
                            Some(s) => s.clone(),
                            None => todo!(),
                        };

                        match src.as_str() {
                            "[RAX]" => {
                                expanded_tokens.push(Token {
                                    mnemonic: "MOV".to_string(),
                                    dst: "RDX".to_string(),
                                    var1: Some("[RAX]".to_string()),
                                    var2: None,
                                });

                                expanded_tokens.push(Token {
                                    mnemonic: "LI".to_string(),
                                    dst: "RAX".to_string(),
                                    var1: Some(d.to_string()),
                                    var2: None,
                                });

                                expanded_tokens.push(Token {
                                    mnemonic: "MOV".to_string(),
                                    dst: "[RAX]".to_string(),
                                    var1: Some("RDX".to_string()),
                                    var2: None,
                                });
                            }
                            "RDX" => {
                                expanded_tokens.push(Token {
                                    mnemonic: "LI".to_string(),
                                    dst: "RAX".to_string(),
                                    var1: Some(d.to_string()),
                                    var2: None,
                                });

                                expanded_tokens.push(Token {
                                    mnemonic: "MOV".to_string(),
                                    dst: "[RAX]".to_string(),
                                    var1: Some("RDX".to_string()),
                                    var2: None,
                                });
                            }
                            "RAX" => {
                                expanded_tokens.push(Token {
                                    mnemonic: "MOV".to_string(),
                                    dst: "RDX".to_string(),
                                    var1: Some("RAX".to_string()),
                                    var2: None,
                                });

                                expanded_tokens.push(Token {
                                    mnemonic: "LI".to_string(),
                                    dst: "RAX".to_string(),
                                    var1: Some(d.to_string()),
                                    var2: None,
                                });

                                expanded_tokens.push(Token {
                                    mnemonic: "MOV".to_string(),
                                    dst: "[RAX]".to_string(),
                                    var1: Some("RDX".to_string()),
                                    var2: None,
                                });
                            }
                            _ => {
                                expanded_tokens.push(Token {
                                    mnemonic: "LI".to_string(),
                                    dst: "RAX".to_string(),
                                    var1: t.var1.clone(),
                                    var2: None,
                                });

                                expanded_tokens.push(Token {
                                    mnemonic: "MOV".to_string(),
                                    dst: "RDX".to_string(),
                                    var1: Some("RAX".to_string()),
                                    var2: None,
                                });
                                expanded_tokens.push(Token {
                                    mnemonic: "LI".to_string(),
                                    dst: "RAX".to_string(),
                                    var1: Some(d.to_string()),
                                    var2: None,
                                });

                                expanded_tokens.push(Token {
                                    mnemonic: "MOV".to_string(),
                                    dst: "[RAX]".to_string(),
                                    var1: Some("RDX".to_string()),
                                    var2: None,
                                });
                            }
                        }
                    }
                    //src is variable
                    (false, true, None, Some(s)) => {
                        // ; MOV RAX temp
                        // ; MOV [RAX] temp
                        // ; MOV RDX temp

                        log::info!("Dest is variable {:?}", t);

                        match t.dst.as_str() {
                            "RAX" => {
                                expanded_tokens.push(Token {
                                    mnemonic: "LI".to_string(),
                                    dst: "RAX".to_string(),
                                    var1: Some(s.to_string()),
                                    var2: None,
                                });
                                expanded_tokens.push(Token {
                                    mnemonic: "MOV".to_string(),
                                    dst: "RDX".to_string(),
                                    var1: Some("[RAX]".to_string()),
                                    var2: None,
                                });
                                expanded_tokens.push(Token {
                                    mnemonic: "MOV".to_string(),
                                    dst: "RAX".to_string(),
                                    var1: Some("RDX".to_string()),
                                    var2: None,
                                });
                            }
                            "[RAX]" => {
                                // MOV RDX RAX
                                // LI RAX R0
                                // MOV [RAX] RDX
                                //
                                let reg_number = 9;
                                expanded_tokens.push(Token {
                                    mnemonic: "MOV".to_string(),
                                    dst: "RDX".to_string(),
                                    var1: Some("RAX".to_string()),
                                    var2: None,
                                });
                                expanded_tokens.push(Token {
                                    mnemonic: "LI".to_string(),
                                    dst: "RAX".to_string(),
                                    var1: Some(reg_number.to_string()),
                                    var2: None,
                                });
                                expanded_tokens.push(Token {
                                    mnemonic: "MOV".to_string(),
                                    dst: "[RAX]".to_string(),
                                    var1: Some("RDX".to_string()),
                                    var2: None,
                                });

                                // LI RAX s
                                // MOV RDX [RAX]
                                //
                                expanded_tokens.push(Token {
                                    mnemonic: "LI".to_string(),
                                    dst: "RAX".to_string(),
                                    var1: Some(s.to_string()),
                                    var2: None,
                                });
                                expanded_tokens.push(Token {
                                    mnemonic: "MOV".to_string(),
                                    dst: "RDX".to_string(),
                                    var1: Some("[RAX]".to_string()),
                                    var2: None,
                                });

                                // LI RAX R0
                                // MOV RAX [RAX]
                                // MOV [RAX] RDX
                                expanded_tokens.push(Token {
                                    mnemonic: "LI".to_string(),
                                    dst: "RAX".to_string(),
                                    var1: Some(reg_number.to_string()),
                                    var2: None,
                                });
                                expanded_tokens.push(Token {
                                    mnemonic: "MOV".to_string(),
                                    dst: "RAX".to_string(),
                                    var1: Some("[RAX]".to_string()),
                                    var2: None,
                                });
                                expanded_tokens.push(Token {
                                    mnemonic: "MOV".to_string(),
                                    dst: "[RAX]".to_string(),
                                    var1: Some("RDX".to_string()),
                                    var2: None,
                                });
                            }
                            "RDX" => {
                                expanded_tokens.push(Token {
                                    mnemonic: "LI".to_string(),
                                    dst: "RAX".to_string(),
                                    var1: Some(s.to_string()),
                                    var2: None,
                                });
                                expanded_tokens.push(Token {
                                    mnemonic: "MOV".to_string(),
                                    dst: "[RAX]".to_string(),
                                    var1: Some("RDX".to_string()),
                                    var2: None,
                                });
                            }
                            _ => {}
                        }
                    }
                    //both are registers
                    (false, false, None, None) => {
                        log::info!("Src is variable {:?}", t);
                        expanded_tokens.push(Token {
                            mnemonic: t.mnemonic.to_string(),
                            dst: t.dst.to_string(),
                            var1: t.var1.clone(),
                            var2: t.var2.clone(),
                        });
                    }
                    _ => {
                        panic!("MOV wrong");
                    }
                }
            }
            "ADD" | "SUB" => {
                let src_var_1 = match &t.var1 {
                    Some(v) => v,
                    None => panic!("ADD/SUB is not full"),
                };
                let src_var_2 = match &t.var2 {
                    Some(v) => v,
                    None => panic!("ADD/SUB is not full"),
                };
                let dest_is_var = symbols.contains_key(t.dst.as_str());
                let src_1_is_var = symbols.contains_key(src_var_1);
                let src_2_is_var = symbols.contains_key(src_var_2);

                let dest = symbols.get(t.dst.as_str());
                let src_1 = symbols.get(src_var_1);
                let src_2 = symbols.get(src_var_2);

                match (dest_is_var, src_1_is_var, src_2_is_var, dest, src_1, src_2) {
                    (true, true, true, Some(d), Some(v1), Some(v2)) => {
                        //LI RAX v2
                        //MOV RDX [RAX]
                        //LI RAX v1
                        //ADD RDX [RAX] RDX
                        //LI RAX d
                        //MOV [RAX] RDX

                        expanded_tokens.push(Token {
                            mnemonic: "LI".to_string(),
                            dst: "RAX".to_string(),
                            var1: Some(v2.to_string()),
                            var2: None,
                        });
                        expanded_tokens.push(Token {
                            mnemonic: "MOV".to_string(),
                            dst: "RDX".to_string(),
                            var1: Some("[RAX]".to_string()),
                            var2: None,
                        });
                        expanded_tokens.push(Token {
                            mnemonic: "LI".to_string(),
                            dst: "RAX".to_string(),
                            var1: Some(v1.to_string()),
                            var2: None,
                        });
                        expanded_tokens.push(Token {
                            mnemonic: t.mnemonic.to_string(),
                            dst: "RDX".to_string(),
                            var1: Some("RDX".to_string()),
                            var2: Some("[RAX]".to_string()),
                        });
                        expanded_tokens.push(Token {
                            mnemonic: "LI".to_string(),
                            dst: "RAX".to_string(),
                            var1: Some(d.to_string()),
                            var2: None,
                        });
                        expanded_tokens.push(Token {
                            mnemonic: "MOV".to_string(),
                            dst: "[RAX]".to_string(),
                            var1: Some("RDX".to_string()),
                            var2: None,
                        });
                    }
                    (true, true, false, Some(d), Some(v1), None) => {
                        //LI RAX v1
                        //ADD RDX RDX [RAX]
                        //LI RAX d
                        //MOV [RAX] RDX
                        //
                        expanded_tokens.push(Token {
                            mnemonic: "LI".to_string(),
                            dst: "RAX".to_string(),
                            var1: Some(v1.to_string()),
                            var2: None,
                        });
                        expanded_tokens.push(Token {
                            mnemonic: t.mnemonic.to_string(),
                            dst: "RDX".to_string(),
                            var1: Some("[RAX]".to_string()),
                            var2: t.var2.clone(),
                        });
                        expanded_tokens.push(Token {
                            mnemonic: "LI".to_string(),
                            dst: "RAX".to_string(),
                            var1: Some(d.to_string()),
                            var2: None,
                        });
                        expanded_tokens.push(Token {
                            mnemonic: "MOV".to_string(),
                            dst: "[RAX]".to_string(),
                            var1: Some("RDX".to_string()),
                            var2: None,
                        });
                    }
                    (true, false, true, Some(d), None, Some(v2)) => {
                        //LI RAX v2
                        //ADD RDX RDX [RAX]
                        //LI RAX d
                        //MOV [RAX] RDX

                        expanded_tokens.push(Token {
                            mnemonic: "LI".to_string(),
                            dst: "RAX".to_string(),
                            var1: Some(v2.to_string()),
                            var2: None,
                        });
                        expanded_tokens.push(Token {
                            mnemonic: t.mnemonic.to_string(),
                            dst: "RDX".to_string(),
                            var1: t.var1.clone(),
                            var2: Some("[RAX]".to_string()),
                        });
                        expanded_tokens.push(Token {
                            mnemonic: "LI".to_string(),
                            dst: "RAX".to_string(),
                            var1: Some(d.to_string()),
                            var2: None,
                        });
                        expanded_tokens.push(Token {
                            mnemonic: "MOV".to_string(),
                            dst: "[RAX]".to_string(),
                            var1: Some("RDX".to_string()),
                            var2: None,
                        });
                    }
                    (true, false, false, Some(d), None, None) => {
                        //ADD RDX RAX RDX
                        //LI RAX d
                        //MOV [RAX] RDX

                        expanded_tokens.push(Token {
                            mnemonic: t.mnemonic.to_string(),
                            dst: "RDX".to_string(),
                            var1: t.var1.clone(),
                            var2: t.var2.clone(),
                        });
                        expanded_tokens.push(Token {
                            mnemonic: "LI".to_string(),
                            dst: "RAX".to_string(),
                            var1: Some(d.to_string()),
                            var2: None,
                        });
                        expanded_tokens.push(Token {
                            mnemonic: "MOV".to_string(),
                            dst: "[RAX]".to_string(),
                            var1: Some("RDX".to_string()),
                            var2: None,
                        });
                    }
                    (false, true, true, None, Some(v1), Some(v2)) => {
                        //LI RAX v2
                        //MOV RDX [RAX]
                        //LI RAX v1
                        //Add/sub d [RAX] RDX
                        expanded_tokens.push(Token {
                            mnemonic: "LI".to_string(),
                            dst: "RAX".to_string(),
                            var1: Some(v2.to_string()),
                            var2: None,
                        });
                        expanded_tokens.push(Token {
                            mnemonic: "MOV".to_string(),
                            dst: "RDX".to_string(),
                            var1: Some("[RAX]".to_string()),
                            var2: None,
                        });
                        expanded_tokens.push(Token {
                            mnemonic: "LI".to_string(),
                            dst: "RAX".to_string(),
                            var1: Some(v1.to_string()),
                            var2: None,
                        });
                        expanded_tokens.push(Token {
                            mnemonic: t.mnemonic.to_string(),
                            dst: t.dst.to_string(),
                            var1: Some("RDX".to_string()),
                            var2: Some("[RAX]".to_string()),
                        });
                    }
                    (false, true, false, None, Some(v1), None) => {
                        //LI RAX v1
                        //MOV RDX [RAX]
                        //ADD t.dst [RAX] t.var2
                        expanded_tokens.push(Token {
                            mnemonic: "LI".to_string(),
                            dst: "RAX".to_string(),
                            var1: Some(v1.to_string()),
                            var2: None,
                        });
                        expanded_tokens.push(Token {
                            mnemonic: "MOV".to_string(),
                            dst: "RDX".to_string(),
                            var1: Some("[RAX]".to_string()),
                            var2: None,
                        });
                        expanded_tokens.push(Token {
                            mnemonic: t.mnemonic.to_string(),
                            dst: t.dst.to_string(),
                            var1: Some("RDX".to_string()),
                            var2: t.var2.clone(),
                        });
                    }
                    (false, false, true, None, None, Some(v2)) => {
                        //LI RAX v2
                        //MOV RDX [RAX]
                        //ADD t.dst t.var1 [RAX]
                        expanded_tokens.push(Token {
                            mnemonic: "LI".to_string(),
                            dst: "RAX".to_string(),
                            var1: Some(v2.to_string()),
                            var2: None,
                        });
                        expanded_tokens.push(Token {
                            mnemonic: "MOV".to_string(),
                            dst: "RDX".to_string(),
                            var1: Some("[RAX]".to_string()),
                            var2: None,
                        });
                        expanded_tokens.push(Token {
                            mnemonic: t.mnemonic.to_string(),
                            dst: t.dst.to_string(),
                            var1: t.var1.clone(),
                            var2: Some("RDX".to_string()),
                        });
                    }
                    (false, false, false, None, None, None) => {
                        expanded_tokens.push(Token {
                            mnemonic: t.mnemonic.to_string(),
                            dst: t.dst.to_string(),
                            var1: t.var1.clone(),
                            var2: t.var2.clone(),
                        });
                    }
                    _ => {
                        panic!("ADD/SUB wrong params");
                    }
                }
            }

            "JMP" | "JNT" | "JGT" | "JGE" | "JEQ" | "JNE" | "JLT" | "JLE" => {
                let dest_is_var = symbols.contains_key(t.dst.as_str());
                let dest_var = symbols.get(t.dst.as_str());

                let dest_is_label = labels.contains_key(t.dst.as_str());
                let dest_label = labels.get(t.dst.as_str());

                match (dest_is_var, dest_var) {
                    (true, Some(d)) => {
                        expanded_tokens.push(Token {
                            mnemonic: "LI".to_string(),
                            dst: "RAX".to_string(),
                            var1: Some(d.to_string()),
                            var2: None,
                        });
                        expanded_tokens.push(Token {
                            mnemonic: t.mnemonic.to_string(),
                            dst: "[RAX]".to_string(),
                            var1: None,
                            var2: None,
                        });
                    }
                    (false, None) => {
                        expanded_tokens.push(Token {
                            mnemonic: t.mnemonic.to_string(),
                            dst: t.dst.to_string(),
                            var1: t.var1.clone(),
                            var2: t.var2.clone(),
                        });
                    }
                    _ => match (dest_is_label, dest_label) {
                        (true, Some(d)) => {
                            expanded_tokens.push(Token {
                                mnemonic: "LI".to_string(),
                                dst: "RAX".to_string(),
                                var1: Some(d.to_string()),
                                var2: None,
                            });
                        }
                        (false, None) => {
                            expanded_tokens.push(Token {
                                mnemonic: t.mnemonic.to_string(),
                                dst: t.dst.to_string(),
                                var1: t.var1.clone(),
                                var2: t.var2.clone(),
                            });
                        }
                        _ => panic!("Var is not a symbol or a label!"),
                    },
                }
            }
            _ => {}
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
            log::error!("Unknown dest {dest}");
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
                log::error!("Unknown mov src {:?}", v1);
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
