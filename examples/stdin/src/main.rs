use std::{io::Read, path::PathBuf};

use clap::Parser;

/// Simple program to greet a person
#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    elf: PathBuf,
}

/// Takes path to elf as argument, and parses logs whose IDs are read from stdin.
/// Example usage:
/// <path to example-stdout> | cargo run examples/stdin <path to example-stdout>
fn main() -> std::result::Result<(), String> {
    let args = Args::parse();

    let file = std::fs::File::open(args.elf).map_err(|e| e.to_string())?;
    let mmap = unsafe { memmap2::Mmap::map(&file) }.map_err(|e| e.to_string())?;

    let mut decoder = cdefmt_decoder::Decoder::new(&*mmap).map_err(|e| e.to_string())?;

    let start = std::time::Instant::now();
    let count = decoder.precache_log_metadata().map_err(|e| e.to_string())?;
    let duration = start.elapsed();

    println!("Precached {count} logs in {}[ms]", duration.as_millis());

    let mut stdin = std::io::stdin();

    let mut len = [0; std::mem::size_of::<usize>()];

    while stdin.read_exact(&mut len).is_ok() {
        let len = usize::from_ne_bytes(len);
        let mut buff = vec![0; len];

        stdin
            .read_exact(buff.as_mut_slice())
            .map_err(|e| e.to_string())?;

        let log = decoder.decode_log(&buff);

        match log {
            Ok(log) => println!("{:<7} > {}", log.get_level(), log),
            Err(e) => println!("Err: {}", e),
        }

        buff.clear();
    }

    Ok(())
}
