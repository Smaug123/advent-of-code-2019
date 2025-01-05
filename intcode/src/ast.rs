use std::{
    fmt::Display,
    ops::{Add, Mul},
};

use crate::{intcode::Num, linked_list::List};

#[derive(Clone, Debug)]
pub enum Ast {
    Constant(i64),
    Zero,
    One,
    AddNode(Box<Ast>, Box<Ast>),
    MulNode(Box<Ast>, Box<Ast>),
    IfEqThen(Box<Ast>, Box<Ast>, Box<Ast>, Box<Ast>),
    IfLessThen(Box<Ast>, Box<Ast>, Box<Ast>, Box<Ast>),
    Variable(char),
}

pub enum Condition {
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
            (Ast::AddNode(a, b), Ast::AddNode(a2, b2)) => a.strict_equal(a2) && b.strict_equal(b2),
            (Ast::MulNode(a, b), Ast::MulNode(a2, b2)) => a.strict_equal(a2) && b.strict_equal(b2),
            (Ast::IfEqThen(a, b, c, d), Ast::IfEqThen(a2, b2, c2, d2)) => {
                a.strict_equal(a2) && b.strict_equal(b2) && c.strict_equal(c2) && d.strict_equal(d2)
            }
            (Ast::IfLessThen(a, b, c, d), Ast::IfLessThen(a2, b2, c2, d2)) => {
                a.strict_equal(a2) && b.strict_equal(b2) && c.strict_equal(c2) && d.strict_equal(d2)
            }
            (Ast::Variable(a), Ast::Variable(b)) => *a == *b,
            (_, _) => false,
        }
    }

    /// Evaluate the AST with the given mapping of variable name to value.
    /// Returns Err(var) if we need to evaluate a variable which hasn't been given a value.
    pub fn eval<F>(&self, var: &mut F) -> Result<i64, char>
    where
        F: FnMut(char) -> Option<i64>,
    {
        match self {
            Ast::Constant(i) => Ok(*i),
            Ast::Zero => Ok(0),
            Ast::One => Ok(1),
            Ast::Variable(c) => match var(*c) {
                None => Err(*c),
                Some(x) => Ok(x),
            },
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

    /// Perform heuristic algebraic manipulations to simplify this AST under the given assumptions.
    pub fn simplify(&self, conditions: &List<Condition>) -> Ast {
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
                                Box::new(eq_res.simplify(&conditions.prepend(Condition::Equal(
                                    Box::new(Ast::Variable(x)),
                                    Box::new(Ast::Variable(y)),
                                )))),
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
                                neq_res.simplify(&conditions.prepend(Condition::NotEqual(a, b))),
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
                Ast::IfLessThen(
                    Box::new(a.clone()),
                    Box::new(b.clone()),
                    Box::new(if_less.simplify(&conditions.prepend(Condition::LessThan(
                        Box::new(a.clone()),
                        Box::new(b.clone()),
                    )))),
                    Box::new(if_geq.simplify(
                        &conditions.prepend(Condition::NotLess(Box::new(a), Box::new(b))),
                    )),
                )
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
                    (Ast::Variable(a), b) => Ast::AddNode(Box::new(b), Box::new(Ast::Variable(a))),
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
                    (Ast::AddNode(ast, ast1), other) => Ast::AddNode(
                        ast,
                        Box::new(Ast::AddNode(ast1, Box::new(other)).simplify(conditions)),
                    ),
                    (Ast::IfLessThen(a, b, if_less, if_not_less), Ast::Constant(c))
                    | (Ast::Constant(c), Ast::IfLessThen(a, b, if_less, if_not_less)) => {
                        Ast::IfLessThen(
                            a.clone(),
                            b.clone(),
                            Box::new(Ast::AddNode(if_less, Box::new(Ast::Constant(c))).simplify(
                                &conditions.prepend(Condition::LessThan(a.clone(), b.clone())),
                            )),
                            Box::new(
                                Ast::AddNode(if_not_less, Box::new(Ast::Constant(c))).simplify(
                                    &conditions.prepend(Condition::NotLess(a.clone(), b.clone())),
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
                    (Ast::Constant(v), Ast::Variable(x)) | (Ast::Variable(x), Ast::Constant(v)) => {
                        Ast::MulNode(Box::new(Ast::Constant(v)), Box::new(Ast::Variable(x)))
                    }
                    (a, Ast::Constant(x)) => {
                        Ast::MulNode(Box::new(Ast::Constant(x)), Box::new(a)).simplify(conditions)
                    }
                    (Ast::Constant(x), Ast::AddNode(ast, ast1)) => Ast::AddNode(
                        Box::new(Ast::MulNode(Box::new(Ast::Constant(x)), ast)),
                        Box::new(Ast::MulNode(Box::new(Ast::Constant(x)), ast1)),
                    )
                    .simplify(conditions),
                    (Ast::Variable(v), Ast::Variable(w)) => {
                        Ast::MulNode(Box::new(Ast::Variable(v)), Box::new(Ast::Variable(w)))
                    }
                    (Ast::Constant(x), Ast::MulNode(a, b)) => Ast::MulNode(
                        Box::new(Ast::MulNode(Box::new(Ast::Constant(x)), a).simplify(conditions)),
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
            Ok(i) => match other.eval(&mut |_| None) {
                Err(v) => panic!("{v}"),
                Ok(j) => i == j,
            },
        }
    }
}

impl Eq for Ast {}

impl PartialOrd for Ast {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Ast {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.eval(&mut |_| None) {
            Err(v) => panic!("{v}"),
            Ok(i) => match other.eval(&mut |_| None) {
                Err(v) => panic!("{v}"),
                Ok(j) => i.cmp(&j),
            },
        }
    }
}

impl Add for Ast {
    type Output = Ast;

    fn add(self, rhs: Ast) -> Self::Output {
        match (self, rhs) {
            (Ast::Zero, x) | (x, Ast::Zero) | (Ast::Constant(0), x) | (x, Ast::Constant(0)) => x,
            (Ast::Constant(a), Ast::Constant(b)) => Ast::Constant(a + b),
            (Ast::AddNode(a, b), c) => a.add(b.add(c)),
            (Ast::Constant(c), Ast::IfLessThen(x, y, lt_res, geq_res))
            | (Ast::IfLessThen(x, y, lt_res, geq_res), Ast::Constant(c)) => Ast::IfLessThen(
                x,
                y,
                Box::new(lt_res.add(Ast::Constant(c))),
                Box::new(geq_res.add(Ast::Constant(c))),
            ),
            (Ast::Constant(a), Ast::AddNode(b, c)) => match *b {
                Ast::Constant(b) => Ast::Constant(a + b).add(*c),
                b => Ast::AddNode(
                    Box::new(Ast::Constant(a)),
                    Box::new(Ast::AddNode(Box::new(b), c)),
                ),
            },
            (
                Ast::IfLessThen(x, y, lt_res, geq_res),
                Ast::IfLessThen(x2, y2, lt_res2, geq_res2),
            ) => {
                if x.strict_equal(&x2) && y.strict_equal(&y2) {
                    Ast::IfLessThen(
                        x,
                        y,
                        Box::new(lt_res.add(*lt_res2)),
                        Box::new(geq_res.add(*geq_res2)),
                    )
                } else {
                    Ast::AddNode(
                        Box::new(Ast::IfLessThen(x, y, lt_res, geq_res)),
                        Box::new(Ast::IfLessThen(x2, y2, lt_res2, geq_res2)),
                    )
                }
            }
            (x, Ast::MulNode(y, c)) => match *c {
                Ast::Constant(-1) => {
                    if x.strict_equal(&y) {
                        Ast::Zero
                    } else {
                        Ast::AddNode(
                            Box::new(x),
                            Box::new(Ast::MulNode(y, Box::new(Ast::Constant(-1)))),
                        )
                    }
                }
                _ => Ast::AddNode(Box::new(x), Box::new(Ast::MulNode(y, c))),
            },
            (x, y) => Ast::AddNode(Box::new(x), Box::new(y)),
        }
    }
}

impl Mul for Ast {
    type Output = Ast;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Ast::Zero, _) | (_, Ast::Zero) | (Ast::Constant(0), _) | (_, Ast::Constant(0)) => {
                Ast::Zero
            }
            (Ast::One, x) | (x, Ast::One) | (Ast::Constant(1), x) | (x, Ast::Constant(1)) => x,
            (Ast::Constant(a), Ast::Constant(b)) => Ast::Constant(a * b),
            (Ast::Constant(a), Ast::AddNode(b, c)) | (Ast::AddNode(b, c), Ast::Constant(a)) => {
                Ast::Constant(a).mul(*b).add(Ast::Constant(a).mul(*c))
            }
            (Ast::Constant(a), Ast::MulNode(b, c)) => match *b {
                Ast::Constant(b) => Ast::Constant(a * b).mul(*c),
                b => Ast::MulNode(
                    Box::new(Ast::Constant(a)),
                    Box::new(Ast::MulNode(Box::new(b), c)),
                ),
            },
            (Ast::IfLessThen(x, y, lt_res, geq_res), Ast::Constant(c))
            | (Ast::Constant(c), Ast::IfLessThen(x, y, lt_res, geq_res)) => Ast::IfLessThen(
                x,
                y,
                Box::new(lt_res.mul(Ast::Constant(c))),
                Box::new(geq_res.mul(Ast::Constant(c))),
            ),
            (Ast::MulNode(a, b), c) => a.mul(b.mul(c)),
            (Ast::IfLessThen(x, y, lt_res, geq_res), Ast::Variable(v)) => Ast::IfLessThen(
                x,
                y,
                Box::new(lt_res.mul(Ast::Variable(v))),
                Box::new(geq_res.mul(Ast::Variable(v))),
            ),
            (x, y) => Ast::MulNode(Box::new(x), Box::new(y)),
        }
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
            }
        }
    }

    fn to_i32(self) -> Option<i32> {
        match self.eval(&mut |_| None) {
            Err(_) => None,
            Ok(eval) => {
                if eval <= i32::MAX as i64 && eval >= i32::MIN as i64 {
                    Some(eval as i32)
                } else {
                    None
                }
            }
        }
    }

    fn if_less_then_else(self, other: Self, if_less: Self, if_not_less: Self) -> Self {
        // Pigeonhole optimisation for the "absolute value" pattern
        let other = match self {
            Ast::Constant(0) | Ast::Zero => match other {
                Ast::IfLessThen(a, b, if_less1, if_not_less1) => {
                    if a.strict_equal(&Ast::Zero) {
                        match (*b, *if_less1, *if_not_less1) {
                            (Ast::Variable(var), Ast::Variable(var2), Ast::MulNode(mul1, mul2)) => {
                                if var == var2 {
                                    match (*mul1, *mul2) {
                                        (Ast::Constant(-1), Ast::Variable(var3)) => {
                                            if var2 == var3 {
                                                return if_less;
                                            } else {
                                                Ast::IfLessThen(
                                                    a,
                                                    Box::new(Ast::Variable(var)),
                                                    Box::new(Ast::Variable(var2)),
                                                    Box::new(Ast::MulNode(
                                                        Box::new(Ast::Constant(-1)),
                                                        Box::new(Ast::Variable(var3)),
                                                    )),
                                                )
                                            }
                                        }
                                        (mul1, mul2) => Ast::IfLessThen(
                                            a,
                                            Box::new(Ast::Variable(var)),
                                            Box::new(Ast::Variable(var2)),
                                            Box::new(Ast::MulNode(Box::new(mul1), Box::new(mul2))),
                                        ),
                                    }
                                } else {
                                    Ast::IfLessThen(
                                        a,
                                        Box::new(Ast::Variable(var)),
                                        Box::new(Ast::Variable(var2)),
                                        Box::new(Ast::MulNode(mul1, mul2)),
                                    )
                                }
                            }
                            (b, if_less_1, if_not_less_1) => Ast::IfLessThen(
                                a,
                                Box::new(b),
                                Box::new(if_less_1),
                                Box::new(if_not_less_1),
                            ),
                        }
                    } else {
                        Ast::IfLessThen(a, b, if_less1, if_not_less1)
                    }
                }
                _ => other,
            },
            _ => other,
        };

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
