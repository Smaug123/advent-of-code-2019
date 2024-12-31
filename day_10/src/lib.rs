pub mod day_10 {
    pub struct Board {
        elts: Vec<bool>,
        row_count: usize,
        col_count: usize,
    }

    impl Board {
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
        pub fn parse(s: &str) -> Option<Board> {
            let s = s.trim();
            let col_count = s.find('\n')?;
            // +1 for the trailing newline; -1 for the line breaks
            let row_count = (s.len() + 1) / (col_count - 1);
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
        0
    }

    pub fn part_2(input: &Board) -> u32 {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::day_10::*;

    #[test]
    fn part1_known() {
        {
            let board = input(
                ".#..#
.....
#####
....#
...##",
            );
            assert_eq!(part_1(&board), 8);
        }
        {
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
        {
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
        {
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
            assert_eq!(part_1(&board), 210);
        }
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
