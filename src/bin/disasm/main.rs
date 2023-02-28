use std::env;
use std::fs;

use anyhow::{bail, Result};

use gbrust::gameboy::cpu::instruction::Instruction;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        bail!("Syntax: {} <filename>", args[0]);
    }

    let f = fs::read(&args[1])?;
    let mut pos = 0;
    while pos < f.len() {
        let ins = Instruction::decode(&f[pos..])?;
        println!("{:04X} {}", pos, ins);
        pos += ins.len;
    }
    Ok(())
}
