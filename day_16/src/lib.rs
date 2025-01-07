pub mod day_16 {
    pub fn input(s: &str) -> Vec<i16> {
        s.trim()
            .chars()
            .map(|l: char| {
                if l.is_ascii_digit() {
                    ((l as u8) - b'0') as i16
                } else {
                    panic!("bad")
                }
            })
            .collect()
    }

    pub(crate) fn single_phase(input: &[i16], output: &mut [i16]) {
        // First entry:
        {
            let mut value = 0;
            let mut multiplier = 1;
            let mut i = 0;
            while i < input.len() {
                value += input[i] * multiplier;
                multiplier *= -1;
                i += 2;
            }
            value %= 10;
            if value < 0 {
                value = -value;
            }
            output[0] = value;
        }

        // Entries up to halfway:
        for (i, output_loc) in output
            .iter_mut()
            .enumerate()
            .take(input.len() / 2 + 1)
            .skip(1)
        {
            let mut value = 0;
            let mut count = 0;
            let mut multiplier = 1;
            let mut index = i;
            while index < input.len() {
                value += (multiplier * input[index]) % 10;
                count += 1;
                if count > i {
                    count = 0;
                    multiplier *= -1;
                    index += i + 2;
                } else {
                    index += 1;
                }
            }
            if value < 0 {
                value = -value;
            }
            value %= 10;
            *output_loc = value;
        }

        // The last half is easy because (j + 1) / (i + 1) = 0 or 1.

        let mut big_sum: i16 = input.iter().skip(input.len() / 2 + 1).copied().sum();
        output[input.len() / 2 + 1] = big_sum % 10;

        for i in input.len() / 2 + 2..input.len() {
            big_sum -= input[i - 1];
            output[i] = big_sum % 10;
        }
    }

    pub fn part_1(numbers: &[i16]) -> String {
        let mut next_phase: Vec<i16> = Vec::with_capacity(numbers.len());
        next_phase.extend(numbers);
        let mut next_phase_2: Vec<i16> = vec![0; numbers.len()];

        for _ in 0..50 {
            single_phase(&next_phase, &mut next_phase_2);
            single_phase(&next_phase_2, &mut next_phase);
        }

        next_phase
            .iter()
            .take(8)
            .map(|x| char::from_digit(*x as u32, 10).unwrap())
            .collect()
    }

    pub fn part_2<T>(_numbers: &T) -> u32 {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::day_16::*;

    #[test]
    fn part1_known_single_1() {
        let mut input = input("12345678");
        let mut output = Vec::with_capacity(input.len());
        output.resize(input.len(), 0);
        single_phase(&input, &mut output);
        assert_eq!(output, [4, 8, 2, 2, 6, 1, 5, 8]);
        single_phase(&output, &mut input);
        assert_eq!(input, [3, 4, 0, 4, 0, 4, 3, 8]);
        single_phase(&input, &mut output);
        assert_eq!(output, [0, 3, 4, 1, 5, 5, 1, 8]);
        single_phase(&output, &mut input);
        assert_eq!(input, [0, 1, 0, 2, 9, 4, 9, 8]);
    }

    #[test]
    fn part1_known_1() {
        let input = input("80871224585914546619083218645595");
        assert_eq!(part_1(&input), "24176176");
    }

    #[test]
    fn part1_known_2() {
        let input = input("19617804207202209144916044189917");
        assert_eq!(part_1(&input), "73745418");
    }

    #[test]
    fn part1_known_3() {
        let input = input("69317163492948606335995924319873");
        assert_eq!(part_1(&input), "52432133");
    }

    #[test]
    fn part2_known() {
        assert_eq!(part_2(&[14]), 2);
        assert_eq!(part_2(&[1969]), 966);
        assert_eq!(part_2(&[100756]), 50346);
    }

    #[test]
    #[cfg(not(feature = "no_real_inputs"))]
    fn test_day_16() {
        let input = input(include_str!("../input.txt"));
        assert_eq!(part_1(&input), "76795888");
        assert_eq!(part_2(&input), 0);
    }
}
