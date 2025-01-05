pub mod day_19 {
    use intcode::ast::{Ast, Condition};
    use intcode::intcode::{MachineExecutionError, MachineState};
    use intcode::linked_list::List;

    pub fn input(s: &str) -> Vec<i64> {
        s.trim()
            .split(',')
            .map(|l| str::parse(l).unwrap())
            .collect()
    }

    fn get_output(input: &[i64]) -> Result<Ast, MachineExecutionError> {
        let mut machine = MachineState::new_with_memory(&input.iter().copied().map(Ast::Constant));
        match machine.execute_until_input()? {
            intcode::intcode::StepIoResult::Terminated => {
                panic!("terminated unexpectedly");
            }
            intcode::intcode::StepIoResult::Output(_) => {
                panic!("unexpectedly output");
            }
            intcode::intcode::StepIoResult::AwaitingInput(loc) => {
                machine.set_mem_elt(loc, Ast::Variable('x'));
            }
        };
        match machine.execute_until_input()? {
            intcode::intcode::StepIoResult::Terminated => {
                panic!("terminated unexpectedly");
            }
            intcode::intcode::StepIoResult::Output(_) => {
                panic!("unexpectedly output");
            }
            intcode::intcode::StepIoResult::AwaitingInput(loc) => {
                machine.set_mem_elt(loc, Ast::Variable('y'));
            }
        };
        let output = match machine.execute_until_input()? {
            intcode::intcode::StepIoResult::Terminated => {
                panic!("terminated unexpectedly");
            }
            intcode::intcode::StepIoResult::AwaitingInput(_) => {
                panic!("unexpectedly asked for input");
            }
            intcode::intcode::StepIoResult::Output(ast) => ast,
        };

        Ok(output)
    }

    pub fn part_1(input: &[i64]) -> Result<u32, MachineExecutionError> {
        let output = get_output(input)?.simplify(&List::new());
        let mut result = 0;
        for y in 0..=49 {
            for x in 0..=49 {
                let query_result = output
                    .eval(&mut |c| match c {
                        'x' => Some(x),
                        'y' => Some(y),
                        _ => None,
                    })
                    .unwrap();
                result += query_result as u32
            }
        }
        Ok(result)
    }

    /// Returns the first x strictly after x_known_good for which f(x) is false,
    /// assuming that F's support is [i, infty) for some i > x_known_good.
    /// You must ensure yourself that f(x_known_good) == true.
    fn find_upper_boundary<F>(x_known_good: i64, f: &mut F) -> i64
    where
        F: FnMut(i64) -> bool,
    {
        let x_known_good = if x_known_good == 0 {
            if f(1) {
                1
            } else {
                return 1;
            }
        } else {
            x_known_good
        };

        let mut upper_false = 2 * x_known_good;
        loop {
            assert!(upper_false != 0);
            if f(upper_false) {
                // Keep doubling!
                upper_false *= 2;
            } else {
                // Walked past the top.
                break;
            }
        }

        let mut lower_true = x_known_good;

        // Loop invariant: upper_false is known to be false and lower_true is known to be true.
        while lower_true + 1 < upper_false {
            let midpoint = (upper_false - lower_true) / 2 + lower_true;
            // midpoint > lower_true, because upper_false - lower_true >= 2 due to the `while` condition.
            if f(midpoint) {
                lower_true = midpoint;
            } else {
                upper_false = midpoint;
            }
        }

        upper_false
    }

    pub fn part_2(input: &[i64]) -> Result<i64, MachineExecutionError> {
        let output = get_output(input)?.simplify(
            &List::new()
                .prepend(Condition::LessThan(
                    Box::new(Ast::Zero),
                    Box::new(Ast::Variable('y')),
                ))
                .prepend(Condition::LessThan(
                    Box::new(Ast::Zero),
                    Box::new(Ast::Variable('x')),
                )),
        );

        let desired_dim = 100;

        let mut start_x = 1;
        let mut best_x = -1;
        let result = find_upper_boundary(9, &mut |y| {
            let old_start_x = start_x;
            let mut x = start_x;
            let mut found_x = false;
            loop {
                let v = output
                    .eval(&mut |v| if v == 'x' { Some(x) } else { Some(y) })
                    .unwrap();
                if !found_x && v == 1 {
                    start_x = x;
                    found_x = true;
                } else if !found_x {
                    x += 1;
                    continue;
                }
                if v == 0 {
                    // walked off the end
                    break true;
                }
                let is_good_row = output
                    .eval(&mut |v| {
                        if v == 'x' {
                            Some(x + desired_dim - 1)
                        } else {
                            Some(y)
                        }
                    })
                    .unwrap()
                    == 1;
                if is_good_row {
                    if output
                        .eval(&mut |v| {
                            if v == 'x' {
                                Some(x)
                            } else {
                                Some(y + desired_dim - 1)
                            }
                        })
                        .unwrap()
                        == 1
                    {
                        start_x = old_start_x;
                        best_x = x;
                        return false;
                    } else {
                        x += 1;
                    }
                } else {
                    // Row is too short; get a new line.
                    break true;
                }
            }
        });
        Ok(best_x * 10000 + result)
    }
}

#[cfg(test)]
mod tests {
    use super::day_19::*;

    #[test]
    #[cfg(not(feature = "no_real_inputs"))]
    fn test_day_19() {
        let input = input(include_str!("../input.txt"));
        assert_eq!(part_1(&input).unwrap(), 226);
        assert_eq!(part_2(&input).unwrap(), 7900946);
    }
}
