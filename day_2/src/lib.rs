pub mod day_2 {
    use intcode::intcode::{MachineExecutionError, MachineState};

    pub fn input(s: &str) -> Vec<usize> {
        s.trim()
            .split(',')
            .map(|l| str::parse(l).unwrap())
            .collect()
    }

    pub fn part_1<T>(numbers: &T) -> Result<usize, MachineExecutionError>
    where
        T: IntoIterator<Item = usize>,
        T: Clone,
    {
        let mut machine = MachineState::new_with_memory(numbers);
        machine.set_mem_elt(1, 12)?;
        machine.set_mem_elt(2, 2)?;

        machine.execute_to_end()?;

        let result = machine.read_mem_elt(0)?;
        Ok(*result)
    }

    pub fn part_2<T>(numbers: &T, target: usize) -> usize
    where
        T: IntoIterator<Item = usize>,
        T: Clone,
    {
        let mut machine = MachineState::new();
        let (noun, verb) = (0..=99)
            .filter_map(|noun| {
                (0..=99)
                    .filter_map(|verb| {
                        machine.reset_memory(numbers.clone());
                        machine.set_mem_elt(1, noun).ok()?;
                        machine.set_mem_elt(2, verb).ok()?;
                        machine.execute_to_end().ok()?;
                        // safety: on termination, program counter is on opcode 99,
                        // so there is an element in the array
                        if *machine.read_mem_elt(0).unwrap() == target {
                            Some((noun, verb))
                        } else {
                            None
                        }
                    })
                    .next()
            })
            .next()
            .unwrap();
        100 * noun + verb
    }
}

#[cfg(test)]
mod tests {
    use super::day_2::*;

    #[test]
    #[cfg(not(feature = "no_real_inputs"))]
    fn test_day_2() {
        let input = input(include_str!("../input.txt"));
        assert_eq!(part_1(&input).unwrap(), 3765464);
        assert_eq!(part_2(&input, 19690720), 7610);
    }
}
