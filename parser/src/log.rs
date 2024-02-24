use serde::Deserialize;
use serde_repr::Deserialize_repr;

#[derive(Debug, Deserialize_repr)]
#[repr(u8)]
enum Level {
    Error,
    Warning,
    Info,
    Debug,
    Verbose,
}

#[derive(Debug, Deserialize)]
pub struct Log<'str> {
    counter: usize,
    level: Level,
    file: &'str str,
    line: usize,
    message: &'str str,
}
