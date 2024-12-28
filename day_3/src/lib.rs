pub mod day_3 {
    use std::collections::HashMap;

    #[derive(Debug)]
    pub enum Direction {
        Left,
        Right,
        Up,
        Down,
    }

    impl Direction {
        pub fn parse(c: char) -> Option<Direction> {
            match c {
                'L' => Some(Direction::Left),
                'R' => Some(Direction::Right),
                'U' => Some(Direction::Up),
                'D' => Some(Direction::Down),
                _ => None,
            }
        }
    }

    #[derive(Debug)]
    pub struct Move {
        dir: Direction,
        distance: u32,
    }

    impl Move {
        pub fn parse(s: &str) -> Option<Move> {
            let mut chars = s.chars();
            let dir = Direction::parse(chars.next()?)?;
            let mut distance = 0u32;
            for c in chars {
                distance = distance * 10 + c.to_digit(10)?;
            }

            Some(Move { dir, distance })
        }
    }

    pub fn input(s: &str) -> (Vec<Move>, Vec<Move>) {
        let mut lines = s.trim().split('\n').map(|l| {
            l.split(',')
                .map(|m| Move::parse(m).unwrap())
                .collect::<Vec<Move>>()
        });
        (lines.next().unwrap(), lines.next().unwrap())
    }

    pub fn part_1(wire1: &[Move], wire2: &[Move]) -> u32 {
        let mut positions = HashMap::new();
        let mut x = 0;
        let mut y = 0;
        let mut steps = 0;

        for mov in wire1 {
            match mov.dir {
                Direction::Up => {
                    for _ in 0..mov.distance {
                        positions.entry((x, y)).or_insert(steps);
                        y += 1;
                        steps += 1;
                    }
                }
                Direction::Down => {
                    for _ in 0..mov.distance {
                        positions.entry((x, y)).or_insert(steps);
                        y -= 1;
                        steps += 1;
                    }
                }
                Direction::Left => {
                    for _ in 0..mov.distance {
                        positions.entry((x, y)).or_insert(steps);
                        x -= 1;
                        steps += 1;
                    }
                }
                Direction::Right => {
                    for _ in 0..mov.distance {
                        positions.entry((x, y)).or_insert(steps);
                        x += 1;
                        steps += 1;
                    }
                }
            }
        }

        let mut x = 0;
        let mut y = 0;
        let mut steps = 0;
        wire2
            .iter()
            .filter_map(|mov| {
                (0..mov.distance)
                    .filter_map(|_| match mov.dir {
                        Direction::Down => {
                            let ret = if positions.contains_key(&(x, y)) {
                                Some((x, y))
                            } else {
                                None
                            };
                            y -= 1;
                            steps += 1;
                            ret
                        }
                        Direction::Up => {
                            let ret = if positions.contains_key(&(x, y)) {
                                Some((x, y))
                            } else {
                                None
                            };
                            y += 1;
                            steps += 1;
                            ret
                        }
                        Direction::Left => {
                            let ret = if positions.contains_key(&(x, y)) {
                                Some((x, y))
                            } else {
                                None
                            };
                            x -= 1;
                            steps += 1;
                            ret
                        }
                        Direction::Right => {
                            let ret = if positions.contains_key(&(x, y)) {
                                Some((x, y))
                            } else {
                                None
                            };
                            x += 1;
                            steps += 1;
                            ret
                        }
                    })
                    .map(|(x, y)| i32::abs(x) as u32 + i32::abs(y) as u32)
                    .filter(|x| *x > 0)
                    .min()
            })
            .filter(|x| *x > 0)
            .min()
            .unwrap()
    }

    pub fn part_2(wire1: &[Move], wire2: &[Move]) -> u32 {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::day_3::*;

    #[test]
    fn part1_known() {
        {
            let (wire1, wire2) = input("R8,U5,L5,D3\nU7,R6,D4,L4");
            assert_eq!(part_1(&wire1, &wire2), 6);
        }
        {
            let (wire1, wire2) =
                input("R75,D30,R83,U83,L12,D49,R71,U7,L72\nU62,R66,U55,R34,D71,R55,D58,R83");
            assert_eq!(part_1(&wire1, &wire2), 159);
        }
        {
            let (wire1, wire2) = input(
                "R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51\nU98,R91,D20,R16,D67,R40,U7,R15,U6,R7",
            );
            assert_eq!(part_1(&wire1, &wire2), 135);
        }
    }

    #[test]
    fn part2_known() {
        // assert_eq!(part_2(&[14]), 2);
        // assert_eq!(part_2(&[1969]), 966);
        // assert_eq!(part_2(&[100756]), 50346);
    }

    #[test]
    #[cfg(not(feature = "no_real_inputs"))]
    fn test_day_1() {
        let (wire1, wire2) = input(include_str!("../input.txt"));
        assert_eq!(part_1(&wire1, &wire2), 225);
        assert_eq!(part_2(&wire1, &wire2), 35194);
    }
}
