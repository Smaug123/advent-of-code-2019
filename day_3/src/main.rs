use day_3::day_3;
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
    let (wire1, wire2) = day_3::input(&input_str);

    println!("part 1 => {}", day_3::part_1(&wire1, &wire2));
    println!("part 2 => {}", day_3::part_2(&wire1, &wire2));
    Ok(())
}
