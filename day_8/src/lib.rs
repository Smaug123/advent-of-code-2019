pub mod day_8 {
    use std::fmt::{Display, Write};

    pub struct Board<const ROWS: usize, const COLS: usize> {
        elts: [[u8; COLS]; ROWS],
    }

    impl<const ROWS: usize, const COLS: usize> Display for Board<ROWS, COLS> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            for row in self.elts {
                for elt in row {
                    f.write_char(match elt {
                        2 => ' ',
                        1 => 'X',
                        0 => '.',
                        _ => {
                            panic!("bad elt {elt}");
                        }
                    })?
                }
                f.write_char('\n')?;
            }
            Ok(())
        }
    }

    pub fn input<const ROWS: usize, const COLS: usize>(s: &str) -> Vec<Board<ROWS, COLS>> {
        let mut result = Vec::new();
        let mut start = [[0; COLS]; ROWS];
        let mut row = 0;
        let mut col = 0;
        for c in s.chars() {
            start[row][col] = char::to_digit(c, 10).unwrap() as u8;
            if col == COLS - 1 {
                col = 0;
                if row == ROWS - 1 {
                    row = 0;
                    result.push(Board { elts: start });
                    start = [[0; COLS]; ROWS];
                } else {
                    row += 1;
                }
            } else {
                col += 1;
            }
        }

        result
    }

    pub fn part_1<const ROWS: usize, const COLS: usize>(input: &[Board<ROWS, COLS>]) -> u32 {
        let best_layer = input
            .iter()
            .min_by_key(|layer| {
                layer
                    .elts
                    .iter()
                    .flat_map(|row| row.iter())
                    .filter(|x| **x == 0)
                    .count()
            })
            .unwrap();
        let mut ones = 0;
        let mut twos = 0;
        for row in best_layer.elts.iter() {
            for i in row {
                match *i {
                    1 => {
                        ones += 1;
                    }
                    2 => {
                        twos += 1;
                    }
                    _ => {}
                }
            }
        }

        ones * twos
    }

    pub fn part_2<const ROWS: usize, const COLS: usize>(
        input: &[Board<ROWS, COLS>],
    ) -> Board<ROWS, COLS> {
        // 2 = transparent, 1 = white, 0 = black
        let mut result = [[2; COLS]; ROWS];

        for layer in input {
            for col in 0..COLS {
                for row in 0..ROWS {
                    match result[row][col] {
                        0 => {}
                        1 => {}
                        2 => result[row][col] = layer.elts[row][col],
                        _ => {
                            panic!("logic error");
                        }
                    }
                }
            }
        }

        Board { elts: result }
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::day_8::*;

    #[test]
    fn test_part1_known() {
        let input = input::<2, 3>("123456789012");
        assert_eq!(part_1(&input), 1);
    }

    #[test]
    fn test_part2_known() {
        let input = input::<2, 2>("0222112222120000");
        assert_snapshot!(part_2(&input));
    }

    #[test]
    #[cfg(not(feature = "no_real_inputs"))]
    fn test_day_8() {
        let input = input::<6, 25>(include_str!("../input.txt"));
        assert_eq!(part_1(&input), 2016);
        assert_snapshot!(part_2(&input));
    }
}
