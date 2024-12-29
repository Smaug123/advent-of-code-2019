pub mod day_5 {
    use intcode::intcode::num;
    use intcode::intcode::{MachineExecutionError, MachineState};

    pub fn input(s: &str) -> Vec<i32> {
        s.trim()
            .split(',')
            .map(|l| str::parse(l).unwrap())
            .collect()
    }

    pub fn part_1<T>(numbers: &T) -> Result<i32, MachineExecutionError>
    where
        T: IntoIterator<Item = i32>,
        T: Clone,
    {
        let mut machine = MachineState::new_with_memory(numbers, std::iter::once(1));
        let outputs = machine.execute_to_end(&num::i32())?;
        let mut outputs_iter = outputs.iter().rev();
        let ans = *outputs_iter.next().unwrap();
        for &output in outputs_iter {
            if output != 0 {
                panic!("Didn't get 0 output")
            }
        }

        Ok(ans)
    }

    pub fn part_2<T>(numbers: &T) -> Result<i32, MachineExecutionError>
    where
        T: IntoIterator<Item = i32>,
        T: Clone,
    {
        let mut machine = MachineState::new_with_memory(numbers, std::iter::once(5));
        let outputs = machine.execute_to_end(&num::i32())?;
        if outputs.len() != 1 {
            panic!("bad len {}", outputs.len())
        }

        Ok(outputs[0])
    }
}

#[cfg(test)]
mod tests {
    use super::day_5::*;

    #[test]
    #[cfg(not(feature = "no_real_inputs"))]
    fn test_day_5() {
        let input = input(include_str!("../input.txt"));
        assert_eq!(part_1(&input).unwrap(), 6731945);
        assert_eq!(part_2(&input).unwrap(), 9571668);
    }
}
