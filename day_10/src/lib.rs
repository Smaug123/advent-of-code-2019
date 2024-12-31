pub mod day_10 {
    use std::fmt::Write;

    #[derive(Clone)]
    pub struct Board {
        elts: Vec<bool>,
        row_count: usize,
        col_count: usize,
    }

    impl std::fmt::Display for Board {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            for row in 0..self.get_row_count() {
                for col in 0..self.get_col_count() {
                    if self.get(row, col).unwrap() {
                        f.write_char('#')?;
                    } else {
                        f.write_char('.')?;
                    }
                }
                f.write_char('\n')?;
            }

            Ok(())
        }
    }

    impl Board {
        pub fn new_with_size(row_count: usize, col_count: usize) -> Board {
            let mut elts = Vec::with_capacity(row_count * col_count);
            elts.extend(std::iter::repeat_n(false, row_count * col_count));
            Board {
                elts,
                row_count,
                col_count,
            }
        }

        pub fn overwrite(&mut self, other: &Board) {
            assert!(other.row_count == self.row_count);
            assert!(other.col_count == self.col_count);
            self.elts.clear();
            self.elts.extend(other.elts.iter());
        }

        pub fn get_row_count(&self) -> usize {
            self.row_count
        }
        pub fn get_col_count(&self) -> usize {
            self.col_count
        }
        pub fn get(&self, row: usize, col: usize) -> Option<bool> {
            let index = row * self.get_col_count() + col;
            self.elts.get(index).cloned()
        }
        pub fn set(&mut self, row: usize, col: usize, val: bool) {
            let index = row * self.get_col_count() + col;
            *self.elts.get_mut(index).unwrap() = val;
        }
        pub fn parse(s: &str) -> Option<Board> {
            let s = s.trim();
            let col_count = s.find('\n')?;
            // +1 for the trailing newline
            let row_count = (s.len() + 1) / (col_count) - 1;
            let mut elts = Vec::with_capacity(col_count * row_count);
            for c in s.chars() {
                if c == '\n' {
                    continue;
                }
                elts.push(c == '#')
            }

            Some(Board {
                elts,
                row_count,
                col_count,
            })
        }
    }

    pub fn input(s: &str) -> Board {
        Board::parse(s).unwrap()
    }

    pub fn part_1(input: &Board) -> u32 {
        // I find this kind of thing deathly dull, so here's a really dumb algorithm.
        let mut best = 0;
        let mut copy = Board::new_with_size(input.row_count, input.col_count);

        for row in 0..input.row_count {
            for col in 0..input.col_count {
                if !input.get(row, col).unwrap() {
                    continue;
                }

                copy.overwrite(input);

                let mut asteroids = 0;

                for direction_row_sign in [-1, 1] {
                    for direction_col_sign in [-1, 1] {
                        for direction_row in 0..(input.row_count as i32) {
                            for direction_col in 0..(input.col_count as i32) {
                                if direction_row == 0 && direction_col == 0 {
                                    continue;
                                }
                                let first_in_direction_row =
                                    row as i32 + direction_row * direction_row_sign;
                                let first_in_direction_col =
                                    col as i32 + direction_col * direction_col_sign;

                                if first_in_direction_col < 0 || first_in_direction_row < 0 {
                                    break;
                                }
                                let first_in_direction_row = first_in_direction_row as usize;
                                let first_in_direction_col = first_in_direction_col as usize;
                                if first_in_direction_col >= input.get_col_count()
                                    || first_in_direction_row >= input.get_row_count()
                                {
                                    break;
                                }

                                let mut has_found = false;
                                for i in 1.. {
                                    let row = row as i32 + i * direction_row * direction_row_sign;
                                    let col = col as i32 + i * direction_col * direction_col_sign;
                                    if row < 0 || col < 0 {
                                        break;
                                    }
                                    let row = row as usize;
                                    let col = col as usize;
                                    if row >= input.get_row_count() || col >= input.get_col_count()
                                    {
                                        break;
                                    }
                                    if copy.get(row, col) == Some(true) {
                                        if !has_found {
                                            has_found = true;
                                            asteroids += 1;
                                        }
                                        copy.set(row, col, false);
                                    }
                                }
                            }
                        }
                    }
                }

                if asteroids > best {
                    best = asteroids;
                }
            }
        }

        best
    }

    pub fn part_2(input: &Board) -> u32 {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::day_10::*;

    #[test]
    fn part1_known_1() {
        let board = input(
            ".#..#
.....
#####
....#
...##",
        );
        assert_eq!(part_1(&board), 8);
    }

    #[test]
    fn part1_known_2() {
        let input = input(
            "......#.#.
#..#.#....
..#######.
.#.#.###..
.#..#.....
..#....#.#
#..#....#.
.##.#..###
##...#..#.
.#....####",
        );
        assert_eq!(part_1(&input), 33);
    }

    #[test]
    fn part1_known_3() {
        let board = input(
            "#.#...#.#.
.###....#.
.#....#...
##.#.#.#.#
....#.#.#.
.##..###.#
..#...##..
..##....##
......#...
.####.###.",
        );
        assert_eq!(part_1(&board), 35);
    }
    #[test]
    fn part1_known_4() {
        let board = input(
            ".#..#..###
####.###.#
....###.#.
..###.##.#
##.##.#.#.
....###..#
..#.#..#.#
#..#.#.###
.##...##.#
.....#.#..",
        );
        assert_eq!(part_1(&board), 41);
    }
    #[test]
    fn part1_known_5() {
        let board = input(
            ".#..##.###...#######
##.############..##.
.#.######.########.#
.###.#######.####.#.
#####.##.#.##.###.##
..#####..#.#########
####################
#.####....###.#.#.##
##.#################
#####.##.###..####..
..######..##.#######
####.##.####...##..#
.#####..#.######.###
##...#.##########...
#.##########.#######
.####.#.###.###.#.##
....##.##.###..#####
.#.#.###########.###
#.#.#.#####.####.###
###.##.####.##.#..##",
        );
        assert_eq!(part_1(&board), 210);
    }

    #[test]
    fn part2_known() {
        {
            let board = input(
                ".#..##.###...#######
##.############..##.
.#.######.########.#
.###.#######.####.#.
#####.##.#.##.###.##
..#####..#.#########
####################
#.####....###.#.#.##
##.#################
#####.##.###..####..
..######..##.#######
####.##.####...##..#
.#####..#.######.###
##...#.##########...
#.##########.#######
.####.#.###.###.#.##
....##.##.###..#####
.#.#.###########.###
#.#.#.#####.####.###
###.##.####.##.#..##",
            );
            assert_eq!(part_2(&board), 802);
        }
    }

    #[test]
    #[cfg(not(feature = "no_real_inputs"))]
    fn test_day_10() {
        let input = input(include_str!("../input.txt"));
        assert_eq!(part_1(&input), 314);
        assert_eq!(part_2(&input), 0);
    }
}
