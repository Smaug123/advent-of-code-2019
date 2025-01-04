pub mod day_13 {
    use std::collections::HashMap;

    use intcode::intcode::{MachineExecutionError, MachineState};

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    enum Tile {
        Empty,
        Wall,
        Block,
        Paddle,
        Ball,
    }

    impl Tile {
        fn from_int(i: i32) -> Option<Tile> {
            match i {
                0 => Some(Tile::Empty),
                1 => Some(Tile::Wall),
                2 => Some(Tile::Block),
                3 => Some(Tile::Paddle),
                4 => Some(Tile::Ball),
                _ => None,
            }
        }
    }

    pub fn input(s: &str) -> Vec<i32> {
        s.trim()
            .split(',')
            .map(|l| str::parse(l).unwrap())
            .collect()
    }

    fn render_board(outputs: &[i32]) -> (i32, HashMap<(i32, i32), Tile>) {
        let mut iter = outputs.iter().copied();
        let mut output = HashMap::new();
        let mut score = 0;
        loop {
            let x = match iter.next() {
                None => {
                    return (score, output);
                }
                Some(x) => x,
            };
            let y = iter.next().unwrap();

            if x == 0 && y == -1 {
                score = iter.next().unwrap();
            } else {
                let tile = iter.next().and_then(|x| Tile::from_int(x)).unwrap();
                output.insert((x, y), tile);
            }
        }
    }

    pub fn part_1(input: &[i32]) -> Result<u32, MachineExecutionError> {
        let mut machine = MachineState::new_with_memory(&input.iter().copied());
        let output = machine.execute_to_end(&mut std::iter::empty())?;

        let (_score, board) = render_board(&output);

        Ok(board.iter().filter(|(_, x)| **x == Tile::Block).count() as u32)
    }

    pub fn part_2(input: &[i32]) -> Result<i32, MachineExecutionError> {
        let mut machine = MachineState::new_with_memory(&input.iter().copied());
        machine.set_mem_elt(0, 2);

        let mut score = 0;
        let mut paddle_x = 0;
        let mut ball_x = 0;

        loop {
            match machine.execute_until_input()? {
                intcode::intcode::StepIoResult::Terminated => {
                    return Ok(score);
                }
                intcode::intcode::StepIoResult::Output(x) => {
                    // Get two more outputs
                    let y = match machine.execute_until_input()? {
                        intcode::intcode::StepIoResult::Terminated => {
                            panic!("Expected outputs to come in threes, but terminated");
                        }
                        intcode::intcode::StepIoResult::AwaitingInput(_) => {
                            panic!("Expected outputs to come in threes, but asked for input");
                        }
                        intcode::intcode::StepIoResult::Output(y) => y,
                    };
                    let v = match machine.execute_until_input()? {
                        intcode::intcode::StepIoResult::Terminated => {
                            panic!("Expected outputs to come in threes, but terminated");
                        }
                        intcode::intcode::StepIoResult::AwaitingInput(_) => {
                            panic!("Expected outputs to come in threes, but asked for input");
                        }
                        intcode::intcode::StepIoResult::Output(v) => v,
                    };
                    if x == -1 && y == 0 {
                        score = v;
                    } else {
                        let tile = Tile::from_int(v).unwrap();
                        match tile {
                            Tile::Ball => {
                                ball_x = x;
                            }
                            Tile::Paddle => {
                                paddle_x = x;
                            }
                            _ => {}
                        }
                    }
                }
                intcode::intcode::StepIoResult::AwaitingInput(loc) => match paddle_x.cmp(&ball_x) {
                    std::cmp::Ordering::Less => {
                        machine.set_mem_elt(loc, 1);
                    }
                    std::cmp::Ordering::Equal => {
                        machine.set_mem_elt(loc, 0);
                    }
                    std::cmp::Ordering::Greater => {
                        machine.set_mem_elt(loc, -1);
                    }
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::day_13::*;

    #[test]
    #[cfg(not(feature = "no_real_inputs"))]
    fn test_day_13() {
        let input = input(include_str!("../input.txt"));
        assert_eq!(part_1(&input).unwrap(), 376);
        assert_eq!(part_2(&input).unwrap(), 18509);
    }
}
