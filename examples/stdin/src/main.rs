use std::{io::Read, path::PathBuf};

use clap::Parser;
use gimli::Reader;

/// Simple program to greet a person
#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    elf: PathBuf,
}

fn main() {
    if let Err(e) = main_impl() {
        eprintln!("Error: {e:?}");
        std::process::exit(1);
    }
}

/// Takes path to elf as argument, and parses logs whose IDs are read from stdin.
/// Example usage:
/// <path to example-stdout> | cargo run examples/stdin <path to example-stdout>
fn main_impl() -> std::result::Result<(), anyhow::Error> {
    let args = Args::parse();

    let file = std::fs::File::open(args.elf)?;
    let mmap = unsafe { memmap2::Mmap::map(&file) }?;

    let start = std::time::Instant::now();
    let mut decoder = cdefmt_decoder::Decoder::new(&*mmap)?;
    let count = decoder.precache_log_metadata()?;
    let duration = start.elapsed();

    println!("pre-cached {count} logs in {}[ms]", duration.as_millis());

    let endianness = decoder.get_endianness();

    // stdout example writes length-value pairs
    // read the length, then use that to read the value.

    let mut stdin = std::io::stdin();
    let mut len = [0; std::mem::size_of::<u64>()];
    let mut buff = vec![0; 0];

    while stdin.read_exact(&mut len).is_ok() {
        let len = gimli::EndianSlice::new(&len, endianness).read_u64()? as usize;

        buff.resize(len, 0);
        let current_buff = &mut buff[..len];

        stdin.read_exact(current_buff)?;
        let log = decoder
            .decode_log(current_buff)
            .and_then(|l| l.to_string().map(|s| (s, l.get_level())));

        match log {
            Ok((log, level)) => println!("{:<7} > {}", format!("{:?}", level), log),
            Err(e) => println!("Error: {:?}", e),
        }
    }

    Ok(())
}
