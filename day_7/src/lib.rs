pub mod day_7 {
    use std::array;

    use intcode::intcode::{num, StepIoResult};
    use intcode::intcode::{MachineExecutionError, MachineState};
    use itertools::Itertools;

    pub fn input(s: &str) -> Vec<i32> {
        s.trim()
            .split(',')
            .map(|l| str::parse(l).unwrap())
            .collect()
    }

    #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    enum ExecutionState<T> {
        AwaitingInput(usize),
        OutputPending(T),
        Ready,
        Terminated,
    }

    pub fn initialise<F, const N: usize>(
        phase: &[u8],
        machines: &mut [MachineState<i32>; N],
        num: &num::NumImpl<i32, F>,
    ) -> Result<(), MachineExecutionError>
    where
        F: Fn(i32) -> Option<usize>,
    {
        for i in 0..N {
            let phase = phase[i];
            match machines[i].execute_until_input(num)? {
                StepIoResult::AwaitingInput(loc) => {
                    machines[i].set_mem_elt(loc, phase as i32)?;
                }
                _ => {
                    panic!("unexpected IO result from machine {i}");
                }
            }
        }
        Ok(())
    }

    /// Runs until machine E emits a value, returning that value;
    /// or until all machines have halted, in which case you get back None.
    fn execute<F, const N: usize>(
        input_to_first: Option<i32>,
        readiness: &mut [ExecutionState<i32>; N],
        machines: &mut [MachineState<i32>; N],
        num: &num::NumImpl<i32, F>,
    ) -> Result<Option<i32>, MachineExecutionError>
    where
        F: Fn(i32) -> Option<usize>,
    {
        let mut first_input_consumed = false;

        loop {
            let mut progress_made = false;

            for i in 0..N {
                match readiness[i] {
                    ExecutionState::Ready => {
                        progress_made = true;
                        match machines[i].execute_until_input(num)? {
                            StepIoResult::Terminated => {
                                readiness[i] = ExecutionState::Terminated;
                            }
                            StepIoResult::Output(output_val) => {
                                readiness[i] = ExecutionState::OutputPending(output_val)
                            }
                            StepIoResult::AwaitingInput(loc) => {
                                readiness[i] = ExecutionState::AwaitingInput(loc)
                            }
                        }
                    }
                    ExecutionState::AwaitingInput(loc) => {
                        if i == 0 {
                            if !first_input_consumed {
                                progress_made = true;
                                match input_to_first {
                                    None => {
                                        machines[0].set_mem_elt(loc, 0)?;
                                    }
                                    Some(input) => {
                                        machines[0].set_mem_elt(loc, input)?;
                                        readiness[N - 1] = ExecutionState::Ready;
                                    }
                                }
                                readiness[0] = ExecutionState::Ready;
                                first_input_consumed = true;
                            }
                        } else {
                            match readiness[i - 1] {
                                ExecutionState::OutputPending(output) => {
                                    progress_made = true;
                                    machines[i].set_mem_elt(loc, output)?;
                                    readiness[i] = ExecutionState::Ready;
                                    readiness[i - 1] = ExecutionState::Ready;
                                }
                                ExecutionState::Terminated => {
                                    panic!("Machine {i} is waiting for input which will never come due to termination of another machine")
                                }
                                _ => {}
                            }
                        }
                    }
                    ExecutionState::OutputPending(val) => {
                        // first_input_consumed, to determine whether the output is
                        // still pending from a previous round
                        if i == N - 1 && first_input_consumed {
                            return Ok(Some(val));
                        }
                    }
                    ExecutionState::Terminated => {}
                }
            }

            if !progress_made {
                return Ok(None);
            }
        }
    }

    fn clear_all<T, I>(machines: &mut [MachineState<T>], initial: &I)
    where
        I: IntoIterator<Item = T>,
        I: Clone,
    {
        for machine in machines {
            machine.reset(initial.clone());
        }
    }

    pub fn part_1<T>(numbers: &T) -> Result<i32, MachineExecutionError>
    where
        T: IntoIterator<Item = i32>,
        T: Clone,
    {
        let num = num::i32();
        let mut machines: [MachineState<_>; 5] =
            array::from_fn(|_| MachineState::new_with_memory(numbers));

        let mut best = i32::MIN;

        for phase in (0..=4).permutations(5) {
            initialise(&phase, &mut machines, &num)?;
            let mut readiness = [ExecutionState::<i32>::Ready; 5];

            let result = execute(None, &mut readiness, &mut machines, &num)?.unwrap();
            if result > best {
                best = result;
            }

            clear_all(&mut machines, numbers);
        }

        Ok(best)
    }

    pub fn part_2<T>(numbers: &T) -> Result<i32, MachineExecutionError>
    where
        T: IntoIterator<Item = i32>,
        T: Clone,
    {
        let num = num::i32();
        let mut machines: [MachineState<_>; 5] =
            array::from_fn(|_| MachineState::new_with_memory(numbers));

        let mut best = i32::MIN;

        for phase in (5..=9).permutations(5) {
            initialise(&phase, &mut machines, &num)?;

            let mut readiness = [ExecutionState::<i32>::Ready; 5];

            let mut input_to_first = None;

            while let Some(result) = execute(input_to_first, &mut readiness, &mut machines, &num)? {
                input_to_first = Some(result);
            }

            if let Some(x) = input_to_first {
                if x > best {
                    best = x;
                }
            }
            clear_all(&mut machines, numbers);
        }

        Ok(best)
    }
}

#[cfg(test)]
mod tests {
    use super::day_7::*;

    #[test]
    fn test_part_1() {
        let i = input("3,15,3,16,1002,16,10,16,1,16,15,15,4,15,99,0,0");
        assert_eq!(part_1(&i).unwrap(), 43210);
        let i = input("3,23,3,24,1002,24,10,24,1002,23,-1,23,101,5,23,23,1,24,23,23,4,23,99,0,0");
        assert_eq!(part_1(&i).unwrap(), 54321);
        let i = input("3,31,3,32,1002,32,10,32,1001,31,-2,31,1007,31,0,33,1002,33,7,33,1,33,31,31,1,32,31,31,4,31,99,0,0,0");
        assert_eq!(part_1(&i).unwrap(), 65210);
    }

    #[test]
    fn test_part_2() {
        let i = input(
            "3,26,1001,26,-4,26,3,27,1002,27,2,27,1,27,26,27,4,27,1001,28,-1,28,1005,28,6,99,0,0,5",
        );
        assert_eq!(part_2(&i).unwrap(), 139629729);
        let i = input("3,52,1001,52,-5,52,3,53,1,52,56,54,1007,54,5,55,1005,55,26,1001,54,-5,54,1105,1,12,1,53,54,53,1008,54,0,55,1001,55,1,55,2,53,55,53,4,53,1001,56,-1,56,1005,56,6,99,0,0,0,0,10");
        assert_eq!(part_2(&i).unwrap(), 18216);
    }

    #[test]
    #[cfg(not(feature = "no_real_inputs"))]
    fn test_day_7() {
        let input = input(include_str!("../input.txt"));
        assert_eq!(part_1(&input).unwrap(), 255590);
        assert_eq!(part_2(&input).unwrap(), 58285150);
    }
}
