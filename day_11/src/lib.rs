pub mod day_11 {
    use std::collections::HashMap;

    use intcode::intcode::{MachineExecutionError, MachineState};

    #[derive(Copy, Clone, Debug)]
    enum Direction {
        Up,
        Down,
        Left,
        Right,
    }

    impl Direction {
        pub fn rotate_clockwise(d: Direction) -> Direction {
            match d {
                Direction::Down => Direction::Left,
                Direction::Left => Direction::Up,
                Direction::Up => Direction::Right,
                Direction::Right => Direction::Down,
            }
        }

        pub fn rotate_anticlockwise(d: Direction) -> Direction {
            match d {
                Direction::Down => Direction::Right,
                Direction::Right => Direction::Up,
                Direction::Up => Direction::Left,
                Direction::Left => Direction::Down,
            }
        }
    }

    pub fn input(s: &str) -> Vec<i64> {
        s.trim()
            .split(',')
            .map(|l| str::parse(l).unwrap())
            .collect()
    }

    fn run_machine(
        mut machine: MachineState<i64>,
        locations: &mut HashMap<(i32, i32), bool>,
    ) -> Result<(), MachineExecutionError> {
        let mut current_x = 0;
        let mut current_y = 0;
        let mut direction = Direction::Up;
        loop {
            match machine.execute_until_input()? {
                intcode::intcode::StepIoResult::Terminated => {
                    break;
                }
                intcode::intcode::StepIoResult::Output(v) => {
                    assert!(v == 0 || v == 1);
                    locations.insert((current_x, current_y), v == 1);
                    match machine.execute_until_input()? {
                        intcode::intcode::StepIoResult::Terminated => {
                            panic!("unexpectedly terminated");
                        }
                        intcode::intcode::StepIoResult::AwaitingInput(_) => {
                            panic!("unexpectedly asked for input");
                        }
                        intcode::intcode::StepIoResult::Output(v) => {
                            match v {
                                0 => {
                                    direction = Direction::rotate_anticlockwise(direction);
                                }
                                1 => {
                                    direction = Direction::rotate_clockwise(direction);
                                }
                                _ => {
                                    panic!("Unexpected direction output: {v}");
                                }
                            }
                            match direction {
                                Direction::Up => {
                                    current_y += 1;
                                }
                                Direction::Down => {
                                    current_y -= 1;
                                }
                                Direction::Left => {
                                    current_x -= 1;
                                }
                                Direction::Right => {
                                    current_x += 1;
                                }
                            }
                        }
                    }
                }
                intcode::intcode::StepIoResult::AwaitingInput(loc) => {
                    machine.set_mem_elt(
                        loc,
                        *locations.get(&(current_x, current_y)).unwrap_or(&false) as i64,
                    );
                }
            }
        }

        Ok(())
    }

    pub fn part_1(input: &[i64]) -> Result<u32, MachineExecutionError> {
        let machine = MachineState::new_with_memory(&input.iter().copied());
        let mut locations: HashMap<(i32, i32), bool> = HashMap::new();
        run_machine(machine, &mut locations)?;

        Ok(locations.len() as u32)
    }

    fn format_map(map: &HashMap<(i32, i32), bool>) -> String {
        let (max_x, min_x, max_y, min_y) = map.iter().fold(
            (i32::MIN, i32::MAX, i32::MIN, i32::MAX),
            |(max_x, min_x, max_y, min_y), ((x, y), _)| {
                let max_x = max_x.max(*x);
                let min_x = min_x.min(*x);
                let max_y = max_y.max(*y);
                let min_y = min_y.min(*y);
                (max_x, min_x, max_y, min_y)
            },
        );

        let mut result =
            String::with_capacity((max_x - min_x + 2) as usize * (max_y - min_y + 1) as usize);

        for y in 0..=max_y - min_y {
            for x in min_x..=max_x {
                result.push(if *map.get(&(x, max_y - y)).unwrap_or(&false) {
                    'X'
                } else {
                    '.'
                });
            }
            result.push('\n');
        }

        result
    }

    pub fn part_2(input: &[i64]) -> Result<String, MachineExecutionError> {
        let machine = MachineState::new_with_memory(&input.iter().copied());
        let mut locations: HashMap<(i32, i32), bool> = HashMap::new();
        locations.insert((0, 0), true);
        run_machine(machine, &mut locations)?;

        Ok(format_map(&locations))
    }
}

#[cfg(test)]
mod tests {
    use super::day_11::*;

    #[test]
    #[cfg(not(feature = "no_real_inputs"))]
    fn test_day_11() {
        use insta::assert_snapshot;

        let input = input(include_str!("../input.txt"));
        assert_eq!(part_1(&input).unwrap(), 2441);
        assert_snapshot!(part_2(&input).unwrap());
    }
}
