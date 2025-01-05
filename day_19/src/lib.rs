pub mod day_19 {
    use intcode::intcode::{MachineExecutionError, MachineState, Num};
    use std::{
        collections::HashSet,
        fmt::Display,
        ops::{Add, Mul},
        rc::Rc,
    };

    #[derive(Clone, Debug)]
    enum Ast {
        Constant(i32),
        Zero,
        One,
        AddNode(Box<Ast>, Box<Ast>),
        MulNode(Box<Ast>, Box<Ast>),
        IfEqThen(Box<Ast>, Box<Ast>, Box<Ast>, Box<Ast>),
        IfLessThen(Box<Ast>, Box<Ast>, Box<Ast>, Box<Ast>),
        Variable(char),
    }

    pub struct List<T> {
        head: Link<T>,
    }

    type Link<T> = Option<Rc<Node<T>>>;

    struct Node<T> {
        elem: T,
        next: Link<T>,
    }

    impl<T> List<T> {
        pub fn new() -> Self {
            List { head: None }
        }
        pub fn prepend(&self, elem: T) -> List<T> {
            List {
                head: Some(Rc::new(Node {
                    elem: elem,
                    next: self.head.clone(),
                })),
            }
        }
        pub fn tail(&self) -> List<T> {
            List {
                head: self.head.as_ref().and_then(|node| node.next.clone()),
            }
        }
        pub fn head(&self) -> Option<&T> {
            self.head.as_ref().map(|node| &node.elem)
        }
    }
    pub struct Iter<'a, T> {
        next: Option<&'a Node<T>>,
    }

    impl<T> List<T> {
        pub fn iter(&self) -> Iter<'_, T> {
            Iter {
                next: self.head.as_deref(),
            }
        }
    }

    impl<'a, T> Iterator for Iter<'a, T> {
        type Item = &'a T;

        fn next(&mut self) -> Option<Self::Item> {
            self.next.map(|node| {
                self.next = node.next.as_deref();
                &node.elem
            })
        }
    }

    enum Condition {
        LessThan(Box<Ast>, Box<Ast>),
        Equal(Box<Ast>, Box<Ast>),
        NotEqual(Box<Ast>, Box<Ast>),
        NotLess(Box<Ast>, Box<Ast>),
    }

    impl Display for Ast {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Ast::Constant(i) => f.write_str(&format!("{}", i)),
                Ast::Zero => f.write_str("0"),
                Ast::One => f.write_str("1"),
                Ast::AddNode(ast, ast1) => {
                    f.write_str("(")?;
                    ast.fmt(f)?;
                    f.write_str(" + ")?;
                    ast1.fmt(f)?;
                    f.write_str(")")
                }
                Ast::MulNode(ast, ast1) => {
                    f.write_str("(")?;
                    ast.fmt(f)?;
                    f.write_str(" * ")?;
                    ast1.fmt(f)?;
                    f.write_str(")")
                }
                Ast::IfEqThen(ast, ast1, ast2, ast3) => {
                    f.write_str("If[")?;
                    ast.fmt(f)?;
                    f.write_str(" == ")?;
                    ast1.fmt(f)?;
                    f.write_str(", \n")?;
                    ast2.fmt(f)?;
                    f.write_str(", \n")?;
                    ast3.fmt(f)?;
                    f.write_str("]")
                }
                Ast::IfLessThen(ast, ast1, ast2, ast3) => {
                    f.write_str("If[")?;
                    ast.fmt(f)?;
                    f.write_str(" < ")?;
                    ast1.fmt(f)?;
                    f.write_str(", \n")?;
                    ast2.fmt(f)?;
                    f.write_str(", \n")?;
                    ast3.fmt(f)?;
                    f.write_str("]")
                }
                Ast::Variable(x) => f.write_str(&format!("{x}")),
            }
        }
    }

    impl Ast {
        fn strict_equal(&self, other: &Ast) -> bool {
            match (self, other) {
                (Ast::Constant(a), Ast::Constant(b)) => *a == *b,
                (Ast::Constant(a), Ast::Zero) | (Ast::Zero, Ast::Constant(a)) => *a == 0,
                (Ast::Constant(a), Ast::One) | (Ast::One, Ast::Constant(a)) => *a == 1,
                (Ast::Zero, Ast::Zero) => true,
                (Ast::Zero, Ast::One) => false,
                (Ast::One, Ast::Zero) => false,
                (Ast::One, Ast::One) => true,
                (Ast::AddNode(a, b), Ast::AddNode(a2, b2)) => {
                    a.strict_equal(a2) && b.strict_equal(b2)
                }
                (Ast::MulNode(a, b), Ast::MulNode(a2, b2)) => {
                    a.strict_equal(a2) && b.strict_equal(b2)
                }
                (Ast::IfEqThen(a, b, c, d), Ast::IfEqThen(a2, b2, c2, d2)) => {
                    a.strict_equal(a2)
                        && b.strict_equal(b2)
                        && c.strict_equal(c2)
                        && d.strict_equal(d2)
                }
                (Ast::IfLessThen(a, b, c, d), Ast::IfLessThen(a2, b2, c2, d2)) => {
                    a.strict_equal(a2)
                        && b.strict_equal(b2)
                        && c.strict_equal(c2)
                        && d.strict_equal(d2)
                }
                (Ast::Variable(a), Ast::Variable(b)) => *a == *b,
                (_, _) => false,
            }
        }

        fn eval<F>(&self, var: &mut F) -> Result<i32, char>
        where
            F: FnMut(char) -> Option<i32>,
        {
            match self {
                Ast::Constant(i) => Ok(*i),
                Ast::Zero => Ok(0),
                Ast::One => Ok(1),
                Ast::Variable(c) => match var(*c) { None => Err(*c), Some(x) => Ok(x) },
                Ast::AddNode(x, y) => Ok(x.eval(var)? + y.eval(var)?),
                Ast::MulNode(x, y) => Ok(x.eval(var)? * y.eval(var)?),
                Ast::IfEqThen(us, other, eq_res, neq_res) => {
                    if us.eval(var) == other.eval(var) {
                        eq_res.eval(var)
                    } else {
                        neq_res.eval(var)
                    }
                }
                Ast::IfLessThen(us, other, eq_res, neq_res) => {
                    if us.eval(var) < other.eval(var) {
                        eq_res.eval(var)
                    } else {
                        neq_res.eval(var)
                    }
                }
            }
        }

        fn simplify(&self, conditions: &List<Condition>) -> Ast {
            match self {
                Ast::Constant(i) => Ast::Constant(*i),
                Ast::Zero => Ast::Zero,
                Ast::One => Ast::One,
                Ast::Variable(c) => Ast::Variable(*c),
                Ast::IfEqThen(a, b, eq_res, neq_res) => {
                    let a = a.simplify(conditions);
                    let b = b.simplify(conditions);
                    for cond in conditions.iter() {
                        match cond {
                            Condition::NotEqual(v1, v2) => {
                                if a.strict_equal(v1) && b.strict_equal(v2) {
                                    return neq_res.simplify(conditions);
                                }
                            }
                            Condition::Equal(v1, v2) => {
                                if a.strict_equal(v1) && b.strict_equal(v2) {
                                    return eq_res.simplify(conditions);
                                }
                            }
                            Condition::LessThan(v1, v2) => {
                                if a.strict_equal(v1) && b.strict_equal(v2) {
                                    return neq_res.simplify(conditions);
                                }
                            }
                            _ => {}
                        }
                    }
                    match (a, b) {
                        (Ast::Zero, Ast::Zero) => eq_res.simplify(conditions),
                        (Ast::Constant(a), Ast::Constant(b)) => {
                            if a == b {
                                eq_res.simplify(conditions)
                            } else {
                                neq_res.simplify(conditions)
                            }
                        }
                        (Ast::Constant(0), Ast::Zero) => eq_res.simplify(conditions),
                        (Ast::Constant(_), Ast::Zero) => neq_res.simplify(conditions),
                        (Ast::Constant(1), Ast::One) => eq_res.simplify(conditions),
                        (Ast::Constant(_), Ast::One) => neq_res.simplify(conditions),
                        (Ast::Zero, Ast::Constant(0)) => eq_res.simplify(conditions),
                        (Ast::Zero, Ast::Constant(_)) => neq_res.simplify(conditions),
                        (Ast::Zero, Ast::One) => neq_res.simplify(conditions),
                        (Ast::One, Ast::Constant(1)) => eq_res.simplify(conditions),
                        (Ast::One, Ast::Constant(_)) => neq_res.simplify(conditions),
                        (Ast::One, Ast::Zero) => neq_res.simplify(conditions),
                        (Ast::One, Ast::One) => eq_res.simplify(conditions),
                        (Ast::Variable(x), Ast::Variable(y)) => {
                            if x == y {
                                eq_res.simplify(conditions)
                            } else {
                                Ast::IfEqThen(
                                    Box::new(Ast::Variable(x)),
                                    Box::new(Ast::Variable(y)),
                                    Box::new(eq_res.simplify(&conditions.prepend(
                                        Condition::Equal(
                                            Box::new(Ast::Variable(x)),
                                            Box::new(Ast::Variable(y)),
                                        ),
                                    ))),
                                    Box::new(neq_res.simplify(&conditions.prepend(
                                        Condition::NotEqual(
                                            Box::new(Ast::Variable(x)),
                                            Box::new(Ast::Variable(y)),
                                        ),
                                    ))),
                                )
                            }
                        }
                        (a, b) => {
                            let a = Box::new(a);
                            let b = Box::new(b);
                            Ast::IfEqThen(
                                a.clone(),
                                b.clone(),
                                Box::new(eq_res.simplify(
                                    &conditions.prepend(Condition::Equal(a.clone(), b.clone())),
                                )),
                                Box::new(
                                    neq_res
                                        .simplify(&conditions.prepend(Condition::NotEqual(a, b))),
                                ),
                            )
                        }
                    }
                }
                Ast::IfLessThen(a, b, if_less, if_geq) => {
                    let a = a.simplify(conditions);
                    let b = b.simplify(conditions);
                    for cond in conditions.iter() {
                        match cond {
                            Condition::Equal(v1, v2) => {
                                if a.strict_equal(v1) && b.strict_equal(v2) {
                                    return if_geq.simplify(conditions);
                                }
                            }
                            Condition::LessThan(v1, v2) => {
                                if a.strict_equal(v1) && b.strict_equal(v2) {
                                    return if_less.simplify(conditions);
                                }
                            }
                            Condition::NotLess(v1, v2) => {
                                if a.strict_equal(v1) && b.strict_equal(v2) {
                                    return if_geq.simplify(conditions);
                                }
                            }
                            _ => {}
                        }
                    }
                    match (a, b) {
                        (a, b) => Ast::IfLessThen(
                            Box::new(a.clone()),
                            Box::new(b.clone()),
                            Box::new(if_less.simplify(&conditions.prepend(Condition::LessThan(
                                Box::new(a.clone()),
                                Box::new(b.clone()),
                            )))),
                            Box::new(if_geq.simplify(
                                &conditions.prepend(Condition::NotLess(Box::new(a), Box::new(b))),
                            )),
                        ),
                    }
                }
                Ast::AddNode(ast, ast1) => {
                    match (ast.simplify(conditions), ast1.simplify(conditions)) {
                        (Ast::Constant(0), a) => a,
                        (Ast::Zero, a) => a,
                        (a, Ast::Constant(0)) => a,
                        (a, Ast::Zero) => a,
                        (Ast::Constant(a), Ast::Constant(b)) => Ast::Constant(a + b),
                        (Ast::Constant(a), Ast::One) => Ast::Constant(a + 1),
                        (Ast::Constant(a), Ast::Variable(x)) => {
                            Ast::AddNode(Box::new(Ast::Constant(a)), Box::new(Ast::Variable(x)))
                        }
                        (Ast::One, Ast::Constant(a)) => Ast::Constant(a + 1),
                        (Ast::One, Ast::One) => Ast::Constant(2),
                        (Ast::One, Ast::Variable(a)) => {
                            Ast::AddNode(Box::new(Ast::One), Box::new(Ast::Variable(a)))
                        }
                        (Ast::Variable(a), b) => {
                            Ast::AddNode(Box::new(b), Box::new(Ast::Variable(a)))
                        }
                        (Ast::Constant(v), Ast::AddNode(ast, ast1)) => Ast::AddNode(
                            Box::new(
                                Ast::AddNode(Box::new(Ast::Constant(v)), ast).simplify(conditions),
                            ),
                            ast1,
                        ),
                        (Ast::Constant(c), Ast::MulNode(ast, ast1)) => Ast::AddNode(
                            Box::new(Ast::Constant(c)),
                            Box::new(Ast::MulNode(ast, ast1)),
                        ),
                        (Ast::One, Ast::AddNode(ast, ast1)) => Ast::AddNode(
                            Box::new(Ast::AddNode(Box::new(Ast::One), ast).simplify(conditions)),
                            ast1,
                        ),
                        (Ast::One, Ast::MulNode(ast, ast1)) => {
                            Ast::AddNode(Box::new(Ast::One), Box::new(Ast::MulNode(ast, ast1)))
                        }
                        (Ast::AddNode(ast, ast1), other) => {
                            Ast::AddNode(ast, Box::new(Ast::AddNode(ast1, Box::new(other)).simplify(conditions)))
                        }
                        (Ast::IfLessThen(a, b, if_less, if_not_less), Ast::Constant(c))
                        | (Ast::Constant(c), Ast::IfLessThen(a, b, if_less, if_not_less)) => {
                            Ast::IfLessThen(
                                a.clone(),
                                b.clone(),
                                Box::new(
                                    Ast::AddNode(if_less, Box::new(Ast::Constant(c))).simplify(
                                        &conditions
                                            .prepend(Condition::LessThan(a.clone(), b.clone())),
                                    ),
                                ),
                                Box::new(
                                    Ast::AddNode(if_not_less, Box::new(Ast::Constant(c))).simplify(
                                        &conditions
                                            .prepend(Condition::NotLess(a.clone(), b.clone())),
                                    ),
                                ),
                            )
                        }
                        (a, b) => Ast::AddNode(Box::new(a), Box::new(b)),
                    }
                }
                Ast::MulNode(ast, ast1) => {
                    match (ast.simplify(conditions), ast1.simplify(conditions)) {
                        (_, Ast::Zero) => Ast::Zero,
                        (_, Ast::Constant(0)) => Ast::Zero,
                        (Ast::Constant(0), _) => Ast::Zero,
                        (Ast::Zero, _) => Ast::Zero,
                        (Ast::Constant(1), a) => a,
                        (Ast::One, a) => a,
                        (a, Ast::One) => a,
                        (a, Ast::Constant(1)) => a,
                        (Ast::Constant(a), Ast::Constant(b)) => Ast::Constant(a * b),
                        (Ast::Constant(v), Ast::Variable(x))
                        | (Ast::Variable(x), Ast::Constant(v)) => {
                            Ast::MulNode(Box::new(Ast::Constant(v)), Box::new(Ast::Variable(x)))
                        }
                        (a, Ast::Constant(x)) => {
                            Ast::MulNode(Box::new(Ast::Constant(x)), Box::new(a))
                                .simplify(conditions)
                        }
                        (Ast::Constant(x), Ast::AddNode(ast, ast1)) => Ast::AddNode(
                            Box::new(Ast::MulNode(Box::new(Ast::Constant(x)), ast)),
                            Box::new(Ast::MulNode(Box::new(Ast::Constant(x)), ast1)),
                        )
                        .simplify(&conditions),
                        (Ast::Variable(v), Ast::Variable(w)) => {
                            Ast::MulNode(Box::new(Ast::Variable(v)), Box::new(Ast::Variable(w)))
                        }
                        (Ast::Constant(x), Ast::MulNode(a, b)) => Ast::MulNode(
                            Box::new(
                                Ast::MulNode(Box::new(Ast::Constant(x)), a).simplify(conditions),
                            ),
                            b,
                        ),
                        (Ast::IfLessThen(a, b, if_less, if_not_less), other)
                        | (other, Ast::IfLessThen(a, b, if_less, if_not_less)) => Ast::IfLessThen(
                            a.clone(),
                            b.clone(),
                            Box::new(Ast::MulNode(if_less, Box::new(other.clone())).simplify(
                                &conditions.prepend(Condition::LessThan(a.clone(), b.clone())),
                            )),
                            Box::new(
                                Ast::MulNode(if_not_less, Box::new(other))
                                    .simplify(&conditions.prepend(Condition::NotLess(a, b))),
                            ),
                        ),
                        (a, b) => Ast::MulNode(Box::new(a), Box::new(b)),
                    }
                }
            }
        }
    }

    impl PartialEq for Ast {
        fn eq(&self, other: &Self) -> bool {
            match self.eval(&mut |_| None) {
                Err(v) => panic!("{v}"),
                Ok(i) => {
                    match other.eval(&mut |_| None) {
                        Err(v) => panic!("{v}"),
                        Ok(j) => i == j
                    }
                }
            }
        }
    }

    impl Eq for Ast {}

    impl PartialOrd for Ast {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            match self.eval(&mut |_| None) {
                Err(v) => panic!("{v}"),
                Ok(i) => {
                    match other.eval(&mut |_| None) {
                        Err(v) => panic!("{v}"),
                        Ok(j) => Some(i.cmp(&j))
                    }
                }
            }
        }
    }

    impl Ord for Ast {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            match self.eval(&mut |_| None) {
                Err(v) => panic!("{v}"),
                Ok(i) => {
                    match other.eval(&mut |_| None) {
                        Err(v) => panic!("{v}"),
                        Ok(j) => i.cmp(&j)
                    }
                }
            }
        }
    }

    impl Add for Ast {
        type Output = Ast;

        fn add(self, rhs: Ast) -> Self::Output {
            Ast::AddNode(Box::new(self), Box::new(rhs))
        }
    }

    impl Mul for Ast {
        type Output = Ast;

        fn mul(self, rhs: Self) -> Self::Output {
            Ast::MulNode(Box::new(self), Box::new(rhs))
        }
    }

    impl Num for Ast {
        fn zero() -> Self {
            Ast::Zero
        }

        fn one() -> Self {
            Ast::One
        }

        fn to_usize(self) -> Option<usize> {
            match self.eval(&mut |_| None) {
                Err(_) => None,
                Ok(eval) => {
            if eval < 0 {
                None
            } else {
                Some(eval as usize)
            }
        }}
        }

        fn to_i32(self) -> Option<i32> {
            match self.eval(&mut |_| None) {
                Err(_) => None,
                Ok(eval) => Some(eval)
            }
        }

        fn if_less_then_else(self, other: Self, if_less: Self, if_not_less: Self) -> Self {
            Ast::IfLessThen(
                Box::new(self),
                Box::new(other),
                Box::new(if_less),
                Box::new(if_not_less),
            )
        }

        fn if_eq_then_else(self, other: Self, if_eq: Self, if_neq: Self) -> Self {
            Ast::IfEqThen(
                Box::new(self),
                Box::new(other),
                Box::new(if_eq),
                Box::new(if_neq),
            )
        }
    }

    pub fn input(s: &str) -> Vec<i32> {
        s.trim()
            .split(',')
            .map(|l| str::parse(l).unwrap())
            .collect()
    }

    fn query_machine<T>(
        machine: &mut MachineState<T>,
        x: T,
        y: T,
    ) -> Result<T, MachineExecutionError>
    where
        T: Add<T, Output = T> + Mul<T, Output = T> + Copy + Ord + Num,
    {
        match machine.execute_until_input()? {
            intcode::intcode::StepIoResult::Terminated => {
                panic!("Unexpectedly terminated");
            }
            intcode::intcode::StepIoResult::Output(_) => {
                panic!("Unexpectedly terminated");
            }
            intcode::intcode::StepIoResult::AwaitingInput(v) => {
                machine.set_mem_elt(v, x);
            }
        }
        match machine.execute_until_input()? {
            intcode::intcode::StepIoResult::Terminated => {
                panic!("Unexpectedly terminated");
            }
            intcode::intcode::StepIoResult::Output(_) => {
                panic!("Unexpectedly terminated");
            }
            intcode::intcode::StepIoResult::AwaitingInput(v) => {
                machine.set_mem_elt(v, y);
            }
        }
        let v = match machine.execute_until_input()? {
            intcode::intcode::StepIoResult::Terminated => {
                panic!("Unexpectedly terminated");
            }
            intcode::intcode::StepIoResult::Output(v) => v,
            intcode::intcode::StepIoResult::AwaitingInput(_) => {
                panic!("Unexpectedly asked for input")
            }
        };
        Ok(v)
    }

    // Returns the first x for which (x, y) is 1, and perhaps also a higher x for which (x, y) is also known to be 1.
    fn find_lower_boundary(
        machine: &mut MachineState<i32>,
        y: i32,
    ) -> Result<(i32, Option<i32>), MachineExecutionError> {
        if query_machine(machine, 0, y)? == 1 {
            return Ok((0, None));
        }

        if query_machine(machine, 1, y)? == 1 {
            return Ok((1, None));
        }

        let mut lower_guess = 2;

        let known_upper_is_one = loop {
            let query_result = query_machine(machine, lower_guess, y)?;
            if query_result == 0 {
                lower_guess *= 2;
            } else {
                break lower_guess;
            }
        };

        let mut upper_is_one = known_upper_is_one;
        let mut lower_is_zero = upper_is_one / 2;

        // Loop invariant: upper_is_one is known to be 1 and known_upper / 2 is known to be 0.
        while lower_is_zero + 1 < upper_is_one {
            let midpoint = (upper_is_one - lower_is_zero) / 2 + lower_is_zero;
            // midpoint > lower_is_zero, because upper_is_one - lower_is_zero >= 2 due to the `while` condition.
            let query_result = query_machine(machine, midpoint, y)?;
            if query_result == 0 {
                lower_is_zero = query_result;
            } else {
                upper_is_one = query_result;
            }
        }

        Ok((upper_is_one, Some(known_upper_is_one)))
    }

    pub fn part_1(input: &[i32]) -> Result<u32, MachineExecutionError> {
        let mut machine = MachineState::new_with_memory(&input.iter().copied());
        let mut result = 0u32;
        for y in 0..=49 {
            for x in 0..=49 {
                machine.reset(input.iter().copied());
                let query_result = query_machine(&mut machine, x, y)?;
                result += query_result as u32
            }
        }
        Ok(result)
    }

    pub fn part_2(input: &[i32]) -> Result<i32, MachineExecutionError> {
        let mut machine =
            MachineState::new_with_memory(&input.iter().copied().map(|x| Ast::Constant(x)));
        match machine.execute_until_input()? {
            intcode::intcode::StepIoResult::Terminated => {
                panic!("terminated unexpectedly");
            }
            intcode::intcode::StepIoResult::Output(_) => {
                panic!("unexpectedly output");
            }
            intcode::intcode::StepIoResult::AwaitingInput(loc) => {
                machine.set_mem_elt(loc, Ast::Variable('x'));
            }
        };
        match machine.execute_until_input()? {
            intcode::intcode::StepIoResult::Terminated => {
                panic!("terminated unexpectedly");
            }
            intcode::intcode::StepIoResult::Output(_) => {
                panic!("unexpectedly output");
            }
            intcode::intcode::StepIoResult::AwaitingInput(loc) => {
                machine.set_mem_elt(loc, Ast::Variable('y'));
            }
        };
        let output = match machine.execute_until_input()? {
            intcode::intcode::StepIoResult::Terminated => {
                panic!("terminated unexpectedly");
            }
            intcode::intcode::StepIoResult::AwaitingInput(_) => {
                panic!("unexpectedly asked for input");
            }
            intcode::intcode::StepIoResult::Output(ast) => ast,
        };
        let mut m = HashSet::new();
        println!(
            "{}",
            output.simplify(
                &List::new()
                    .prepend(Condition::LessThan(
                        Box::new(Ast::Zero),
                        Box::new(Ast::Variable('y'))
                    ))
                    .prepend(Condition::LessThan(
                        Box::new(Ast::Zero),
                        Box::new(Ast::Variable('x'))
                    ))
            )
            .simplify(
                &List::new()
                    .prepend(Condition::LessThan(
                        Box::new(Ast::Zero),
                        Box::new(Ast::Variable('y'))
                    ))
                    .prepend(Condition::LessThan(
                        Box::new(Ast::Zero),
                        Box::new(Ast::Variable('x'))
                    ))
                )
            .simplify(
                &List::new()
                    .prepend(Condition::LessThan(
                        Box::new(Ast::Zero),
                        Box::new(Ast::Variable('y'))
                    ))
                    .prepend(Condition::LessThan(
                        Box::new(Ast::Zero),
                        Box::new(Ast::Variable('x'))
                    ))
            ).simplify(&List::new())
        );
        println!(
            "{:?}",
            output.eval(&mut |c| {
                m.insert(c);
                Some(0)
            })
        );
        panic!("Asked for: {:?}", m)
    }
}

#[cfg(test)]
mod tests {
    use super::day_19::*;

    #[test]
    #[cfg(not(feature = "no_real_inputs"))]
    fn test_day_19() {
        let input = input(include_str!("../input.txt"));
        assert_eq!(part_1(&input).unwrap(), 226);
        assert_eq!(part_2(&input).unwrap(), 18509);
    }
}
