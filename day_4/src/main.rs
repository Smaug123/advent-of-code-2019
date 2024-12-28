use day_4::day_4;
use std::fs;

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() <= 1 {
        return Err("Required the first arg to be a path to an input file".to_string());
    }
    let path = &args[1];
    let input_str = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => return Err(format!("Error while accessing path {path} : {e}")),
    };
    let (low, high) = day_4::input(&input_str);

    println!("part 1 => {}", day_4::part_1(low, high));
    println!("part 2 => {}", day_4::part_2(low, high));
    Ok(())
}
