use cdefmt_parser::log::LogParser;

/// Takes path to elf as argument, and parses logs whose IDs are read from stdin.
/// Example usage:
/// <path to example-stdout> | cargo run examples/stdin <path to example-stdout>
fn main() -> std::result::Result<(), String> {
    let path = std::env::args().nth(2).ok_or("Y U NO GIVE PATH?")?;
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mmap = unsafe { memmap2::Mmap::map(&file) }.map_err(|e| e.to_string())?;

    let mut logger = LogParser::new(&*mmap).map_err(|e| e.to_string())?;

    let stdin = std::io::stdin();
    let mut buff = Default::default();

    while let Ok(s) = stdin.read_line(&mut buff) {
        if s == 0 {
            break;
        }

        let log_id = buff.trim().parse::<usize>().map_err(|e| e.to_string())?;

        let log = logger.get_log(log_id).map_err(|e| e.to_string())?;
        println!("[{:#010x}] {:?}", log_id, log);

        buff.clear();
    }

    let file = object::File::parse(&*mmap).unwrap();

    cdefmt_parser::dwarf::dump_file(&file).unwrap();

    Ok(())
}
