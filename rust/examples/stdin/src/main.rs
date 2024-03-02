use cdefmt_parser::Parser;

/// Takes path to elf as argument, and parses logs whose IDs are read from stdin.
/// Example usage:
/// <path to example-stdout> | cargo run examples/stdin <path to example-stdout>
fn main() -> std::result::Result<(), String> {
    let path = std::env::args().nth(2).ok_or("Y U NO GIVE PATH?")?;
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mmap = unsafe { memmap2::Mmap::map(&file) }.map_err(|e| e.to_string())?;

    let mut logger = Parser::new(&*mmap).map_err(|e| e.to_string())?;

    let stdin = std::io::stdin();
    let mut buff = Default::default();

    while let Ok(s) = stdin.read_line(&mut buff) {
        if s == 0 {
            break;
        }

        let parsed_buff = buff
            .trim()
            .split(';')
            .map(|b| u8::from_str_radix(b, 16).unwrap())
            .collect::<Vec<_>>();
        let log = logger.parse_log(&parsed_buff);
        match log {
            Ok(log) => println!("{:<7} > {}", log.get_level(), log),
            Err(e) => println!("Err: {}", e),
        }

        buff.clear()
    }
    Ok(())
}
