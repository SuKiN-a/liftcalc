use chumsky::prelude::*;

pub fn parser() -> impl Parser<char, Expr, Error = Simple<char>> {
    let int = text::int(10)
        .map(|s: String| Expr::Num(s.parse().unwrap()))
        .padded();

    let float = int.then_ignore(just(".")).then(int).map(|f: (Expr, Expr)| {
        Expr::Num(
            format!("{}.{}", f.0.unwrap_num(), f.1.unwrap_num())
                .parse()
                .unwrap(),
        )
    });

    let atom = float.or(int);

    let op = |c| just(c).padded();

    let unary = op('-')
        .repeated()
        .then(atom)
        .foldr(|_op, rhs| Expr::Neg(Box::new(rhs)));

    let product = unary
        .then(
            op('*')
                .to(Expr::Mul as fn(_, _) -> _)
                .or(op('/').to(Expr::Div as fn(_, _) -> _))
                .then(unary)
                .repeated(),
        )
        .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));

    let sum = product
        .then(
            op('+')
                .to(Expr::Add as fn(_, _) -> _)
                .or(op('-').to(Expr::Sub as fn(_, _) -> _))
                .then(product)
                .repeated(),
        )
        .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));

    sum.then_ignore(end())
}

#[derive(Debug)]
pub enum Expr {
    Num(f64),
    Neg(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
}

impl Expr {
    fn unwrap_num(self) -> f64 {
        match self {
            Expr::Num(num) => num,
            _ => panic!("expected Expr::Num, recieved {self:?}"),
        }
    }
}
