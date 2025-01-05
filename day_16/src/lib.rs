pub mod day_16 {
    pub fn input(s: &str) -> Vec<i32> {
        s.trim()
            .split('\n')
            .map(|l| str::parse(l).unwrap())
            .collect()
    }

    pub fn part_1<T>(numbers: &[T]) -> u32 {
        todo!()
    }

    pub fn part_2<T>(numbers: &T) -> u32 {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::day_16::*;

    #[test]
    fn part1_known_1() {
        let input = input("80871224585914546619083218645595");
        assert_eq!(part_1(&input), 24176176);
    }

    #[test]
    fn part1_known_2() {
        let input = input("19617804207202209144916044189917");
        assert_eq!(part_1(&input), 73745418);
    }

    #[test]
    fn part1_known_3() {
        let input = input("69317163492948606335995924319873");
        assert_eq!(part_1(&input), 52432133);
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
        assert_eq!(part_1(&input), 0);
        assert_eq!(part_2(&input), 4948732);
    }
}
