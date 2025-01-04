use day_13::day_13;
use intcode::intcode::MachineExecutionError;
use std::fs;

enum Error {
    Basic(String),
    Eval(MachineExecutionError),
}

impl From<MachineExecutionError> for Error {
    fn from(value: MachineExecutionError) -> Self {
        Error::Eval(value)
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Basic(arg0) => f.debug_tuple("Basic").field(arg0).finish(),
            Self::Eval(arg0) => f.debug_tuple("Eval").field(arg0).finish(),
        }
    }
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() <= 1 {
        return Err(Error::Basic(
            "Required the first arg to be a path to an input file".to_string(),
        ));
    }
    let path = &args[1];
    let input_str = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            return Err(Error::Basic(format!(
                "Error while accessing path {path} : {e}"
            )))
        }
    };
    let input = day_13::input(&input_str);

    println!("part 1 => {}", day_13::part_1(&input)?);
    println!("part 2 => {}", day_13::part_2(&input)?);
    Ok(())
}
