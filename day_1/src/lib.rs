pub mod day_1 {
    pub fn input(s: &str) -> Vec<u32> {
        s.trim()
            .split('\n')
            .map(|l| str::parse(l).unwrap())
            .collect()
    }

    pub fn part_1<T>(numbers: &T) -> u32
    where
        T: IntoIterator<Item = u32>,
        T: Clone,
    {
        numbers.clone().into_iter().map(|n| (n / 3) - 2).sum()
    }

    pub fn part_2<T>(numbers: &T) -> u32
    where
        T: IntoIterator<Item = u32>,
        T: Clone,
    {
        numbers
            .clone()
            .into_iter()
            .map(|n| {
                let mut ans = 0;
                let mut n = n;
                while n > 6 {
                    let new = (n / 3) - 2;
                    ans += new;
                    n = new;
                }
                ans
            })
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::day_1::*;

    #[test]
    fn part1_known() {
        assert_eq!(part_1(&[12]), 2);
        assert_eq!(part_1(&[14]), 2);
        assert_eq!(part_1(&[1969]), 654);
        assert_eq!(part_1(&[100756]), 33583);
    }

    #[test]
    fn part2_known() {
        assert_eq!(part_2(&[14]), 2);
        assert_eq!(part_2(&[1969]), 966);
        assert_eq!(part_2(&[100756]), 50346);
    }

    #[test]
    #[cfg(not(feature = "no_real_inputs"))]
    fn test_day_1() {
        let input = input(include_str!("../input.txt"));
        assert_eq!(part_1(&input), 3301059);
        assert_eq!(part_2(&input), 4948732);
    }
}
