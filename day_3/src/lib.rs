pub mod day_3 {
    use std::{collections::HashMap, u32};

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

    fn extend_wire(wire: &[Move]) -> HashMap<(i32, i32), u32> {
        let mut positions = HashMap::new();
        wire.iter().fold((0u32, 0i32, 0i32), |(steps, x, y), mov| {
            let (x, y) = match mov.dir {
                Direction::Up => {
                    for i in 0..mov.distance {
                        positions.entry((x, y + (i as i32))).or_insert(steps + i);
                    }
                    (x, y + mov.distance as i32)
                }
                Direction::Down => {
                    for i in 0..mov.distance {
                        positions.entry((x, y - (i as i32))).or_insert(steps + i);
                    }
                    (x, y - mov.distance as i32)
                }
                Direction::Left => {
                    for i in 0..mov.distance {
                        positions.entry((x - (i as i32), y)).or_insert(steps + i);
                    }
                    (x - mov.distance as i32, y)
                }
                Direction::Right => {
                    for i in 0..mov.distance {
                        positions.entry((x + (i as i32), y)).or_insert(steps + i);
                    }
                    (x + mov.distance as i32, y)
                }
            };
            (steps + mov.distance, x, y)
        });

        positions
    }

    pub fn part_1(wire1: &[Move], wire2: &[Move]) -> u32 {
        let positions = extend_wire(wire1);

        let mut x = 0i32;
        let mut y = 0i32;
        let mut best_distance = u32::MAX;

        let mut recompute = |x, y| match positions.get(&(x, y)) {
            None => {}
            Some(_) => {
                let new_distance = (i32::abs(x) + y.abs()) as u32;
                if new_distance > 0 && new_distance < best_distance {
                    best_distance = new_distance;
                }
            }
        };

        for mov in wire2 {
            match mov.dir {
                Direction::Up => {
                    for y in y..y + mov.distance as i32 {
                        recompute(x, y);
                    }
                    y += mov.distance as i32;
                }
                Direction::Down => {
                    for y in y - (mov.distance as i32) + 1..=y {
                        recompute(x, y);
                    }
                    y -= mov.distance as i32;
                }
                Direction::Left => {
                    for x in x - (mov.distance as i32) + 1..=x {
                        recompute(x, y);
                    }
                    x -= mov.distance as i32;
                }
                Direction::Right => {
                    for x in x..x + mov.distance as i32 {
                        recompute(x, y);
                    }
                    x += mov.distance as i32;
                }
            }
        }

        best_distance
    }

    pub fn part_2(wire1: &[Move], wire2: &[Move]) -> u32 {
        let positions = extend_wire(wire1);

        let mut x = 0i32;
        let mut y = 0i32;
        let mut steps = 0u32;
        let mut best_steps = u32::MAX;

        let mut recompute = |x, y, step| match positions.get(&(x, y)) {
            None => {}
            Some(&s2) => {
                let new_steps = s2 + step;
                if x != 0 && y != 0 && new_steps < best_steps {
                    best_steps = new_steps;
                }
            }
        };

        for mov in wire2 {
            match mov.dir {
                Direction::Up => {
                    for i in 0..mov.distance {
                        recompute(x, y + i as i32, steps + i);
                    }
                    y += mov.distance as i32;
                }
                Direction::Down => {
                    for i in 0..mov.distance {
                        recompute(x, y - i as i32, steps + i);
                    }
                    y -= mov.distance as i32;
                }
                Direction::Left => {
                    for i in 0..mov.distance {
                        recompute(x - i as i32, y, steps + i);
                    }
                    x -= mov.distance as i32;
                }
                Direction::Right => {
                    for i in 0..mov.distance {
                        recompute(x + i as i32, y, steps + i);
                    }
                    x += mov.distance as i32;
                }
            }
            steps += mov.distance;
        }

        best_steps
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
        {
            let (wire1, wire2) = input("R8,U5,L5,D3\nU7,R6,D4,L4");
            assert_eq!(part_2(&wire1, &wire2), 30);
        }
        {
            let (wire1, wire2) =
                input("R75,D30,R83,U83,L12,D49,R71,U7,L72\nU62,R66,U55,R34,D71,R55,D58,R83");
            assert_eq!(part_2(&wire1, &wire2), 610);
        }
        {
            let (wire1, wire2) = input(
                "R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51\nU98,R91,D20,R16,D67,R40,U7,R15,U6,R7",
            );
            assert_eq!(part_2(&wire1, &wire2), 410);
        }
    }

    #[test]
    #[cfg(not(feature = "no_real_inputs"))]
    fn test_day_3() {
        let (wire1, wire2) = input(include_str!("../input.txt"));
        assert_eq!(part_1(&wire1, &wire2), 225);
        assert_eq!(part_2(&wire1, &wire2), 35194);
    }
}
