pub mod day_4 {
    use std::cmp::Ordering;

    pub fn input(s: &str) -> (u32, u32) {
        let mut inputs = s.trim().split('-').map(|l| str::parse(l).unwrap());
        (inputs.next().unwrap(), inputs.next().unwrap())
    }

    pub(crate) fn is_valid(i: u32) -> bool {
        let mut i = i;
        let mut prev = 10;
        let mut has_double = false;
        while i > 0 {
            let digit = i % 10;
            i /= 10;

            match digit.cmp(&prev) {
                Ordering::Greater => {
                    return false;
                }
                Ordering::Equal => {
                    has_double = true;
                }
                Ordering::Less => {}
            }

            prev = digit;
        }

        has_double
    }

    pub(crate) fn is_valid_2(i: u32) -> bool {
        let mut i = i;
        let mut prev = 10;
        let mut has_double = false;
        let mut current_run_len = 1;

        while i > 0 {
            let digit = i % 10;
            i /= 10;

            match digit.cmp(&prev) {
                Ordering::Greater => {
                    return false;
                }
                Ordering::Equal => {
                    current_run_len += 1;
                }
                Ordering::Less => {
                    has_double |= current_run_len == 2;
                    current_run_len = 1;
                }
            }

            prev = digit;
        }

        has_double || current_run_len == 2
    }

    pub fn part_1(low: u32, high: u32) -> u32 {
        // Can't be bothered to do this efficiently, although IIRC I did this correctly for
        // a Project Euler problem which had much more rigorous requirements.
        (u32::max(low, 123456)..=u32::min(high, 999999))
            .filter(|&x| is_valid(x))
            .count() as u32
    }

    pub fn part_2(low: u32, high: u32) -> u32 {
        (u32::max(low, 100000)..=u32::min(high, 998888))
            .filter(|&x| is_valid_2(x))
            .count() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::day_4::*;

    #[test]
    fn test_is_valid() {
        assert!(is_valid(111111));
        assert!(!is_valid(223450));
        assert!(!is_valid(123789));
    }

    #[test]
    fn test_is_valid_2() {
        assert!(is_valid_2(112233));
        assert!(!is_valid_2(123444));
        assert!(is_valid(111122));
    }

    #[test]
    #[cfg(not(feature = "no_real_inputs"))]
    fn test_day_4() {
        let (low, high) = input(include_str!("../input.txt"));
        assert_eq!(part_1(low, high), 1855);
        assert_eq!(part_2(low, high), 1253);
    }
}
