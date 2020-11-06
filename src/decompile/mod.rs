use crate::Binary;
use crate::inst::instruction::{InstSet, ArgumentType, RuntimeSignature};
use crate::inst::register::Register;

pub fn decompile(program: &Binary, iset: &InstSet) -> String {
    let mut text = String::new();
    
    let mut text_addr = crate::TEXT_BOT;

    for &word in &program.text {
        for (label, &addr) in program.labels.iter() {
            if addr == text_addr {
                text.push_str(&format!("\n{}: \n", label));
            }
        }

        let opcode = word >> 26;
        let rs =    (word >> 21) & 0x1F;
        let rt =    (word >> 16) & 0x1F;
        let rd =    (word >> 11) & 0x1F;
        let shamt = (word >> 6) & 0x1F;
        let funct =  word & 0x3F;
        let imm =    (word & 0xFFFF) as i16;
        let addr =   word & 0x3FFFFFF;
        
        let mut inst = None;

        for native_inst in &iset.native_set {
            match &native_inst.runtime {
                RuntimeSignature::R { funct: inst_funct } => {
                    if opcode != 0 || *inst_funct as u32 != funct {
                        continue;
                    }
                }

                RuntimeSignature::I { opcode: inst_opcode, rt: inst_rt } => {
                    if *inst_opcode as u32 != opcode || inst_rt.is_some() && inst_rt.unwrap() as u32 != rt {
                        continue;
                    }
                }

                RuntimeSignature::J { opcode: inst_opcode, .. } => {
                    if *inst_opcode as u32 != opcode {
                        continue;
                    }
                }
            }

            inst = Some(native_inst);
            break;
        }

        text.push_str(&format!("0x{:08x} [0x{:08x}]    ", text_addr, word));

        if let Some(inst) = inst {

            if inst.name == "sll" && rd == 0 && rt == 0 && shamt == 0 {
                text.push_str("nop");
            } else {
                text.push_str(&format!("{:6} ", inst.name));

                text.push_str(
                    &inst.compile.format.iter()
                            .map(|arg| match arg {
                                ArgumentType::Rd => format!("${}", Register::u32_to_str(rd)),
                                ArgumentType::Rt => format!("${}", Register::u32_to_str(rt)),
                                ArgumentType::Rs => format!("${}", Register::u32_to_str(rs)),
                                ArgumentType::Shamt  => format!("{}", shamt),
                                ArgumentType::Imm    => format!("{}", imm),
                                ArgumentType::OffRs => format!("{}(${})", imm, Register::u32_to_str(rs)),
                                ArgumentType::OffRt => format!("{}(${})", imm, Register::u32_to_str(rt)),
                                ArgumentType::J      => {
                                    let j_addr = (text_addr + 4) & 0xF000_0000 | addr << 2;
                                    let mut j_label = None;
                                    for (label, &label_addr) in program.labels.iter() {
                                        if label_addr == j_addr {
                                            j_label = Some(label);
                                            break;
                                        }
                                    }

                                    j_label.cloned().unwrap_or(format!("{:08x}", j_addr))
                                }
                                _ => unreachable!(),
                            })
                            .collect::<Vec<String>>()
                            .join(", ")
                            .to_ascii_lowercase()
                );
            }
        } else {
            text.push_str(&format!("# Unknown instruction: opcode={} funct={}", opcode, funct));
        }

        text.push('\n');
        text_addr += 4;
    }

    text
}
