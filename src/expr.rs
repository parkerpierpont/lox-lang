use std::{fmt::Debug, rc::Rc};

use crate::token::{Token, TokenLiteral};
use downcast::{downcast, Any};

pub trait NamedExpr {
    fn name(&self) -> &'static str;
}

pub trait Expr: Any + Debug + NamedExpr {}

pub trait ExprVisitor<T: Default> {
    fn visit_binary_expr(&self, expr: &Binary) -> T;
    fn visit_grouping_expr(&self, expr: &Grouping) -> T;
    fn visit_literal_expr(&self, expr: &Literal) -> T;
    fn visit_unary_expr(&self, expr: &Unary) -> T;
}

pub trait VisitorTarget {
    fn accept<T: Default>(&self, visitor: impl ExprVisitor<T>) -> T;
}

impl VisitorTarget for Rc<dyn Expr> {
    fn accept<T: Default>(&self, visitor: impl ExprVisitor<T>) -> T {
        match self.name() {
            "Binary" => visitor.visit_binary_expr(self.downcast_ref::<Binary>().unwrap()),
            "Grouping" => visitor.visit_grouping_expr(self.downcast_ref::<Grouping>().unwrap()),
            "Literal" => visitor.visit_literal_expr(self.downcast_ref::<Literal>().unwrap()),
            "Unary" => visitor.visit_unary_expr(self.downcast_ref::<Unary>().unwrap()),
            _ => T::default(),
        }
    }
}

downcast!(dyn Expr);

pub type Expression = Rc<dyn Expr>;

#[derive(Debug, Clone)]
pub struct Binary {
    pub left: Expression,
    pub operator: Token,
    pub right: Expression,
}

impl Binary {
    pub fn new(left: Expression, operator: Token, right: Expression) -> Expression {
        Rc::new(Binary {
            left,
            operator,
            right,
        })
    }
}
impl Expr for Binary {}
impl NamedExpr for Binary {
    fn name(&self) -> &'static str {
        "Binary"
    }
}

#[derive(Debug, Clone)]
pub struct Grouping {
    pub expression: Expression,
}
impl Grouping {
    pub fn new(expression: Expression) -> Expression {
        Rc::new(Grouping { expression })
    }
}
impl Expr for Grouping {}
impl NamedExpr for Grouping {
    fn name(&self) -> &'static str {
        "Grouping"
    }
}

#[derive(Debug, Clone)]
pub struct Literal {
    pub value: TokenLiteral,
}
impl Literal {
    pub fn new(value: TokenLiteral) -> Expression {
        Rc::new(Literal { value })
    }
}
impl Expr for Literal {}
impl NamedExpr for Literal {
    fn name(&self) -> &'static str {
        "Literal"
    }
}

#[derive(Debug, Clone)]
pub struct Unary {
    pub operator: Token,
    pub right: Expression,
}
impl Unary {
    pub fn new(operator: Token, right: Expression) -> Expression {
        Rc::new(Unary { operator, right })
    }
}

impl Expr for Unary {}
impl NamedExpr for Unary {
    fn name(&self) -> &'static str {
        "Unary"
    }
}
