pub mod day_9 {
    use intcode::intcode::num;
    use intcode::intcode::{MachineExecutionError, MachineState};

    pub fn input(s: &str) -> Vec<i64> {
        s.trim()
            .split(',')
            .map(|l| str::parse(l).unwrap())
            .collect()
    }

    pub fn part_1<T>(numbers: &T) -> Result<i64, MachineExecutionError>
    where
        T: IntoIterator<Item = i64>,
        T: Clone,
    {
        let mut machine = MachineState::new_with_memory(numbers);
        let outputs = machine.execute_to_end(&mut std::iter::once(1), &num::i64())?;
        let mut outputs_iter = outputs.iter().rev();
        let ans = *outputs_iter.next().unwrap();
        for &output in outputs_iter {
            if output != 0 {
                panic!("Didn't get 0 output")
            }
        }

        Ok(ans)
    }

    pub fn part_2<T>(numbers: &T) -> Result<i64, MachineExecutionError>
    where
        T: IntoIterator<Item = i64>,
        T: Clone,
    {
        let mut machine = MachineState::new_with_memory(numbers);
        let outputs = machine.execute_to_end(&mut std::iter::once(2), &num::i64())?;
        if outputs.len() != 1 {
            panic!("bad len {}", outputs.len())
        }

        Ok(outputs[0])
    }
}

#[cfg(test)]
mod tests {
    use super::day_9::*;

    #[test]
    #[cfg(not(feature = "no_real_inputs"))]
    fn test_day_9() {
        let input = input(include_str!("../input.txt"));
        assert_eq!(part_1(&input).unwrap(), 2775723069);
        assert_eq!(part_2(&input).unwrap(), 49115);
    }
}
