pub mod day_6 {
    pub fn input(s: &str) -> Vec<(&str, &str)> {
        s.trim()
            .split('\n')
            .map(|l| {
                let mut iter = l.split(')');
                (iter.next().unwrap(), iter.next().unwrap())
            })
            .collect()
    }

    pub fn part_1(input: &[(&str, &str)]) -> u32 {
        0
    }

    pub fn part_2(input: &[(&str, &str)]) -> u32 {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::day_6::*;

    #[test]
    fn test_part1_known() {
        let input = input(
            "COM)B
B)C
C)D
D)E
E)F
B)G
G)H
D)I
E)J
J)K
K)L",
        );
        assert_eq!(part_1(&input), 42);
    }

    #[test]
    fn test_is_valid_2() {
        let input = input(
            "COM)B
B)C
C)D
D)E
E)F
B)G
G)H
D)I
E)J
J)K
K)L
K)YOU
I)SAN",
        );
        assert_eq!(part_2(&input), 349);
    }

    #[test]
    #[cfg(not(feature = "no_real_inputs"))]
    fn test_day_6() {
        let input = input(include_str!("../input.txt"));
        assert_eq!(part_1(&input), 249308);
        assert_eq!(part_2(&input), 349);
    }
}
