pub mod day_6 {
    use std::collections::{HashMap, HashSet};
    use std::hash::Hash;

    pub struct Edge<T> {
        source: T,
        dest: T,
    }

    struct Tree<Label> {
        arena: Vec<(Label, Vec<usize>)>,
        lookup: HashMap<Label, usize>,
        root: Label,
    }

    #[derive(Debug)]
    enum DagConstructionError {
        MultipleRoots,
        Cycle,
    }

    impl<Label> Tree<Label> {
        fn make(inputs: &[Edge<Label>]) -> Result<Tree<Label>, DagConstructionError>
        where
            Label: Copy + Eq + Hash,
        {
            let mut arena: Vec<(Label, Vec<usize>)> = Vec::with_capacity(inputs.len());
            let mut lookup: HashMap<Label, usize> = HashMap::with_capacity(inputs.len());
            let mut roots: HashSet<Label> = inputs
                .iter()
                .flat_map(|edge| [edge.source, edge.dest])
                .collect();

            for edge in inputs {
                roots.remove(&edge.dest);
                let source_index = *lookup.entry(edge.source).or_insert_with(|| {
                    arena.push((edge.source, vec![]));
                    arena.len() - 1
                });
                let dest_index = *lookup.entry(edge.dest).or_insert_with(|| {
                    arena.push((edge.dest, vec![]));
                    arena.len() - 1
                });

                let (_, ref mut entry) = &mut arena[source_index];
                entry.push(dest_index);
            }

            if roots.len() > 1 {
                return Err(DagConstructionError::MultipleRoots);
            }

            match roots.iter().next() {
                None => Err(DagConstructionError::Cycle),
                Some(root) => Ok(Tree {
                    arena,
                    lookup,
                    root: *root,
                }),
            }
        }

        fn cata_inner<F, Ret>(self: &Tree<Label>, depth: u32, node: usize, f: &mut F) -> Ret
        where
            F: FnMut(u32, &Label, &[Ret]) -> Ret,
            Label: Hash + Eq,
        {
            let (label, children) = &self.arena[node];
            let child_results: Vec<_> = children
                .iter()
                .map(|child| self.cata_inner(depth + 1, *child, f))
                .collect();
            f(depth, label, &child_results)
        }

        /*
         We give you the depth you're at, as well. The root is at depth 0.
        */
        fn cata<F, Ret>(self: &Tree<Label>, f: &mut F) -> Ret
        where
            F: FnMut(u32, &Label, &[Ret]) -> Ret,
            Label: Hash + Eq,
        {
            let root = *self.lookup.get(&self.root).unwrap();
            self.cata_inner(0, root, f)
        }
    }

    pub fn input(s: &str) -> Vec<Edge<&str>> {
        s.trim()
            .split('\n')
            .map(|l| {
                let mut iter = l.split(')');
                Edge {
                    source: iter.next().unwrap(),
                    dest: iter.next().unwrap(),
                }
            })
            .collect()
    }

    pub fn part_1(input: &[Edge<&str>]) -> u32 {
        let dag = Tree::make(input).unwrap();
        dag.cata(&mut |depth, _node, children| {
            children.iter().copied().map(|x| x + depth + 1).sum::<u32>()
        })
    }

    pub fn part_2(input: &[Edge<&str>]) -> u32 {
        let dag = Tree::make(input);
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
    fn test_part2_known() {
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
        assert_eq!(part_2(&input), 4);
    }

    #[test]
    #[cfg(not(feature = "no_real_inputs"))]
    fn test_day_6() {
        let input = input(include_str!("../input.txt"));
        assert_eq!(part_1(&input), 249308);
        //assert_eq!(part_2(&input), 349);
    }
}
