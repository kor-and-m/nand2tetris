use std::env;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use std::path::Path;

mod bindings;
use bindings::HackExecutor;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let file_path = Path::new(&args[1]);
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let mut instructions: Vec<i16> = Vec::new();
    for l in reader.lines() {
        let line = l?;
        let intval = u16::from_str_radix(&line, 2).unwrap();
        instructions.push(intval as i16)
    }

    let h = HackExecutor::new(&mut instructions);

    HackExecutor::run(h, 5_000);

    if args.len() > 2 {
        let v: i16 = args[2].parse().unwrap();
        HackExecutor::read_memory(h, v)
    } else {
        HackExecutor::result(h)
    };

    HackExecutor::drop(h);

    Ok(())
}
